//! A session over WebSocket: the same commands and observations as HTTP, plus push.
//!
//! The client logs in, then sends `command`, `observe`, `subscribe` and `unsubscribe`
//! frames, each with an `id` it chooses; the driver answers each by id. Whenever the
//! world changes (a command, a job, a message through the Gate), the driver pushes the
//! events this actor may see as `event` frames, and re-runs every subscription whose
//! result set those events touched, pushing the fresh observation under the
//! subscription's id. A renderer holds no state of its own: it subscribes to what it
//! draws, evaluates timed values from their curves between pushes, and sends commands.
//!
//! Which events are pushed (S5): the actor's own and those addressed to it, the
//! world's news (arrivals, speech, names, births, the Clocktower: `observe::news_kind`),
//! and any event touching an entity one of its subscriptions holds. A session that
//! wants every event it may see, as before, logs in with `"events": "all"`. The
//! events come with the announcement of a change, read once for every session; a
//! session a change concerns in nothing takes no lock and sends nothing.
//!
//! ```json
//! > {"type":"login","actor":"dave","passphrase":"..."}
//! < {"type":"welcome","actor":"dave","cursor":131,"fresh":false}
//! > {"type":"subscribe","id":"room","query":{"root":"here","relations":{"contents":{}},"timed":"curve"}}
//! < {"type":"observation","id":"room", ...}
//! > {"type":"command","id":"c1","command":"attack","args":["wisp"]}
//! < {"type":"result","id":"c1","ok":true,"summary":"...","result":{...}}
//! < {"type":"event","event":{"seq":132,...}}
//! < {"type":"observation","id":"room", ...}
//! ```
//!
//! S11: a subscription is sent an observation only when something in it differs from
//! the last one sent. With `"patch": true` beside `"query"`, every observation after
//! the first carries `"patch"` in place of `"root"`: what changed, as [`diff`] says.

use crate::net::Shared;
use crate::observe::Request;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use foxing_log::Cursor;
use foxing_registry::{ActorKind, CallContext};
use foxing_store::Viewer;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

/// Most subscriptions one session may hold.
const MAX_SUBSCRIPTIONS: usize = 8;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Frame {
    Login {
        actor: String,
        #[serde(default)]
        passphrase: String,
        #[serde(default)]
        admin: Option<String>,
        /// `"all"`: every event this actor may see, as before S5. Otherwise the
        /// events that concern the session: the actor's own and those addressed to
        /// it, the world's news, and whatever touches something it subscribes to.
        #[serde(default)]
        events: Option<String>,
    },
    Command {
        id: Value,
        command: String,
        #[serde(default)]
        args: Value,
    },
    Observe {
        id: Value,
        query: Request,
    },
    Subscribe {
        id: Value,
        query: Request,
        /// S11: `true` asks for what changed since the last observation sent
        /// (`"patch"` in place of `"root"`) once the first has been sent whole.
        #[serde(default)]
        patch: bool,
    },
    Unsubscribe {
        id: Value,
    },
    Ping {
        #[serde(default)]
        id: Value,
    },
}

/// A subscription to refresh: its key, the fresh observation frame, and the ids it now covers.
type Refresh = (String, Value, HashSet<u64>);

struct Subscription {
    request: Request,
    /// Entity ids in the last observation; an event touching one re-runs the query.
    ids: HashSet<u64>,
    /// The last refresh failed: its error was sent once, and is not sent again
    /// until a refresh succeeds.
    failed: bool,
    /// S10 b: when its observation was last sent, and whether the world has
    /// changed under it since without one being sent.
    sent: Option<std::time::Instant>,
    stale: bool,
    /// S11: the root last sent, and whether the client asked for patches against it.
    last: Option<Value>,
    patch: bool,
}

impl Subscription {
    /// S11: how long somebody else's change waits. A room's observation grows with
    /// the crowd in it and so does the number of sessions it is run for, each under
    /// the world's lock: a crowded room is sent once a second, a packed one every
    /// two. One's own change is still sent at once.
    fn window(&self) -> std::time::Duration {
        REFRESH_EVERY
            * match self.ids.len() {
                0..=40 => 1,
                41..=100 => 2,
                _ => 4,
            }
    }

    /// S11: the frame to send for a fresh observation, and `None` when nothing in it
    /// differs from the one last sent. A session that asked for patches is sent
    /// what changed; the rest are sent the observation whole, as before.
    fn frame(&mut self, mut reply: Value) -> Option<Value> {
        let root = reply["root"].clone();
        let out = match &self.last {
            Some(last) => match diff(last, &root) {
                None => None,
                Some(patch) if self.patch => {
                    if let Some(o) = reply.as_object_mut() {
                        o.remove("root");
                        o.insert("patch".into(), patch);
                    }
                    Some(reply)
                }
                Some(_) => Some(reply),
            },
            None => Some(reply),
        };
        self.last = Some(root);
        out
    }
}

/// S11: what turns `old` into `new`, or `None` when they are the same. An object's
/// patch holds its changed keys (a changed object as its own patch) and `"$del"` for
/// the keys gone. A list of nodes (objects with distinct `id`s, as every relation is)
/// is `{"$ids": [the ids in order], "$set": {id: a new node whole, or a kept node's
/// patch}}`. Anything else that changed is the new value. The play page applies the
/// same (`applyPatch`).
pub(crate) fn diff(old: &Value, new: &Value) -> Option<Value> {
    if old == new {
        return None;
    }
    match (old, new) {
        (Value::Object(a), Value::Object(b)) => {
            let mut m = serde_json::Map::new();
            for (k, v) in b {
                match a.get(k) {
                    None => {
                        m.insert(k.clone(), v.clone());
                    }
                    Some(o) => {
                        if let Some(d) = diff(o, v) {
                            m.insert(k.clone(), d);
                        }
                    }
                }
            }
            let gone: Vec<Value> = a
                .keys()
                .filter(|k| !b.contains_key(*k))
                .map(|k| json!(k))
                .collect();
            if !gone.is_empty() {
                m.insert("$del".into(), Value::Array(gone));
            }
            Some(Value::Object(m))
        }
        (Value::Array(a), Value::Array(b)) => {
            let (Some(ia), Some(ib)) = (node_ids(a), node_ids(b)) else {
                return Some(new.clone());
            };
            let mut set = serde_json::Map::new();
            for (id, node) in ib.iter().zip(b) {
                match ia.iter().position(|x| x == id) {
                    None => {
                        set.insert(id.to_string(), node.clone());
                    }
                    Some(at) => {
                        if let Some(d) = diff(&a[at], node) {
                            set.insert(id.to_string(), d);
                        }
                    }
                }
            }
            Some(json!({"$ids": ib, "$set": set}))
        }
        _ => Some(new.clone()),
    }
}

/// The ids of a list of nodes, when it is one: every element an object with an `id`
/// no other has.
fn node_ids(list: &[Value]) -> Option<Vec<u64>> {
    let mut ids = Vec::with_capacity(list.len());
    for v in list {
        let id = v.as_object()?.get("id")?.as_u64()?;
        if ids.contains(&id) {
            return None;
        }
        ids.push(id);
    }
    Some(ids)
}

/// What [`diff`] undoes: `old` with `patch` applied. The server never needs it; the
/// tests hold the two together, and the play page's `applyPatch` is this in script.
#[cfg(test)]
fn patched(old: &Value, patch: &Value) -> Value {
    match (old, patch) {
        (Value::Array(a), Value::Object(p)) if p.contains_key("$ids") => {
            let set = p["$set"].as_object().cloned().unwrap_or_default();
            p["$ids"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .iter()
                .map(|id| {
                    let kept = a.iter().find(|n| n["id"] == *id);
                    match (kept, set.get(&id.to_string())) {
                        (Some(k), Some(d)) => patched(k, d),
                        (Some(k), None) => k.clone(),
                        (None, Some(n)) => n.clone(),
                        (None, None) => Value::Null,
                    }
                })
                .collect()
        }
        (Value::Object(a), Value::Object(p)) => {
            let mut out = a.clone();
            for (k, v) in p {
                if k == "$del" {
                    for gone in v.as_array().into_iter().flatten() {
                        out.remove(gone.as_str().unwrap_or(""));
                    }
                } else {
                    let next = match out.get(k) {
                        Some(o) if o.is_object() || o.is_array() => patched(o, v),
                        _ => v.clone(),
                    };
                    out.insert(k.clone(), next);
                }
            }
            Value::Object(out)
        }
        _ => patch.clone(),
    }
}

/// S10 b: a subscription is sent its observation again at most this often when the
/// change was somebody else's. A hundred names sent sixty sessions the whole room on
/// every change in it: 389 MB in six minutes, and the pushes held the world's lock
/// a quarter of the run. A change the session's own name made is sent at once; the
/// rest wait for the window to close and are sent once, as the room then stands.
const REFRESH_EVERY: std::time::Duration = std::time::Duration::from_millis(500);

/// Every entity id a node and its relations mention.
fn ids_of(node: &Value, out: &mut HashSet<u64>) {
    if let Some(id) = node["id"].as_u64() {
        out.insert(id);
    }
    if let Some(p) = node["parent"].as_u64() {
        out.insert(p);
    }
    if let Some(rels) = node["relations"].as_object() {
        for v in rels.values() {
            match v {
                Value::Array(items) => items.iter().for_each(|n| ids_of(n, out)),
                Value::Object(_) => ids_of(v, out),
                _ => {}
            }
        }
    }
}

/// The entity ids an event touches: its subject and, for creations and moves, the
/// container it entered or left.
fn touched(e: &foxing_log::Event) -> Vec<u64> {
    let mut out = Vec::new();
    for key in ["id", "parent", "from"] {
        if let Some(v) = e.payload.get(key).and_then(Value::as_u64) {
            out.push(v);
        }
    }
    out
}

/// S5: whether an event is sent to a session that did not ask for everything: the
/// actor's own, one addressed to or about the actor, the clock, or the world's news.
fn wanted(e: &foxing_log::Event, me: &str) -> bool {
    if matches!(&e.actor, foxing_log::Actor::Citizen(a) if a == me) {
        return true;
    }
    if ["to", "citizen", "owner", "actor", "from", "name"]
        .iter()
        .any(|k| e.payload.get(*k).and_then(Value::as_str) == Some(me))
        || crate::disremembered::combat::addressed(&e.payload, me)
    {
        return true;
    }
    e.kind.starts_with("store.clock_") || crate::observe::news_kind(&e.kind)
}

pub async fn upgrade(ws: WebSocketUpgrade, State(s): State<Shared>) -> Response {
    ws.max_message_size(crate::net::MAX_BODY_BYTES)
        .max_frame_size(crate::net::MAX_BODY_BYTES)
        .on_upgrade(move |socket| run(socket, s))
}

/// Counts a push when it leaves the lock, however it leaves.
struct PushTimer {
    asked: std::time::Instant,
    locked: std::time::Instant,
}

impl Drop for PushTimer {
    fn drop(&mut self) {
        crate::ops::pushed(self.locked - self.asked, self.locked.elapsed());
    }
}

/// Why a session ended, said in the journal when it was not the client's own
/// goodbye: sessions have been dropped mid-run with no close frame since the first
/// load test (two of twelve, one of thirty, eight of sixty), and nothing said which
/// side did it.
struct Ending {
    began: std::time::Instant,
    who: Option<String>,
    why: String,
}
impl Drop for Ending {
    fn drop(&mut self) {
        if self.why != "the client closed" {
            eprintln!(
                "session ended: {} after {}s: {}",
                self.who.as_deref().unwrap_or("(not logged in)"),
                self.began.elapsed().as_secs(),
                self.why
            );
        }
    }
}

async fn run(socket: WebSocket, s: Shared) {
    let mut ending = Ending {
        began: std::time::Instant::now(),
        who: None,
        why: "the task was dropped".into(),
    };
    let (mut tx, mut rx) = socket.split();
    let mut changes = s.changed.subscribe();
    let mut actor: Option<String> = None;
    let mut admin = false;
    let mut cursor: u64 = 0;
    let mut subs: HashMap<String, Subscription> = HashMap::new();
    let mut all_events = false;

    macro_rules! send {
        ($v:expr) => {
            if let Err(e) = tx
                .send(Message::Text(
                    serde_json::to_string(&$v).unwrap_or_default().into(),
                ))
                .await
            {
                ending.why = format!("a send failed: {e}");
                return;
            }
        };
    }

    send!(json!({"type": "hello", "world": "disremembered", "protocol": "foxing-session/0"}));
    loop {
        // When the first held-back subscription may be sent again.
        let due: Option<std::time::Instant> = subs
            .values()
            .filter(|sub| sub.stale)
            .map(|sub| {
                sub.sent
                    .map_or_else(std::time::Instant::now, |t| t + sub.window())
            })
            .min();
        tokio::select! {
            _ = async {
                match due {
                    Some(d) => tokio::time::sleep_until(d.into()).await,
                    None => std::future::pending::<()>().await,
                }
            } => {
                let Some(name) = actor.clone() else { continue };
                let now = std::time::Instant::now();
                let ready: Vec<String> = subs
                    .iter()
                    .filter(|(_, sub)| sub.stale && sub.sent.map_or(true, |t| now >= t + sub.window()))
                    .map(|(k, _)| k.clone())
                    .collect();
                let refresh: Vec<Refresh> = {
                    let push_asked = std::time::Instant::now();
                    let Ok(w) = s.world.lock() else { continue };
                    let _counted = PushTimer { asked: push_asked, locked: std::time::Instant::now() };
                    ready
                        .iter()
                        .filter_map(|k| {
                            let sub = subs.get(k)?;
                            let id: Value = serde_json::from_str(k).unwrap_or(Value::String(k.clone()));
                            let reply = observe_frame(&w, &name, admin, &id, &sub.request, &s);
                            let mut ids = HashSet::new();
                            ids_of(&reply["root"], &mut ids);
                            Some((k.clone(), reply, ids))
                        })
                        .collect()
                };
                for (k, reply, ids) in refresh {
                    let ok = reply["type"] == "observation";
                    let mut out = Some(reply);
                    if let Some(sub) = subs.get_mut(&k) {
                        if ok {
                            sub.ids = ids;
                            // S11: what changed, or nothing when nothing did.
                            out = out.and_then(|r| sub.frame(r));
                        } else if sub.failed {
                            out = None;
                        }
                        sub.failed = !ok;
                        sub.sent = Some(std::time::Instant::now());
                        sub.stale = false;
                    }
                    if let Some(reply) = out {
                        send!(reply);
                    }
                }
            }
            incoming = rx.next() => {
                let msg = match incoming {
                    Some(Ok(msg)) => msg,
                    Some(Err(e)) => {
                        ending.why = format!("reading from the client failed: {e}");
                        return;
                    }
                    None => {
                        ending.why = "the client's stream ended with no close frame".into();
                        return;
                    }
                };
                let text = match msg {
                    Message::Text(t) => t.to_string(),
                    Message::Close(_) => {
                        ending.why = "the client closed".into();
                        return;
                    }
                    _ => continue,
                };
                let frame: Frame = match serde_json::from_str(&text) {
                    Ok(f) => f,
                    Err(e) => {
                        send!(json!({"type": "error", "error": format!("bad frame: {e}")}));
                        continue;
                    }
                };
                match frame {
                    Frame::Login { actor: name, passphrase, admin: token, events } => {
                        all_events = events.as_deref() == Some("all");
                        // The cursor is the log's head from before the login, so what the
                        // login itself said to the name (the Docent's greeting, words left
                        // for them) is pushed rather than lost behind the cursor.
                        let logged_in: Result<(bool, u64), String> = {
                            let Ok(mut w) = s.world.lock() else { continue };
                            let before = w.log.head(&w.id).ok().flatten().map(|x| x.0).unwrap_or(0);
                            crate::net::login_for_session(&mut w, &name, &passphrase).map(|fresh| {
                                // The page logs in over HTTP first, and the arrival (with the
                                // Docent's greeting) was spoken then: start from just before
                                // the name's latest arrival if it was within the minute.
                                let now = w.now();
                                let arrived = w
                                    .recent_events(400)
                                    .iter()
                                    .rev()
                                    .find(|e| e.kind == crate::world::ARRIVED && e.payload["actor"] == name.as_str() && now.since(e.at).0 < 60_000)
                                    .map(|e| e.seq.0.saturating_sub(1));
                                (fresh, arrived.map_or(before, |a| a.min(before)))
                            })
                        };
                        let (fresh, head) = match logged_in {
                            Ok(v) => v,
                            Err(e) => {
                                send!(json!({"type": "error", "error": e}));
                                continue;
                            }
                        };
                        admin = s.is_admin(token.as_deref());
                        actor = Some(name.clone());
                        cursor = head;
                        subs.clear();
                        send!(json!({"type": "welcome", "actor": name, "cursor": head, "fresh": fresh, "admin": admin}));
                    }
                    Frame::Ping { id } => send!(json!({"type": "pong", "id": id})),
                    Frame::Command { id, command, args } => {
                        let Some(name) = actor.clone() else {
                            send!(json!({"type": "result", "id": id, "ok": false, "error": "log in first"}));
                            continue;
                        };
                        let kind = if admin { ActorKind::Admin } else { ActorKind::Citizen };
                        // P7: how long the world took, apart from the wire (the wait
                        // for the lock and the command itself), on every reply.
                        let asked = std::time::Instant::now();
                        let mut reply = {
                            let Ok(mut w) = s.world.lock() else { continue };
                            let mark = crate::ops::Mark::locked(asked);
                            let verb = command.clone();
                            let mut v = if command == "observe" {
                                // A read, not speech: runs outside the gate.
                                let ctx = CallContext { actor: name, kind };
                                match w.call(&ctx, &command, &args) {
                                    Ok(out) => json!({"type": "result", "id": id, "ok": true, "summary": out.summary, "result": out.result}),
                                    Err(e) => json!({"type": "result", "id": id, "ok": false, "error": e.to_string(), "detail": e}),
                                }
                            } else {
                                let Ok(mut budget) = s.hour_budget.lock() else { continue };
                                w.begin_unit();
                                let p = crate::net::compiled_passage(&mut w, &s, &mut budget, &name, kind, crate::gate::Via::Session, &command, &args);
                                w.end_unit();
                                let mut v = crate::net::passage_json(&p);
                                v["type"] = json!("result");
                                v["id"] = id;
                                v
                            };
                            // S2: what the world spent it on.
                            v["took"] = mark.done(&verb);
                            v
                        };
                        reply["took_ms"] = json!((asked.elapsed().as_secs_f64() * 1000.0 * 10.0).round() / 10.0);
                        s.announce();
                        send!(reply);
                    }
                    Frame::Observe { id, query } | Frame::Subscribe { id, query, .. } if actor.is_none() => {
                        send!(json!({"type": "error", "id": id, "error": "log in first"}));
                        let _ = query;
                    }
                    Frame::Observe { id, query } => {
                        let name = actor.clone().unwrap_or_default();
                        let reply = {
                            let Ok(w) = s.world.lock() else { continue };
                            observe_frame(&w, &name, admin, &id, &query, &s)
                        };
                        send!(reply);
                    }
                    Frame::Subscribe { id, query, patch } => {
                        let key = id.to_string();
                        if subs.len() >= MAX_SUBSCRIPTIONS && !subs.contains_key(&key) {
                            send!(json!({"type": "error", "id": id, "error": format!("at most {MAX_SUBSCRIPTIONS} subscriptions")}));
                            continue;
                        }
                        let name = actor.clone().unwrap_or_default();
                        let (reply, ids) = {
                            let Ok(w) = s.world.lock() else { continue };
                            let reply = observe_frame(&w, &name, admin, &id, &query, &s);
                            let mut ids = HashSet::new();
                            ids_of(&reply["root"], &mut ids);
                            (reply, ids)
                        };
                        if reply["type"] == "observation" {
                            subs.insert(key, Subscription { request: query, ids, failed: false, sent: Some(std::time::Instant::now()), stale: false, last: Some(reply["root"].clone()), patch });
                        }
                        send!(reply);
                    }
                    Frame::Unsubscribe { id } => {
                        subs.remove(&id.to_string());
                        send!(json!({"type": "unsubscribed", "id": id}));
                    }
                }
            }
            changed = changes.recv() => {
                if changed.is_err() && !matches!(changed, Err(tokio::sync::broadcast::error::RecvError::Lagged(_))) {
                    ending.why = "the world's announcements closed".into();
                    return;
                }
                let Some(name) = actor.clone() else { continue };
                if ending.who.is_none() {
                    ending.who = Some(name.clone());
                }
                // S5: the events came with the announcement. A session they concern in
                // nothing (not its own, not addressed to it, not news, touching nothing
                // it subscribes to) takes no lock and sends nothing; fifty names once
                // woke thirty sessions under the world's lock for every command.
                let batch = changed.ok().filter(|b| {
                    b.events.first().is_some_and(|e| e.seq.0 <= cursor + 1) || b.head <= cursor
                });
                if let Some(b) = &batch {
                    if b.head <= cursor {
                        continue;
                    }
                    let concerned = b.events.iter().filter(|e| e.seq.0 > cursor).any(|e| {
                        all_events
                            || admin
                            || wanted(e, &name)
                            || touched(e).iter().any(|id| subs.values().any(|sub| sub.ids.contains(id)))
                    });
                    if !concerned {
                        cursor = b.head;
                        crate::ops::push_skipped();
                        continue;
                    }
                }
                // Everything since the cursor this actor may see, and which subscriptions it touched.
                let push_asked = std::time::Instant::now();
                let viewer = if admin { Viewer::admin() } else { Viewer::actor(&name) };
                let (events, refresh, held): (Vec<Value>, Vec<Refresh>, Vec<String>) = if let Some(b) = &batch {
                    // S13: the events came judged (who may see each, and which of its
                    // fields), so a session filters them for itself with no lock. It
                    // takes the world's lock only to run an observation that is due.
                    let fresh: Vec<(&std::sync::Arc<foxing_log::Event>, &crate::observe::Seen)> = b
                        .events
                        .iter()
                        .zip(&b.seen)
                        .filter(|(e, _)| e.seq.0 > cursor)
                        .collect();
                    cursor = b.head;
                    let mut dirty: HashSet<String> = HashSet::new();
                    for (e, _) in &fresh {
                        for id in touched(e) {
                            for (k, sub) in &subs {
                                if sub.ids.contains(&id) {
                                    dirty.insert(k.clone());
                                }
                            }
                        }
                    }
                    let events: Vec<Value> = fresh
                        .iter()
                        .filter(|(e, _)| {
                            all_events
                                || admin
                                || wanted(e, &name)
                                || touched(e).iter().any(|id| subs.values().any(|sub| sub.ids.contains(id)))
                        })
                        .filter(|(e, seen)| crate::observe::visible_as(&viewer, e, seen))
                        .map(|(e, seen)| crate::observe::redacted_as(&viewer, (***e).clone(), seen))
                        .map(|e| json!({"type": "event", "event": e}))
                        .collect();
                    // S10 b: a change this name made is shown at once; anybody
                    // else's waits for the subscription's window.
                    let mine = fresh
                        .iter()
                        .any(|(e, _)| matches!(&e.actor, foxing_log::Actor::Citizen(a) if *a == name));
                    let mut due = Vec::new();
                    let mut held = Vec::new();
                    for k in dirty {
                        let Some(sub) = subs.get(&k) else { continue };
                        if !mine && sub.sent.is_some_and(|t| t.elapsed() < sub.window()) {
                            held.push(k);
                        } else {
                            due.push(k);
                        }
                    }
                    let mut refresh = Vec::new();
                    if due.is_empty() {
                        crate::ops::push_skipped();
                    } else {
                        let Ok(w) = s.world.lock() else { continue };
                        // S2: what every open session costs each time the world changes.
                        let _counted = PushTimer { asked: push_asked, locked: std::time::Instant::now() };
                        for k in due {
                            let Some(sub) = subs.get(&k) else { continue };
                            let id: Value = serde_json::from_str(&k).unwrap_or(Value::String(k.clone()));
                            let reply = observe_frame(&w, &name, admin, &id, &sub.request, &s);
                            let mut ids = HashSet::new();
                            ids_of(&reply["root"], &mut ids);
                            refresh.push((k, reply, ids));
                        }
                    }
                    (events, refresh, held)
                } else {
                    // The first announcement, a lagged listener, a wide gap: from the
                    // log, under the lock, as every push once was.
                    let Ok(w) = s.world.lock() else { continue };
                    let push_locked = std::time::Instant::now();
                    let head = w.log.head(&w.id).ok().flatten().map(|x| x.0).unwrap_or(0);
                    if head <= cursor { continue; }
                    let _counted = PushTimer { asked: push_asked, locked: push_locked };
                    let now = w.now();
                    let all: Vec<foxing_log::Event> = w.log.read(&w.id, Cursor { after: cursor }, 500).unwrap_or_default();
                    cursor = head;
                    let mut dirty: HashSet<String> = HashSet::new();
                    for e in &all {
                        for id in touched(e) {
                            for (k, sub) in &subs {
                                if sub.ids.contains(&id) {
                                    dirty.insert(k.clone());
                                }
                            }
                        }
                    }
                    let events: Vec<Value> = all
                        .iter()
                        .filter(|e| {
                            all_events
                                || admin
                                || wanted(e, &name)
                                || touched(e).iter().any(|id| subs.values().any(|sub| sub.ids.contains(id)))
                        })
                        .filter(|e| crate::observe::visible(&w, &viewer, now, e))
                        .map(|e| crate::observe::redacted(&w, &viewer, e.clone()))
                        .map(|e| json!({"type": "event", "event": e}))
                        .collect();
                    let mine = all
                        .iter()
                        .any(|e| matches!(&e.actor, foxing_log::Actor::Citizen(a) if *a == name));
                    let mut refresh = Vec::new();
                    let mut held = Vec::new();
                    for k in dirty {
                        let Some(sub) = subs.get(&k) else { continue };
                        if !mine && sub.sent.is_some_and(|t| t.elapsed() < sub.window()) {
                            held.push(k);
                            continue;
                        }
                        let id: Value = serde_json::from_str(&k).unwrap_or(Value::String(k.clone()));
                        let reply = observe_frame(&w, &name, admin, &id, &sub.request, &s);
                        let mut ids = HashSet::new();
                        ids_of(&reply["root"], &mut ids);
                        refresh.push((k, reply, ids));
                    }
                    (events, refresh, held)
                };
                for k in held {
                    if let Some(sub) = subs.get_mut(&k) {
                        sub.stale = true;
                    }
                }
                for e in events {
                    send!(e);
                }
                for (k, reply, ids) in refresh {
                    // A refresh that failed (a room past the query's breadth, a root
                    // gone for a moment) keeps what the subscription covered, so the
                    // next change tries again; emptied, it never woke again, and a
                    // crowd of fifty at the Threshold left every room panel dead.
                    let ok = reply["type"] == "observation";
                    let mut out = Some(reply);
                    if let Some(sub) = subs.get_mut(&k) {
                        if ok {
                            sub.ids = ids;
                            // S11: what changed, or nothing when nothing did.
                            out = out.and_then(|r| sub.frame(r));
                        } else if sub.failed {
                            out = None;
                        }
                        sub.failed = !ok;
                        sub.sent = Some(std::time::Instant::now());
                        sub.stale = false;
                    }
                    if let Some(reply) = out {
                        send!(reply);
                    }
                }
            }
        }
    }
}

fn observe_frame(
    w: &crate::world::World,
    name: &str,
    admin: bool,
    id: &Value,
    query: &Request,
    s: &Shared,
) -> Value {
    let viewer = if admin {
        Viewer::admin()
    } else {
        Viewer::actor(name)
    };
    let kind = if admin {
        ActorKind::Admin
    } else {
        ActorKind::Citizen
    };
    // S14: from the room's one run when it serves this viewer.
    if let Some(mut v) = w.observe_shared(&viewer, kind, query, &s.limits, &s.rooms) {
        crate::ops::observation_shared();
        v["type"] = json!("observation");
        v["id"] = id.clone();
        return v;
    }
    match w.observe(&viewer, kind, query, &s.limits) {
        Ok(obs) => {
            let mut v = serde_json::to_value(obs).unwrap_or_default();
            v["type"] = json!("observation");
            v["id"] = id.clone();
            v
        }
        Err(e) => json!({"type": "error", "id": id, "error": e.to_string(), "detail": e}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_patch_is_what_changed_and_applied_gives_the_new_observation() {
        let old = json!({"id": 1, "kind": "room", "components": {"room": {"name": "The Threshold", "lit": null}},
            "relations": {"contents": [
                {"id": 2, "kind": "citizen", "components": {"body": {"hp": 30, "engaged": null}}},
                {"id": 3, "kind": "creature", "components": {"creature": {"hp": 8.0, "name": "wisp"}}},
                {"id": 4, "kind": "item", "components": {"item": {"name": "a stone roll"}}}],
              "exits": [{"id": 9, "via": "north"}, {"id": 9, "via": "gate"}]}});
        let new = json!({"id": 1, "kind": "room", "components": {"room": {"name": "The Threshold"}},
            "relations": {"contents": [
                {"id": 3, "kind": "creature", "components": {"creature": {"hp": 5.0, "name": "wisp"}}},
                {"id": 2, "kind": "citizen", "components": {"body": {"hp": 30, "engaged": 3}}},
                {"id": 7, "kind": "citizen", "components": {"body": {"hp": 30, "engaged": null}}}],
              "exits": [{"id": 9, "via": "north"}, {"id": 9, "via": "gate"}, {"id": 10, "via": "east"}]}});
        assert_eq!(diff(&old, &old), None);
        let patch = diff(&old, &new).expect("they differ");
        assert_eq!(patched(&old, &patch), new);
        // The patch says only what changed: the wisp's hit points, not its name.
        let wisp = &patch["relations"]["contents"]["$set"]["3"];
        assert_eq!(*wisp, json!({"components": {"creature": {"hp": 5.0}}}));
        assert_eq!(patch["relations"]["contents"]["$ids"], json!([3, 2, 7]));
        assert_eq!(patch["components"]["room"]["$del"], json!(["lit"]));
        // Exits to one room by two names share an id: not a list of nodes, sent whole.
        assert!(patch["relations"]["exits"].is_array());

        // A subscription that did not ask is sent the whole or nothing.
        let frame = |root: &Value| json!({"type": "observation", "id": "room", "root": root});
        let mut sub = Subscription {
            request: serde_json::from_value(json!({"root": "here"})).unwrap(),
            ids: HashSet::new(),
            failed: false,
            sent: None,
            stale: false,
            last: Some(old.clone()),
            patch: false,
        };
        assert_eq!(
            sub.frame(frame(&old)),
            None,
            "nothing changed, nothing sent"
        );
        assert_eq!(sub.frame(frame(&new)), Some(frame(&new)));
        sub.patch = true;
        let sent = sub.frame(frame(&old)).expect("changed back");
        assert!(sent.get("root").is_none());
        assert_eq!(patched(&new, &sent["patch"]), old);
        // And the window widens with the crowd.
        assert_eq!(sub.window(), REFRESH_EVERY);
        sub.ids = (0..60).collect();
        assert_eq!(sub.window(), REFRESH_EVERY * 2);
        sub.ids = (0..200).collect();
        assert_eq!(sub.window(), REFRESH_EVERY * 4);
    }
}
