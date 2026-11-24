//! What things are called, in place of their numbers.
//!
//! David, 11-22: "we should hide the creature identifier from the player (give each
//! an adjective to distinguish them)", then, of items and chunks: "yes we should
//! apply the same". A memory was chunk-32767 and a thing on the ground item-28567.
//!
//! A memory has a word of its own, from its id, which it keeps for as long as it
//! lasts: amber, birch, cedar. Two in one window that would share a word are told
//! apart by a number after it (amber, amber-2), the older first; that is rare. A
//! thing is called by its name, and where two of a name lie together, by a word
//! before it, as twin creatures are (the dusty wisp ember).
//!
//! The world's own lines are written with the ids, as they always were, and the
//! ids are still understood wherever one is typed. What a player is shown goes
//! through `humanize` on its way out: the narrator's lines as they are written to
//! the record, and a reply as the gate renders it. The play page gives memories
//! the same words by the same rule (`CHUNK_WORDS` there must match, in order).

use crate::world::World;
use foxing_store::EntityId;
use serde_json::Value;

pub const CHUNK_WORDS: &[&str] = &[
    "amber",
    "ash",
    "aspen",
    "azure",
    "barley",
    "basalt",
    "beech",
    "birch",
    "bramble",
    "brass",
    "briar",
    "bronze",
    "calico",
    "cedar",
    "chalk",
    "cherry",
    "cinder",
    "clay",
    "clover",
    "cobalt",
    "copper",
    "coral",
    "cotton",
    "crimson",
    "damson",
    "dapple",
    "denim",
    "dove",
    "dune",
    "ebony",
    "elder",
    "elm",
    "fawn",
    "fennel",
    "fern",
    "flax",
    "flint",
    "garnet",
    "ginger",
    "gorse",
    "granite",
    "hazel",
    "heather",
    "hemp",
    "henna",
    "holly",
    "indigo",
    "iris",
    "ivory",
    "ivy",
    "jade",
    "jasper",
    "juniper",
    "kelp",
    "khaki",
    "larch",
    "laurel",
    "lichen",
    "lilac",
    "lime",
    "linen",
    "loam",
    "madder",
    "maple",
    "marl",
    "mauve",
    "mint",
    "moss",
    "mulberry",
    "myrtle",
    "nettle",
    "nutmeg",
    "oak",
    "oat",
    "ochre",
    "olive",
    "onyx",
    "opal",
    "orchid",
    "osier",
    "pearl",
    "peat",
    "pewter",
    "pine",
    "plum",
    "poppy",
    "quartz",
    "quince",
    "raven",
    "reed",
    "rowan",
    "ruby",
    "russet",
    "rye",
    "saffron",
    "sage",
    "sable",
    "sand",
    "sedge",
    "sepia",
    "shale",
    "sienna",
    "silver",
    "slate",
    "sorrel",
    "spruce",
    "straw",
    "suede",
    "sumac",
    "tallow",
    "tansy",
    "teal",
    "thistle",
    "thyme",
    "topaz",
    "tulip",
    "umber",
    "velvet",
    "vetch",
    "violet",
    "walnut",
    "wheat",
    "willow",
    "woad",
    "yarrow",
    "yew",
    "zinc",
    "agate",
    "alder",
    "anise",
    "apricot",
    "bay",
    "beryl",
    "borage",
    "burdock",
    "buttercup",
    "campion",
    "caraway",
    "chicory",
    "cicely",
    "comfrey",
    "cowslip",
    "cress",
    "daisy",
    "dill",
    "dock",
    "gentian",
    "harebell",
    "hawthorn",
    "hyssop",
    "larkspur",
    "lavender",
    "lupin",
    "mallow",
    "marigold",
    "medlar",
    "mistle",
    "mullein",
    "oxlip",
    "parsley",
    "pansy",
    "peony",
    "primrose",
    "ragwort",
    "rosemary",
    "sloe",
    "speedwell",
    "tamarisk",
    "teasel",
    "trefoil",
    "valerian",
    "verbena",
    "wormwood",
];

fn base(id: u64) -> &'static str {
    CHUNK_WORDS[(id % CHUNK_WORDS.len() as u64) as usize]
}

/// A memory's word: `amber`, or `amber-2` beside an older amber.
pub fn chunk_word(world: &World, chunk: EntityId) -> String {
    let b = base(chunk.0);
    let older = world
        .store
        .entity(chunk)
        .and_then(|e| e.parent)
        .map(|p| {
            world
                .store
                .children(p)
                .filter(|e| e.kind == "chunk" && e.id.0 < chunk.0 && base(e.id.0) == b)
                .count()
        })
        .unwrap_or(0);
    if older == 0 {
        b.to_string()
    } else {
        format!("{b}-{}", older + 1)
    }
}

/// The memory a word names, among these containers.
pub fn find_chunk(world: &World, word: &str, within: &[EntityId]) -> Option<EntityId> {
    let want = word.trim().to_lowercase();
    let want = want.trim_start_matches("the ").trim_end_matches(" memory");
    within.iter().find_map(|&c| {
        world
            .store
            .children(c)
            .filter(|e| e.kind == "chunk")
            .map(|e| e.id)
            .find(|&id| chunk_word(world, id) == want)
    })
}

/// Whether a bare word could be a memory's.
pub fn is_chunk_word(word: &str) -> bool {
    let w = word.to_lowercase();
    let stem = match w.rsplit_once('-') {
        Some((s, n)) if n.chars().all(|c| c.is_ascii_digit()) && !n.is_empty() => s,
        _ => w.as_str(),
    };
    CHUNK_WORDS.contains(&stem)
}

/// The other things of this one's name lying where it lies, itself among them,
/// by id.
fn item_twins(world: &World, item: EntityId) -> Vec<u64> {
    let name = super::craft::display(world, item);
    let mut v: Vec<u64> = world
        .store
        .entity(item)
        .and_then(|e| e.parent)
        .map(|p| {
            world
                .store
                .children(p)
                .filter(|e| e.kind == "item" && super::craft::display(world, e.id) == name)
                .map(|e| e.id.0)
                .collect()
        })
        .unwrap_or_default();
    v.sort_unstable();
    v
}

/// The word that tells a thing from another of its name beside it, if there is one.
pub fn item_tag(world: &World, item: EntityId) -> Option<&'static str> {
    let twins = item_twins(world, item);
    if twins.len() < 2 {
        return None;
    }
    let tags = super::mind::TAGS;
    let n = tags.len() as u64;
    let mut taken: Vec<u64> = Vec::new();
    for t in twins {
        let mut i = t % n;
        while taken.contains(&i) && (taken.len() as u64) < n {
            i = (i + 1) % n;
        }
        taken.push(i);
        if t == item.0 {
            return Some(tags[i as usize]);
        }
    }
    None
}

fn bare(name: &str) -> &str {
    name.trim_start_matches("a ")
        .trim_start_matches("an ")
        .trim_start_matches("the ")
}

/// A thing as it is typed: "lantern", or "dusty wisp ember" beside another ember.
pub fn item_handle(world: &World, item: EntityId) -> String {
    let name = super::craft::display(world, item);
    match item_tag(world, item) {
        Some(t) => format!("{t} {}", bare(&name)),
        None => bare(&name).to_string(),
    }
}

/// The thing a tagged name ("dusty wisp ember", or "dusty") names, in a container.
pub fn find_item(world: &World, arg: &str, within: EntityId) -> Option<EntityId> {
    let want = arg.trim().to_lowercase();
    let want = bare(&want);
    let (tag, rest) = match want.split_once(' ') {
        Some((t, r)) => (t, Some(r)),
        None => (want, None),
    };
    if !super::mind::TAGS.contains(&tag) {
        return None;
    }
    world
        .store
        .children(within)
        .filter(|e| e.kind == "item")
        .map(|e| e.id)
        .find(|&id| {
            item_tag(world, id) == Some(tag)
                && rest.map_or(true, |r| {
                    bare(&super::craft::display(world, id).to_lowercase()) == r
                })
        })
}

/// A line as a player is shown it: the ids of memories and things said as what
/// they are called.
pub fn humanize(world: &World, text: &str) -> String {
    the_lowered(&ids_said(world, text))
}

/// A room's name inside a sentence: "in the Fogbank", not "in The Fogbank". The
/// names are stored with their capital, and a sentence that opens with one keeps it.
pub fn the_lowered(text: &str) -> String {
    if !text.contains(" The ") {
        return text.to_string();
    }
    let mut out = text.to_string();
    for before in [
        "in", "to", "into", "at", "from", "of", "toward", "past", "through", "for", "and", "is",
    ] {
        out = out.replace(&format!(" {before} The "), &format!(" {before} the "));
    }
    out
}

/// A name with its first letter raised, to open a sentence.
pub fn capital(name: &str) -> String {
    let mut c = name.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Names as a sentence lists them: twins counted ("two stone rolls"), the last
/// joined with "and".
pub fn listed(names: &[&str]) -> String {
    let mut seen: Vec<(&str, usize)> = Vec::new();
    for n in names {
        match seen.iter_mut().find(|(s, _)| s == n) {
            Some(e) => e.1 += 1,
            None => seen.push((n, 1)),
        }
    }
    let said: Vec<String> = seen
        .iter()
        .map(|(n, k)| {
            if *k == 1 {
                return n.to_string();
            }
            let bare = n
                .strip_prefix("a ")
                .or_else(|| n.strip_prefix("an "))
                .unwrap_or(n);
            let count = match k {
                2 => "two".to_string(),
                3 => "three".to_string(),
                4 => "four".to_string(),
                k => k.to_string(),
            };
            format!(
                "{count} {bare}{}",
                if bare.ends_with('s') { "" } else { "s" }
            )
        })
        .collect();
    match said.len() {
        0 => String::new(),
        1 => said[0].clone(),
        n => format!("{} and {}", said[..n - 1].join(", "), said[n - 1]),
    }
}

fn ids_said(world: &World, text: &str) -> String {
    if !text.contains("chunk-") && !text.contains("item-") {
        return text.to_string();
    }
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < b.len() {
        let rest = &text[i..];
        let boundary = i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'-');
        let hit = if boundary && rest.starts_with("chunk-") {
            Some(("chunk", 6))
        } else if boundary && rest.starts_with("item-") {
            Some(("item", 5))
        } else {
            None
        };
        let Some((kind, skip)) = hit else {
            let ch = rest.chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
            continue;
        };
        let digits: String = rest[skip..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        let Ok(id) = digits.parse::<u64>() else {
            out.push_str(&rest[..skip]);
            i += skip;
            continue;
        };
        let after = &rest[skip + digits.len()..];
        i += skip + digits.len();
        let id = EntityId(id);
        if kind == "chunk" {
            let word = chunk_word(world, id);
            if after.starts_with(" worth") {
                out.push_str(&format!("a memory, {word},"));
            } else {
                out.push_str(&word);
            }
            continue;
        }
        let exists = world.store.entity(id).is_some_and(|e| e.kind == "item");
        let tag = if exists { item_tag(world, id) } else { None };
        // "a wisp ember (item-5, 46% remembered)": the number goes, and its
        // brackets with it when it was all they held.
        if out.ends_with('(') {
            match (tag, after.strip_prefix(", "), after.strip_prefix(')')) {
                (Some(t), _, _) => out.push_str(t),
                (None, Some(_), _) => i += 2,
                (None, None, Some(_)) => {
                    out.pop();
                    if out.ends_with(' ') {
                        out.pop();
                    }
                    i += 1;
                }
                _ => out.push_str("it"),
            }
            continue;
        }
        if exists {
            let h = item_handle(world, id);
            // After a colon or as an argument: the thing by its name.
            if h.contains(' ') && out.ends_with(' ') && !out.ends_with(": ") {
                out.push_str(&format!("\"{h}\""));
            } else {
                out.push_str(&h);
            }
        } else {
            out.push_str("the thing");
        }
    }
    out
}

/// The gate reads bare words that stand together as one argument, so `remember
/// amber wisp` arrives as "amber wisp" and `dig den amber` as "den amber". Where
/// a verb takes a memory, its word is taken apart from its neighbor again.
/// `bare[i]` says whether argument `i` was written without quotes.
pub fn unmerge(world: &World, verb: &str, args: Vec<Value>, bare_args: &[bool]) -> Vec<Value> {
    let verb = verb.to_lowercase();
    let Some(spec) = world
        .registry
        .specs()
        .find(|s| s.name.rsplit('.').next() == Some(verb.as_str()))
    else {
        return args;
    };
    let Some(p) = spec.params.iter().position(|x| x.name == "chunk") else {
        return args;
    };
    let mut args = args;
    let is_bare = |i: usize| bare_args.get(i).copied().unwrap_or(false);
    let words = |v: &Value| -> Vec<String> {
        v.as_str()
            .map(|s| s.split_whitespace().map(str::to_string).collect())
            .unwrap_or_default()
    };
    // The memory first, then what it is for: "amber wisp".
    if let Some(v) = args.get(p).filter(|_| is_bare(p)) {
        let w = words(v);
        if w.len() > 1 && is_chunk_word(&w[0]) {
            args[p] = Value::String(w[0].clone());
            args.insert(p + 1, Value::String(w[1..].join(" ")));
            return args;
        }
    }
    // What it is for first, then the memory: "den amber".
    if p > 0 {
        if let Some(v) = args.get(p - 1).filter(|_| is_bare(p - 1)) {
            let w = words(v);
            if w.len() > 1 && is_chunk_word(&w[w.len() - 1]) {
                args[p - 1] = Value::String(w[..w.len() - 1].join(" "));
                args.insert(p, Value::String(w[w.len() - 1].clone()));
            }
        }
    }
    args
}

/// A stretch of a reply the page folds away behind "more": written between these
/// two, which print as nothing anywhere else (the gate takes them out). The words
/// are all still there, in their order, for whoever reads the reply whole.
pub const FOLD: char = '\u{2063}';
/// The end of a folded stretch.
pub const UNFOLD: char = '\u{2064}';

/// Close the fold opened at `at` (where [`FOLD`] was written). A stretch too short
/// to be worth a button is left showing.
pub fn fold_close(text: &mut String, at: usize) {
    let n = FOLD.len_utf8();
    if text.len() < at + n || !text[at..].starts_with(FOLD) {
        return;
    }
    if text[at + n..].trim().len() < 140 {
        text.replace_range(at..at + n, "");
    } else {
        text.push(UNFOLD);
    }
}

/// `text` with one stretch folded, where it is long enough to be worth it.
pub fn folded(text: &str) -> String {
    let mut out = String::from(FOLD);
    out.push_str(text);
    fold_close(&mut out, 0);
    out
}

/// A reply with its folds taken out, for a reader that has no page.
pub fn unfolded(text: &str) -> String {
    text.replace([FOLD, UNFOLD], "")
}

/// A description's first sentence and the rest of it (with its leading space).
/// The rest is empty when there is one sentence, or no clean place to cut.
pub fn first_sentence(desc: &str) -> (&str, &str) {
    let b = desc.as_bytes();
    let mut quotes = 0;
    for i in 0..b.len().saturating_sub(2) {
        if b[i] == b'"' {
            quotes += 1;
        }
        if i >= 30
            && matches!(b[i], b'.' | b'!' | b'?')
            && b[i + 1] == b' '
            && b[i + 2].is_ascii_uppercase()
            && quotes % 2 == 0
        {
            return (&desc[..=i], &desc[i + 1..]);
        }
    }
    (desc, "")
}
