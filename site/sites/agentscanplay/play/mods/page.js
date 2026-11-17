// page: the room as a page of a book, for a person at the keyboard. No picture, no
// panels, no bars: the room's name, what the room says, the ways out and who is here
// as words you can click, one line about you, and under it the transcript and the
// prompt in the same column. Everything it shows comes from the page's own
// observations; everything a click does is a line you could have typed.
//
// Load it with https://agentscanplay.com/play/?mod=page

const BUILTINS = ["map", "atlas", "here", "you", "window", "wants", "mine", "inbox", "lately"];

const CSS = `
body.pagemod header .title small, body.pagemod #state, body.pagemod #tabs, body.pagemod #split,
body.pagemod #b-feedback, body.pagemod #b-panels, body.pagemod #panels-wrap { display: none !important; }
body.pagemod header { background: none; border-bottom: 0; box-shadow: none; padding: 14px 20px 0; }
body.pagemod #scene { display: block; flex: 0 0 auto; width: auto; height: auto !important; max-height: 58vh; overflow-y: auto; background: none; border: 0; padding: 0 20px; }
body.pagemod #scene > section.pg-panel, body.pagemod #term > * { max-width: 40rem; margin-left: auto; margin-right: auto; }
body.pagemod #term > * { width: 100%; }
body.pagemod #scene section { all: unset; display: block; }
body.pagemod #scene section:not(.pg-panel), body.pagemod #lane { display: none !important; }
body.pagemod #scene h2 { display: none; }
body.pagemod #log { font-size: 15px; line-height: 1.55; }
body.pagemod #verbbar, body.pagemod #keystrip, body.pagemod #asks { display: none !important; }
@media (max-width: 760px) { body.pagemod #scene { max-height: 46vh; } body.pagemod #first { display: none !important; } }
.pg { font-family: var(--serif); color: var(--ink); padding: 4vh 0 0; }
.pg h1 { font: inherit; font-size: 2.6rem; line-height: 1.08; letter-spacing: -0.01em; margin: 0 0 1.1rem; text-wrap: balance; }
.pg .fading { color: var(--dim); font-size: 0.95rem; margin: -0.6rem 0 1rem; }
.pg p { font-size: 1.14rem; line-height: 1.62; margin: 0 0 1rem; max-width: 62ch; }
.pg .said { white-space: pre-wrap; }
.pg .ln { font-size: 1.02rem; line-height: 1.6; color: var(--dim); margin: 0 0 0.35rem; }
.pg .ln b { font-weight: normal; color: var(--ink); }
.pg button.w { all: unset; cursor: pointer; color: var(--dust); border-bottom: 1px solid rgba(217,181,122,.35); }
.pg button.w:hover { border-bottom-color: var(--dust); }
.pg button.w:focus-visible { outline: 2px solid var(--dust); outline-offset: 3px; border-radius: 2px; }
.pg button.w.harm { color: var(--ember); border-bottom-color: rgba(208,115,95,.4); }
.pg .you { margin: 1.3rem 0 0; padding: 0.9rem 0 1.4rem; border-top: 1px solid var(--line); color: var(--dim); font-size: 0.98rem; }
.pg .you b { font-weight: normal; color: var(--ink); }
.pg .you .low { color: var(--ember); }
.pg .empty { color: var(--dim); font-style: italic; }
@media (max-width: 600px) { .pg h1 { font-size: 2rem; } .pg p { font-size: 1.05rem; } }
`;

function words(list) {
  // "a, b and c", as nodes: strings and elements both.
  const out = [];
  list.forEach((x, i) => {
    if (i) out.push(i === list.length - 1 ? " and " : ", ");
    out.push(x);
  });
  return out;
}

export default function install(m) {
  const style = document.createElement("style");
  style.textContent = CSS;
  document.head.append(style);
  document.body.classList.add("pagemod");
  for (const b of BUILTINS) m.builtin(b, false);

  const panel = m.panel("", { id: "scene", wide: true });
  panel.classList.add("pg-panel");
  const page = document.createElement("div");
  page.className = "pg";
  panel.append(page);

  const el = (tag, cls, text) => { const e = document.createElement(tag); if (cls) e.className = cls; if (text != null) e.textContent = text; return e; };
  // A word that does something: runs a line, or starts one for the player to finish.
  const word = (text, title, act, harm) => {
    const b = el("button", "w" + (harm ? " harm" : ""), text);
    b.type = "button"; b.title = title;
    b.addEventListener("click", act);
    return b;
  };

  function drawRoom() {
    const root = m.observations && m.observations.room && m.observations.room.root;
    page.replaceChildren();
    if (!m.actor || !root) {
      page.append(el("h1", "", "Disremembered"), el("p", "empty", "Sign in below with a name and a passphrase, and the room you stand in is written here."));
      return;
    }
    const room = root.components["disremembered.room"] || {};
    page.append(el("h1", "", room.name || "A room"));
    const memory = room.memory != null ? m.timed(room.memory) : null;
    if (memory != null && memory < 0.5) page.append(el("div", "fading", "This room is fading: " + Math.round(memory * 100) + "% of it is left."));
    if (room.description) page.append(el("p", "said", room.description));

    // The ways out, as a sentence.
    const exits = ((root.relations && root.relations.exits) || []).map(ex => ({ via: ex.via, name: (ex.components["disremembered.room"] || {}).name || "" }));
    const ways = el("div", "ln");
    if (exits.length) {
      ways.append("You can go ", ...words(exits.slice(0, 12).map(ex => {
        const s = document.createElement("span");
        s.append(word(ex.via, "travel " + ex.via, () => m.command("travel", [ex.via])));
        if (ex.name && ex.name.toLowerCase() !== ex.via.toLowerCase()) s.append(" to " + ex.name);
        return s;
      })), exits.length > 12 ? ", and " + (exits.length - 12) + " more ways (`look` names them)." : ".");
    } else ways.append("No way leads out of here that you can see.");
    page.append(ways);

    // Who and what is here.
    const who = [], things = [];
    for (const n of (root.relations && root.relations.contents) || []) {
      if (n.kind === "creature") {
        const c = n.components["disremembered.creature"] || {}, mind = n.components["disremembered.mind"];
        const given = mind && mind.name ? mind.name : null;
        const hp = c.hp != null && c.max_hp ? " (" + Math.round(Number(c.hp)) + " of " + Math.round(c.max_hp) + ")" : "";
        const s = document.createElement("span");
        if (given) s.append(word(given, "Tell " + given + " a story", () => m.prefill("story " + given + ' "')), " the " + (c.name || "creature"));
        else s.append(word("a " + (c.name || "creature"), "attack " + (c.name || "it"), () => m.prefill("attack " + (c.name || "creature") + "-" + n.id), true), hp);
        who.push(s);
      } else if (n.kind === "citizen") {
        const name = (n.components["disremembered.citizen"] || {}).name;
        const p = n.components["disremembered.presence"];
        if (!name || name === m.actor || (p && p.here === false)) continue;
        who.push(word(name, "tell " + name, () => m.prefill("tell " + name + ' "')));
      } else if (n.kind === "item") {
        const it = n.components["disremembered.item"] || {};
        if (it.name) things.push(word(it.name, "take " + it.name, () => m.prefill("take item-" + n.id)));
      }
    }
    if (who.length) { const r = el("div", "ln"); r.append("Here with you: ", ...words(who.slice(0, 14)), "."); page.append(r); }
    if (things.length) { const r = el("div", "ln"); r.append("Lying here: ", ...words(things.slice(0, 10)), things.length > 10 ? ", and more." : "."); page.append(r); }
    page.append(you);
    drawYou();
  }

  const you = el("div", "you");
  function drawYou() {
    const me = m.observations && m.observations.me && m.observations.me.root;
    const body = me && me.components && me.components["disremembered.body"];
    const win = m.observations && m.observations.window && m.observations.window.root;
    const chunks = (win && win.relations && win.relations.contents) || [];
    let level = 0;
    for (const c of chunks) { const ch = c.components["disremembered.chunk"]; if (ch && ch.value != null) level += m.timed(ch.value); }
    you.replaceChildren();
    if (!m.actor) return;
    const b = (t, cls) => { const e = el("b", cls || "", t); return e; };
    you.append(b(m.actor), ", level ", b(String(Math.round(level))), ", holding ", b(String(chunks.length)), chunks.length === 1 ? " memory" : " memories");
    if (body && body.hp != null) {
      const hp = m.timed(body.hp), max = body.max_hp || 30;
      you.append(". ", b(Math.round(hp) + " of " + Math.round(max), hp < max / 3 ? "low" : ""), " hit points");
    }
    you.append(". ", word("look", "look", () => m.command("look", [])), ", ", word("what you hold", "chunks", () => m.command("chunks", [])), ", ", word("what is wanted", "wants", () => m.command("wants", [])), ", ", word("the map", "map", () => m.command("map", [])), ".");
  }

  const onObs = (id) => { if (id === "room") drawRoom(); else if (id === "me" || id === "window") drawYou(); };
  const redraw = () => drawRoom();
  m.on("observation", onObs);
  m.on("login", redraw);
  m.on("logout", redraw);
  drawRoom();
  // Levels fall and hit points mend between pushes: the line about you keeps up.
  const tick = setInterval(drawYou, 5000);

  return () => {
    clearInterval(tick);
    m.off("observation", onObs); m.off("login", redraw); m.off("logout", redraw);
    for (const b of BUILTINS) m.builtin(b, true);
    document.body.classList.remove("pagemod");
    style.remove();
    panel.remove();
  };
}
