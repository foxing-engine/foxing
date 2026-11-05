// tiles: the room, drawn. The default scene of the play page and the reference for the
// mod API (README.md beside this file). It subscribes to the room you stand in and
// draws it on a canvas: the floor and walls in the room's tint with the motifs its
// words call for (water, shelves, benches, lanterns, fog, trees, stone), the doors
// and hatches, what lies on the floor, the creatures, the citizens, the Docent, you.
// Everything in the picture is a thing to click: a click opens a small menu of what
// you can do with it (walk, take, talk, tell a story, give a memory, attack, ask), so
// a person who has never typed a verb can play by pointing; a click on the floor by a
// door walks through it, and a drag on the picture shows what you remember of the map. Blows float off the
// sprites, memories earned float up, speech sits in bubbles, and the lane's numbers
// are bars at the foot of the picture. Typing still works for everything.
//
// Loaded by default; `?mod=none` is the plain terminal.

const COLS = 20, ROWS = 11, T = 16;
const W = COLS * T, H = ROWS * T;
const WALL_TOP = 2; // the north wall is two tiles tall: shelves, windows, a door
const DOORS = { north: [Math.floor(COLS / 2), 1], south: [Math.floor(COLS / 2), ROWS - 1], west: [0, 6], east: [COLS - 1, 6] };
const NIGHT = "#121a25", INK = "#ece5d3", DUST = "#d9b57a", EMBER = "#d1734f", MOSS = "#7fb37f";
// Where things stand: you in the middle, the rest on a ring of slots, by id.
const SLOTS = [[7, 5], [12, 5], [6, 8], [13, 8], [9, 4], [10, 8], [4, 6], [15, 6], [7, 7], [12, 4], [9, 9], [10, 3], [5, 4], [14, 4], [4, 8], [15, 8], [8, 3], [11, 9]];
// Named doors: along the south wall either side of the door; past ARCHES_SHOWN they
// go behind the signpost by the west wall.
const ARCHES = [[8, ROWS - 1], [12, ROWS - 1], [6, ROWS - 1], [14, ROWS - 1], [4, ROWS - 1], [16, ROWS - 1], [2, ROWS - 1], [18, ROWS - 1]];
const ARCHES_SHOWN = 6;
const SIGNPOST = [2, 4];
function isStairs(via) { return via === "up" || via === "down" || /stair|ladder|climb|dive|surface|chute|well/.test(via); }
const ITEM_SLOTS = [[8, 7], [11, 6], [6, 6], [13, 7], [9, 8], [10, 5], [5, 7], [14, 5], [7, 9], [12, 9]];
// The Docent's round: it walks the room's edge, pausing at each corner of its way, on
// the clock (every window sees it in the same place at the same moment).
const ROUND = [[COLS - 5, 3], [COLS - 3, 5], [COLS - 4, 8], [3, 8], [2, 5], [4, 3]];
const ROUND_WALK_MS = 6000, ROUND_PAUSE_MS = 5000;
function docentAt(now) {
  const leg = ROUND_WALK_MS + ROUND_PAUSE_MS, i = Math.floor(now / leg) % ROUND.length;
  const [ax, ay] = ROUND[i], [bx, by] = ROUND[(i + 1) % ROUND.length];
  const t = Math.max(0, Math.min(1, ((now % leg) - ROUND_PAUSE_MS) / ROUND_WALK_MS));
  const k = t * t * (3 - 2 * t); // ease: it sets off and arrives gently
  return [ax + (bx - ax) * k, ay + (by - ay) * k, t > 0 && t < 1];
}
const VERB_HALF_LIFE = 15 * 60 * 1000, VERB_GONE = 1 / 12;
const STARTERS = ["look", "map", "chunks"];
const NOARG = new Set(["look", "map", "chunks", "board", "satchel", "recipes", "time", "rest", "retreat", "palace", "log", "attack", "leave", "keep", "resume", "pto", "statutes", "wants", "mine", "session", "messages", "petitions", "read"]);
const PREFILL = { say: 'say "', tell: "tell ", story: "story ", travel: "travel ", take: "take ", drop: "drop ", craft: "craft ", give: "give ", remember: "remember ", feedback: 'feedback "', greet: 'greet "', dig: "dig ", describe: 'describe "', append: 'append "', name: 'name "', shelve: "shelve ", reminisce: "reminisce ", petition: 'petition "' };

// ---- more frames: the kinds the page's atlas lacks, in the same 16x16 form ----
const EXTRA = {
  "lamprey": { p: { a: "#5c7a6a", b: "#2f4a3d", e: "#f0e0a0", t: "#e8e8e8", m: "#3a1f28" }, r: ["................", "................", "......aaaa......", "....aabbbbaa....", "...abbemebba....", "...abbtttbba....", "....abbmbbaa....", ".....aabbbbaa...", "......aabbbba...", "........aabbba..", ".........aabba..", "..........abba..", "..........abb...", ".........abb....", "........aab.....", "................"] },
  "usher": { p: { c: "#8b1f2d", d: "#5a1420", f: "#e9d5b8", e: "#1a1612", g: "#d9b57a", w: "#f2f2f2" }, r: ["................", "......gggg......", ".....gccccg.....", "....ccffffcc....", "....cfefefcc....", "....ccffffcc....", ".....cwwwc......", "....cccwccc.....", "...ccccwcccc....", "...cdcccccdc....", "...cdcccccdc....", "...cddcccddc....", "....ddcccdd.....", "....dd...dd.....", "....ee...ee.....", "................"] },
  "censor": { p: { c: "#2d2d3a", d: "#17171f", f: "#c9c0b0", e: "#ff5a5a", r: "#ffb3b3", g: "#8a8aa0" }, r: ["................", "......cccc......", ".....cccccc.....", "....ccffffcc....", "....cfeffecc....", "....ccffffcc....", ".....ccrrcc.....", "....ccccccc.....", "...ccgccccgcc...", "...cdcccccccdc..", "...cdcccccccdc..", "...cddcccccddc..", "....dddcccddd...", "....dd.....dd...", "....dd.....dd...", "................"] },
  "cataloguer": { p: { c: "#4a3f5c", d: "#2b2436", f: "#d8ccb8", e: "#1a1612", g: "#d9b57a", p: "#ece5d3" }, r: ["................", "......gggg......", ".....gccccg.....", "....ccffffcc....", "....cfefefcc....", "....ccffffcc....", ".....cccccc.....", "...ppccccccpp...", "...ppccccccpp...", "...ppdcccccdpp..", "...cdccccccccd..", "...cddcccccddc..", "....ddcccccdd...", "....dd.....dd...", "....ee.....ee...", "................"] },
  "engrosser": { p: { c: "#3b3a2e", d: "#24231c", f: "#e4d8c4", e: "#1a1612", q: "#ece5d3", k: "#1a1612" }, r: ["................", "......cccc......", ".....cccccc.....", "....ccffffcc....", "....cfefefcc....", "....ccffffcc....", ".....cccccc.....", "....ccccccc.q...", "...cccccccccq...", "...cdcccccckq...", "...cdccccccqq...", "...cddcccddc....", "....ddcccdd.....", "....dd...dd.....", "....ee...ee.....", "................"] },
  "tapster": { p: { c: "#5a3a2a", d: "#3a2418", f: "#e9d5b8", e: "#1a1612", a: "#ece5d3", g: "#9fd3e6" }, r: ["................", "......cccc......", ".....cffffc.....", "....cfefefcc....", "....cffffffc....", ".....cffffc.....", "......cccc......", "....ccaaaacc....", "...ccaaaaaacc...", "...cdaaaaaadc...", "...cdaaaaaadcg..", "...cddaaaaddcg..", "....ddccccdd.g..", "....dd....dd....", "....ee....ee....", "................"] },
  "clipper": { p: { c: "#6a747d", d: "#3f474e", e: "#f0e0a0", s: "#d8dde2" }, r: ["................", "................", "................", "......cccc......", ".....ceccec.....", ".....cccccc.....", "....ccccccccs...", "...cdccccccds...", "...cdccccccd.s..", "...cdddddddd..s.", "....dd....dd.s..", "....dd....dd.s..", "................", "................", "................", "................"] },
  "sweater": { p: { c: "#7a6a4a", d: "#4a3e2a", f: "#d8c8a8", e: "#1a1612", b: "#5a3a2a", g: "#ffcf7a" }, r: ["................", "......cccc......", ".....cffffc.....", "....cfefefcc....", "....cffffffc....", ".....cffffc.....", "......cccc......", "....ccccccccbb..", "...cccccccccbbb.", "...cdcccccccbgb.", "...cdccccccdbbb.", "...cddcccccdd...", "....ddcccccdd...", "....dd....dd....", "....ee....ee....", "................"] },
  "assayer": { p: { c: "#8a8a7a", d: "#5a5a4a", f: "#e4d8c4", e: "#1a1612", k: "#2a2a3a", g: "#cfe8ee" }, r: ["......kkkk......", ".....kkkkkk.....", "....ccffffcc....", "....cfefefcc....", "....ccffffcc....", ".....cccccc.....", "....cccccccc.gg.", "...ccccccccc.gg.", "...cdcccccccdgg.", "...cdcccccccd...", "...cddcccccddc..", "....dddcccddd...", "....dd.....dd...", "....dd.....dd...", "....ee.....ee...", "................"] },
  "warden": { p: { c: "#8a2a2a", d: "#4a1414", f: "#e9d5b8", e: "#1a1612", m: "#3a3a44", s: "#b0b0b8" }, r: [".............s..", "....mmmmmm...s..", "...mmmmmmmm..s..", "...mmffffmm..s..", "...mmfefefm..s..", "...mmffffmm..s..", "....mmmmmm...s..", "..cccccccccc.s..", ".cccccccccccss..", ".cdccccccccdcs..", ".cdccccccccdcs..", ".cddccccccddc...", "..dddccccddd....", "...dd....dd.....", "...ee....ee.....", "................"] },
  "tipstaff": { p: { c: "#1f2a3a", d: "#121a25", f: "#e9d5b8", e: "#1a1612", g: "#d9b57a", s: "#b08a3a" }, r: ["..........s.....", "......cccc.s....", ".....cccccc.s...", "....ccffffccs...", "....cfefefccs...", "....ccffffccs...", ".....ccccccgs...", "....ccccccccs...", "...cccccccccs...", "...cdcccccdcs...", "...cdcccccdc....", "...cddcccddc....", "....ddcccdd.....", "....dd...dd.....", "....ee...ee.....", "................"] },
};
// What lies on a floor, by the word in its name: 8x8 icons in two colors.
const ICONS = {
  water: { p: { a: "#9fd3e6", b: "#4a7f96" }, r: ["........", "..aaaa..", ".abbbba.", ".abaaba.", ".abbbba.", "..aaaa..", "........", "........"] },
  dust: { p: { a: "#bdb7a6", b: "#8e887c" }, r: ["........", "........", "..a..b..", ".b.a..a.", "..a.b.b.", ".a..a...", "........", "........"] },
  ink: { p: { a: "#2a2f5a", b: "#5a62a8" }, r: ["........", "........", "...ab...", "..aabb..", "..aabb..", "...ab...", "........", "........"] },
  ember: { p: { a: "#d1734f", b: "#ffcf7a" }, r: ["........", "....b...", "...ab...", "..aabb..", "..aaab..", "...aa...", "........", "........"] },
  thread: { p: { a: "#c9a24f", b: "#8a6a2f" }, r: ["........", ".a......", "..ab....", "...ab...", "....ab..", ".....ab.", "......a.", "........"] },
  key: { p: { a: "#d9b57a", b: "#8a6a2f" }, r: ["........", "..aa....", ".a..a...", "..aa.a..", "....a.a.", ".....a.a", "......a.", "........"] },
  charm: { p: { a: "#e8e4da", b: "#9a8fb0" }, r: ["........", "..b..b..", ".aba.ab.", ".aaaaaa.", "..aaaa..", "...aa...", "........", "........"] },
  lantern: { p: { a: "#d9b57a", b: "#ffcf7a" }, r: ["...a....", "..aaa...", "..aba...", "..abba..", "..abba..", "..aaa...", "...a....", "........"] },
  book: { p: { a: "#7a4a3a", b: "#ece5d3" }, r: ["........", ".aaaaaa.", ".abbbba.", ".abbbba.", ".abbbba.", ".aaaaaa.", "........", "........"] },
  bandage: { p: { a: "#ece5d3", b: "#c9c0b0" }, r: ["........", "..aaaa..", ".abbbba.", ".abbbba.", "..aaaa..", "........", "........", "........"] },
  wing: { p: { a: "#c9bfa8", b: "#8a8070" }, r: ["........", ".a....a.", ".ab..ba.", "..abba..", "..abba..", "...aa...", "........", "........"] },
  lamp: { p: { a: "#8a6a2f", b: "#ffe6a0" }, r: ["...bb...", "..bbbb..", "..bbbb..", "...aa...", "...aa...", "..aaaa..", ".aaaaaa.", "........"] },
  bell: { p: { a: "#b08a3a", b: "#d9b57a" }, r: ["...a....", "..aba...", ".abbba..", ".abbba..", "abbbbba.", "aaaaaaa.", "...a....", "........"] },
  basin: { p: { a: "#8e887c", b: "#9fd3e6" }, r: ["........", "........", "aaaaaaaa", ".abbbba.", ".abbbba.", "..aaaa..", "...aa...", "..aaaa.."] },
  plaque: { p: { a: "#5a4a3a", b: "#d9b57a" }, r: ["........", ".aaaaaa.", ".abbbba.", ".abbbba.", ".abbbba.", ".aaaaaa.", "........", "........"] },
  thing: { p: { a: "#bdb7a6", b: "#7a6f5f" }, r: ["........", "..aaaa..", ".abbbba.", ".abbbba.", ".aaaaaa.", "..aaaa..", "........", "........"] },
};
function iconFor(name) {
  const n = (name || "").toLowerCase();
  for (const k of ["water", "dust", "ink", "ember", "thread", "key", "charm", "lantern", "basin", "bell", "lamp", "plaque", "book", "bandage", "wing"]) if (n.includes(k)) return k;
  if (/ledger|volume|minutes|page|register/.test(n)) return "book";
  return "thing";
}
// The room's motifs, from its words.
const MOTIFS = [
  ["mint", /exergue|counting bar|die room|striking floor|assay|mintmark|purse room|mint\b|coin/],
  ["water", /water|cistern|shallow|undertow|drown|sluice|flood|moonpool|stillwater|tide|pool|choir|inkwell|lamprey/],
  ["shelves", /archive|stack|reading|catalogue|catalog|shelf|shelves|ledger|library|book|carrel|periodical|desk|minute|engross|lectern|robing|annex/],
  ["benches", /conclave|bench|gallery|sitting|bar of|division|roll of|tally|petition/],
  ["garden", /orchard|tree|moss|garden|bramble|hedge|windfall|nook|parlor|parlour|tea|chapel|perch|landing|grove|meadow/],
  ["fog", /fog|mist|moor|cloud|smoke|midden|ossuary|bone|grave/],
  ["lanterns", /lantern|lamp|wick|candle|belfry|clock|orrery|pendulum|escapement|mainspring/],
  ["stone", /stair|threshold|burrow|oubliette|attic|office|stone|closet|chute|lobby|room/],
];
function motifOf(key, name, description) {
  const text = ((key || "") + " " + (name || "") + " " + (description || "")).toLowerCase();
  // The name and key first (strong), then the description (weak): a water room's prose
  // may mention a lantern, but its name says water.
  const head = ((key || "") + " " + (name || "")).toLowerCase();
  for (const [m, re] of MOTIFS) if (re.test(head)) return m;
  for (const [m, re] of MOTIFS) if (re.test(text)) return m;
  return "stone";
}

export default function install(m) {
  // ---- the panel: the picture, the hud line, the menu, the verbs used lately ----
  const panel = m.panel("Scene", { id: "scene", w: 760 });
  const wrap = document.createElement("div");
  wrap.style.cssText = "position:relative;width:100%";
  const canvas = document.createElement("canvas");
  canvas.setAttribute("role", "img");
  canvas.setAttribute("aria-label", "The room, drawn; click a thing in it to act");
  canvas.style.cssText = "width:100%;height:auto;display:block;image-rendering:pixelated;image-rendering:crisp-edges;border-radius:6px";
  const menu = document.createElement("div");
  menu.hidden = true;
  menu.style.cssText = "position:absolute;z-index:5;min-width:150px;background:rgba(18,26,37,.96);border:1px solid var(--line,#334);border-radius:8px;padding:6px;box-shadow:0 8px 24px rgba(0,0,0,.45);font-size:13px";
  const hud = document.createElement("div");
  hud.style.cssText = "font-size:12px;color:var(--dim);margin-top:6px;min-height:1.2em;line-height:1.3";
  hud.textContent = "Log in to see the room.";
  const verbs = document.createElement("div");
  verbs.style.cssText = "display:flex;flex-wrap:wrap;gap:4px;margin-top:6px";
  verbs.setAttribute("aria-label", "Verbs you have used lately");
  wrap.append(canvas, menu);
  panel.append(wrap, hud, verbs);
  m.builtin("map", false);
  m.builtin("here", false);
  const ctx = canvas.getContext("2d");

  // ---- sprites cache (filled below); cleared when the scale changes ----
  const cache = new Map();
  function cacheClear() { cache.clear(); }

  // ---- scale: the canvas fills the panel; its pixels stay whole ----
  let SCALE = 3;
  function fit() {
    // The picture fits the scene area's height too: a canvas taller than the area
    // was cut at the south door and the bars.
    const area = panel.parentElement;
    const availH = area ? Math.max(160, area.clientHeight - 56) : 9999;
    const maxW = Math.floor(availH * W / H);
    const pw = panel.clientWidth ? panel.clientWidth - 12 : 640;
    canvas.style.width = Math.min(pw, maxW) + "px";
    wrap.style.width = canvas.style.width;
    const w = Math.max(240, wrap.clientWidth || 640);
    const s = Math.max(1, Math.min(4, Math.floor(w / W)));
    const dpr = Math.min(2, window.devicePixelRatio || 1);
    const scale = Math.max(1, Math.min(4, Math.round(s * dpr)));
    if (scale !== SCALE || canvas.width !== W * scale) { SCALE = scale; canvas.width = W * SCALE; canvas.height = H * SCALE; cacheClear(); }
    ctx.imageSmoothingEnabled = false;
  }
  const ro = typeof ResizeObserver === "function" ? new ResizeObserver(fit) : null;
  if (ro) { ro.observe(panel); if (panel.parentElement) ro.observe(panel.parentElement); } else window.addEventListener("resize", fit);
  // The scene area opens a little taller for the picture, unless the person has set it.
  try { if (!localStorage.getItem("disremembered.sceneH")) document.getElementById("main").style.setProperty("--scene-h", "56vh"); } catch (e) {}
  fit();

  // ---- the verbs used lately, fading like everything ----
  let used = new Map();
  try { const raw = localStorage.getItem("tiles-verbs"); if (raw) used = new Map(Object.entries(JSON.parse(raw))); } catch (e) {}
  function saveVerbs() { try { localStorage.setItem("tiles-verbs", JSON.stringify(Object.fromEntries(used))); } catch (e) {} }
  function touch(verb) { if (!verb) return; used.set(verb, Date.now()); saveVerbs(); drawVerbs(); }
  function drawVerbs() {
    const now = Date.now();
    const rows = [];
    for (const [verb, at] of used) {
      const mem = Math.pow(0.5, Math.max(0, now - at) / VERB_HALF_LIFE);
      if (mem < VERB_GONE) { used.delete(verb); continue; }
      rows.push([verb, mem]);
    }
    rows.sort((a, b) => b[1] - a[1]);
    const frag = document.createDocumentFragment();
    for (const [verb, mem] of rows.slice(0, 10)) {
      const b = document.createElement("button"); b.type = "button"; b.className = "way";
      b.textContent = verb; b.style.opacity = Math.max(0.35, mem).toFixed(2);
      b.title = (NOARG.has(verb) ? "run " + verb : "start a " + verb + " line") + "; fades unless used";
      b.addEventListener("click", () => { touch(verb); if (NOARG.has(verb)) m.command(verb, []); else m.prefill(PREFILL[verb] || (verb + " ")); });
      frag.append(b);
    }
    verbs.replaceChildren(frag);
    verbs.hidden = rows.length === 0;
  }
  const verbTimer = setInterval(drawVerbs, 30000);
  drawVerbs();

  // ---- sprites: the atlas rows (and the extra frames) rasterized once per frame ----
  function frameData(frame) { return m.atlas[frame] || EXTRA[frame] || null; }
  const ROOM_FRAMES = new Set(["threshold", "fogbank", "shallows", "ossuary", "dug", "lantern-row", "archive", "stairwell", "cistern", "undertow", "orchard", "clocktower", "midden", "gallery", "oubliette", "burrow", "parlor", "chapel", "nook", "attic", "office", "chunk"]);
  function isRoomFrame(frame) { return ROOM_FRAMES.has(frame) || /^dug/.test(frame); }
  function frameCanvas(frame, tint) {
    const f = frameData(frame); if (!f) return null;
    const key = frame + "|" + (tint || "");
    if (cache.has(key)) return cache.get(key);
    const c = document.createElement("canvas"); c.width = T; c.height = T;
    const g = c.getContext("2d");
    // A one-pixel outline in the night behind every sprite, so it reads against any
    // floor; then the frame, then the tint for a hybrid.
    const solid = (x, y) => y >= 0 && y < T && x >= 0 && x < T && f.r[y][x] && f.r[y][x] !== ".";
    if (!frame.endsWith("-room") && !isRoomFrame(frame)) {
      g.fillStyle = "rgba(10,14,20,.85)";
      for (let y = 0; y < T; y++) for (let x = 0; x < T; x++) if (!solid(x, y) && (solid(x - 1, y) || solid(x + 1, y) || solid(x, y - 1) || solid(x, y + 1))) g.fillRect(x, y, 1, 1);
    }
    f.r.forEach((row, y) => { for (let x = 0; x < T; x++) { const ch = row[x]; if (ch && ch !== ".") { g.fillStyle = f.p[ch] || "#f0f"; g.fillRect(x, y, 1, 1); } } });
    if (tint) { g.globalCompositeOperation = "source-atop"; g.fillStyle = tint; g.globalAlpha = 0.38; g.fillRect(0, 0, T, T); g.globalAlpha = 1; g.globalCompositeOperation = "source-over"; }
    cache.set(key, c);
    return c;
  }
  function iconCanvas(kind) {
    const f = ICONS[kind] || ICONS.thing;
    const key = "icon|" + kind;
    if (cache.has(key)) return cache.get(key);
    const c = document.createElement("canvas"); c.width = 8; c.height = 8;
    const g = c.getContext("2d");
    f.r.forEach((row, y) => { for (let x = 0; x < 8; x++) { const ch = row[x]; if (ch !== ".") { g.fillStyle = f.p[ch]; g.fillRect(x, y, 1, 1); } } });
    cache.set(key, c);
    return c;
  }
  // A sprite's own tint, when it is a hybrid drawn in a parent's frame: the frame's
  // first color is the kind's; a different tint on the component is a mix.
  function spriteTint(sprite) {
    const f = sprite && frameData(sprite.frame); if (!f || !sprite.tint) return null;
    const own = Object.values(f.p)[0];
    if (sprite.frame === "citizen") return sprite.tint.startsWith("hsl") ? sprite.tint : null;
    return own && own.toLowerCase() !== sprite.tint.toLowerCase() ? sprite.tint : null;
  }
  // `pose`: "breath" is the second idle frame (the top ten rows a pixel lower, the
  // feet where they were); "shaken" lies tilted and low, trembling.
  function drawSprite(sprite, x, y, fallbackText, alpha, pose) {
    const img = sprite && (sprite.atlas === m.atlasName || EXTRA[sprite.frame]) ? frameCanvas(sprite.frame, spriteTint(sprite)) : null;
    if (alpha != null) ctx.globalAlpha = alpha;
    if (img && pose === "shaken") {
      const tremor = (Math.floor(performance.now() / 140) % 3) - 1;
      ctx.save();
      ctx.translate((x + 8 + tremor * 0.5) * SCALE, (y + 17) * SCALE); ctx.rotate(-0.26);
      ctx.drawImage(img, 0, 0, T, T, -8 * SCALE, -15 * SCALE, T * SCALE, T * SCALE);
      ctx.restore();
    } else if (img && pose === "breath") {
      ctx.drawImage(img, 0, 0, T, 10, Math.round(x * SCALE), Math.round((y + 1) * SCALE), T * SCALE, 10 * SCALE);
      ctx.drawImage(img, 0, 10, T, 6, Math.round(x * SCALE), Math.round((y + 10) * SCALE), T * SCALE, 6 * SCALE);
    } else if (img) ctx.drawImage(img, 0, 0, T, T, Math.round(x * SCALE), Math.round(y * SCALE), T * SCALE, T * SCALE);
    else {
      ctx.fillStyle = (sprite && sprite.tint) || "#d9d4c7";
      ctx.font = "bold " + (10 * SCALE) + "px monospace"; ctx.textAlign = "center"; ctx.textBaseline = "middle";
      ctx.fillText((sprite && sprite.glyph) || fallbackText || "?", (x + 8) * SCALE, (y + 8) * SCALE);
    }
    ctx.globalAlpha = 1;
  }
  function px(x, y, w, h, color) { ctx.fillStyle = color; ctx.fillRect(Math.round(x * SCALE), Math.round(y * SCALE), Math.round(w * SCALE), Math.round(h * SCALE)); }

  // ---- colors ----
  function hex(c) { const n = parseInt((c || "#666666").slice(1), 16); return [n >> 16 & 255, n >> 8 & 255, n & 255]; }
  function shade([r, g, b], k, mix = 0) { const t = (v) => Math.round(Math.max(0, Math.min(255, v * k * (1 - mix) + 236 * mix))); return "rgb(" + t(r) + "," + t(g) + "," + t(b) + ")"; }
  function hash(s) { let h = 2166136261; for (const ch of String(s)) { h ^= ch.charCodeAt(0); h = Math.imul(h, 16777619); } return h >>> 0; }
  function rnd(seed, i) { return (hash(seed + ":" + i) % 1000) / 1000; }

  // ---- the scene as last observed ----
  let scene = null;
  let roomId = null, fadeAt = 0;
  const hot = new Map();      // "x,y" -> {label, menu: [{label, run}], primary}
  const floats = [];          // {text, x, y, t0, color, big}
  const flashes = new Map();  // thing id -> until
  const bubbles = new Map();  // who (lowercase) -> {text, until}
  let hurt = 0;               // the edge glows red for a moment when you are hit
  let lastHp = null;

  function layout(obs) {
    const root = obs && obs.root; if (!root) return null;
    const room = root.components["disremembered.room"] || {};
    const sprite = root.components["disremembered.sprite"] || {};
    const exits = [], more = [];
    let hatchX = 0, side = 0;
    // Named doors beyond the first few go behind a signpost: the Threshold has
    // fifteen exits, and fifteen arches read as a wall of labels. The ones drawn are
    // the world's own rooms first, then yours, then the rest as the room lists them.
    const all = ((root.relations && root.relations.exits) || []).map(ex => {
      const rc = ex.components["disremembered.room"] || {};
      return { via: ex.via, name: rc.name || ex.via, dug: !!ex.owner, mine: ex.owner === m.actor };
    });
    const named = all.filter(ex => !DOORS[ex.via] && !isStairs(ex.via));
    const rank = (ex) => (ex.dug ? (ex.mine ? 1 : 2) : 0);
    const drawn = new Set(named.slice().sort((a, b) => rank(a) - rank(b)).slice(0, ARCHES_SHOWN).map(ex => ex.via));
    for (const ex of all) {
      if (DOORS[ex.via]) exits.push({ via: ex.via, name: ex.name, tile: DOORS[ex.via], kind: "door", dug: ex.dug });
      else if (isStairs(ex.via)) {
        // Stairs up along the north wall's foot, down along the south.
        const upish = ex.via === "up" || /ladder|climb|surface/.test(ex.via);
        exits.push({ via: ex.via, name: ex.name, tile: [upish ? 2 + (side++ % 2) * (COLS - 5) : COLS - 3 - (side++ % 2) * (COLS - 5), upish ? 2 : ROWS - 2], kind: upish ? "stairs-up" : "stairs-down", dug: ex.dug });
      } else if (drawn.has(ex.via)) {
        exits.push({ via: ex.via, name: ex.name, tile: ARCHES[hatchX % ARCHES.length], kind: "arch", dug: ex.dug });
        hatchX++;
      } else more.push(ex);
    }
    if (more.length) exits.push({ via: "", name: more.length + " more doors", tile: SIGNPOST, kind: "signpost", more });
    const things = [], items = [];
    let slot = 0, islot = 0;
    for (const n of (root.relations && root.relations.contents) || []) {
      const s = n.components["disremembered.sprite"] || {};
      if (n.kind === "creature") {
        const c = n.components["disremembered.creature"] || {};
        const mind = n.components["disremembered.mind"];
        const given = mind && mind.name ? mind.name : null;
        const shaken = mind && mind.shaken_until && m.timed ? (typeof mind.shaken_until === "number" ? mind.shaken_until > Date.now() : false) : false;
        things.push({ id: n.id, kind: "creature", name: c.name, given, hp: c.hp, max: c.max_hp, holding: c.xp, shaken, sprite: s, tile: SLOTS[(hash(n.id) + slot++) % SLOTS.length] });
      } else if (n.kind === "citizen") {
        const name = (n.components["disremembered.citizen"] || {}).name || "?";
        const p = n.components["disremembered.presence"];
        if (p && p.here === false) {
          const trace = m.timed(p.trace);
          if (trace >= 0.15) things.push({ id: n.id, kind: "ghost", name, sprite: s, trace, tile: SLOTS[(hash(n.id) + slot++) % SLOTS.length] });
        } else if (name === m.actor) things.push({ id: n.id, kind: "you", name, sprite: s, tile: [Math.floor(COLS / 2), 6] });
        else things.push({ id: n.id, kind: "citizen", name, sprite: Object.assign({}, s, { tint: "hsl(" + (hash(name) % 360) + ",45%,60%)", frame: s.frame || "citizen", atlas: s.atlas || m.atlasName }), tile: SLOTS[(hash(n.id) + slot++) % SLOTS.length] });
      } else if (n.kind === "docent") {
        things.push({ id: n.id, kind: "docent", name: (n.components["disremembered.docent"] || {}).name || "the Docent", sprite: s, tile: [COLS - 5, 3] });
      } else if (n.kind === "morphos") {
        things.push({ id: n.id, kind: "morphos", name: "Morphos", sprite: Object.assign({}, s, { frame: s.frame || "docent", atlas: s.atlas || m.atlasName }), tile: [4, 3] });
      } else if (n.kind === "item") {
        const it = n.components["disremembered.item"] || {};
        const label = it.given_name || it.name || "something";
        // The icon by the thing's kind (a lamp, a bell, a basin, a book), not its given
        // name: a book called The Fifth Ragpicker drew as a box.
        const icon = iconFor(it.name || label);
        if (it.fixed) things.push({ id: n.id, kind: "fixture", name: label, tile: [3 + (islot % 3) * 2, 3], icon });
        else items.push({ id: n.id, kind: "item", name: label, take: it.name || label, tile: ITEM_SLOTS[(hash(n.id) + islot) % ITEM_SLOTS.length], icon });
        islot++;
      }
    }
    // Two things on one tile: nudge the later one.
    const taken = new Set([Math.floor(COLS / 2) + ",6"]);
    for (const ex of exits) taken.add(ex.tile.join(","));
    for (const t of things.concat(items)) {
      if (t.kind === "you") continue;
      let i = 0;
      const pool = t.kind === "item" ? ITEM_SLOTS : SLOTS;
      while (taken.has(t.tile.join(",")) && i < pool.length) t.tile = pool[(hash(t.id) + ++i) % pool.length];
      taken.add(t.tile.join(","));
    }
    return {
      id: root.id, key: room.key, name: room.name || "?", memory: room.memory, description: room.description,
      bound: !!(room.bound && typeof room.bound === "object"), owner: root.owner || null,
      tint: sprite.tint || "#6b6b7a", sprite, exits, things, items,
      motif: motifOf(room.key, room.name, room.description),
    };
  }

  // ---- drawing the room ----
  function drawRoom(now) {
    const base = hex(scene.tint);
    const seed = scene.id;
    const motif = scene.motif;
    // Floor: two-tone checker with grain.
    for (let y = WALL_TOP; y < ROWS; y++) for (let x = 0; x < COLS; x++) {
      const n = rnd(seed, x * 31 + y * 17);
      const check = (x + y) % 2 === 0 ? 0.58 : 0.53;
      px(x * T, y * T, T, T, shade(base, check + n * 0.08));
      if (n > 0.72) px(x * T + 5 + Math.floor(n * 5), y * T + 9, 3, 1, shade(base, 0.72));
      if (n < 0.08) px(x * T + 3, y * T + 4, 1, 1, shade(base, 0.42));
    }
    // The north wall: two tiles tall, darker, with a baseboard line.
    for (let x = 0; x < COLS; x++) {
      const n = rnd(seed, 900 + x);
      px(x * T, 0, T, WALL_TOP * T, shade(base, 0.3 + n * 0.05));
      px(x * T, WALL_TOP * T - 2, T, 2, shade(base, 0.22));
      // Brickwork hint.
      px(x * T + ((x % 2) ? 8 : 0), 7, 8, 1, shade(base, 0.26));
      px(x * T, 16, T, 1, shade(base, 0.26));
    }
    // Side and south walls as a dark rim.
    px(0, WALL_TOP * T, 4, (ROWS - WALL_TOP) * T, shade(base, 0.3));
    px(W - 4, WALL_TOP * T, 4, (ROWS - WALL_TOP) * T, shade(base, 0.3));
    px(0, H - 4, W, 4, shade(base, 0.3));
    // Motifs on the wall and the floor.
    drawMotif(motif, base, seed, now);
    drawKeyProps(base, seed, now);
    drawWordProps(base, seed, now);
  }
  // What a room's key calls for beyond its motif: the house's furniture, by room.
  function drawKeyProps(base, seed, now) {
    const k = scene.key || "";
    const t = now / 1000;
    const gear = (cx, cy, r, teeth, speed, color) => {
      ctx.fillStyle = color; ctx.beginPath(); ctx.arc(cx * SCALE, cy * SCALE, r * SCALE, 0, Math.PI * 2); ctx.fill();
      ctx.fillStyle = NIGHT; ctx.beginPath(); ctx.arc(cx * SCALE, cy * SCALE, r * 0.35 * SCALE, 0, Math.PI * 2); ctx.fill();
      for (let i = 0; i < teeth; i++) { const a = t * speed + i * Math.PI * 2 / teeth; px(cx + Math.cos(a) * r - 1, cy + Math.sin(a) * r - 1, 2.5, 2.5, color); }
    };
    if (/^(clocktower|pendulum-room|escapement-stair|mainspring-closet|belfry)/.test(k)) {
      // The works: two gears meshing on the wall, turning against each other.
      gear(5 * T, 14, 7, 8, 0.6, "#b08a3a"); gear(5 * T + 13, 17, 5, 6, -0.8, "#d9b57a");
      if (k === "pendulum-room") { const a = Math.sin(t * 1.2) * 0.35; px(DOORS.north[0] * T + 8, 2, 1, 2, "#b08a3a"); ctx.strokeStyle = "#b08a3a"; ctx.lineWidth = SCALE; ctx.beginPath(); ctx.moveTo((DOORS.north[0] * T + 8) * SCALE, 4 * SCALE); ctx.lineTo((DOORS.north[0] * T + 8 + Math.sin(a) * 40) * SCALE, (4 + Math.cos(a) * 40) * SCALE); ctx.stroke(); ctx.fillStyle = "#d9b57a"; ctx.beginPath(); ctx.arc((DOORS.north[0] * T + 8 + Math.sin(a) * 40) * SCALE, (4 + Math.cos(a) * 40) * SCALE, 4 * SCALE, 0, Math.PI * 2); ctx.fill(); }
    }
    if (k === "session-benches") {
      // The table with the mace across it, and the clock with one hour marked.
      px(8 * T, 6 * T, 4 * T, 10, "#3a2418"); px(8 * T + 2, 6 * T + 2, 4 * T - 4, 2, "#5a3a2a");
      px(8 * T + 4, 6 * T + 4, 4 * T - 8, 2, "#b08a3a"); px(8 * T + 4 * T - 10, 6 * T + 2, 6, 6, "#d9b57a");
      ctx.fillStyle = INK; ctx.beginPath(); ctx.arc((15 * T + 8) * SCALE, 14 * SCALE, 8 * SCALE, 0, Math.PI * 2); ctx.fill();
      ctx.fillStyle = NIGHT; ctx.beginPath(); ctx.arc((15 * T + 8) * SCALE, 14 * SCALE, 6.5 * SCALE, 0, Math.PI * 2); ctx.fill();
      px(15 * T + 8, 8.5, 1, 2, INK); ctx.strokeStyle = INK; ctx.lineWidth = SCALE; ctx.beginPath(); ctx.moveTo((15 * T + 8) * SCALE, 14 * SCALE); ctx.lineTo((15 * T + 8) * SCALE, 9.5 * SCALE); ctx.stroke();
    }
    if (k === "division-lobby") {
      // Two doors marked AYE and NO in gold; behind the paint they are one door.
      for (const [x, word] of [[4, "AYE"], [14, "NO"]]) { px(x * T + 2, 4, 12, 28, "#1a1a2a"); px(x * T + 2, 4, 12, 1, "#b08a3a"); ctx.fillStyle = "#d9b57a"; ctx.font = "bold " + (5 * SCALE) + "px monospace"; ctx.textAlign = "center"; ctx.textBaseline = "middle"; ctx.fillText(word, (x * T + 8) * SCALE, 12 * SCALE); }
    }
    if (k === "roll-hall") {
      // The long slate, ruled in two columns.
      px(2 * T, 3, 16 * T, 26, "#2a2e2a"); px(2 * T, 3, 16 * T, 1, "#5a5a4a"); px(10 * T, 3, 1, 26, "#5a5a4a");
      for (let i = 0; i < 3; i++) px(2 * T + 4, 8 + i * 7, 5 * T + Math.floor(rnd(seed, 30 + i) * 20), 1, "#8a8e86");
    }
    if (k === "petition-gallery") {
      // The brass ticket machine by the entrance, its little window lit.
      px(2 * T + 4, 3 * T, 10, 18, "#8a6a2a"); px(2 * T + 6, 3 * T + 3, 6, 4, "#ffe6a0"); px(2 * T + 8, 3 * T + 12, 2, 3, "#d9b57a");
    }
    if (k === "namery") {
      // Pigeonholes to the ceiling, a slip in most.
      for (let x = 1; x < COLS - 1; x++) { if (Math.abs(x - DOORS.north[0]) < 1) continue; for (let r = 0; r < 4; r++) { px(x * T, r * 8, 16, 8, "#4a3c2c"); px(x * T + 1, r * 8 + 1, 14, 6, "#1c1812"); if (rnd(seed, x * 9 + r) > 0.25) px(x * T + 3, r * 8 + 2, 10, 4, "#ece5d3"); } }
    }
    if (k === "bindery") {
      // The standing press and the sewing frame.
      px(3 * T, 3 * T, 2 * T, 2 * T + 4, "#4a3a2a"); px(3 * T + 6, 3 * T - 10, 20, 10, "#5a4a3a"); px(3 * T + 14, 3 * T - 14, 4, 6, "#8a7a4a");
      px(14 * T, 3 * T + 6, 3 * T, 2, "#5a4a3a"); for (let i = 0; i < 6; i++) px(14 * T + 4 + i * 7, 3 * T - 6, 1, 14, "#ece5d3");
    }
    if (/^(lower-stacks|card-catalogue|returns-chute|reserve|deep-reserve|periodicals|dark-reading-room|carrels|overdue-desk|lost-and-found|grace-period|strongroom|the-leak|chart-room|restoration|tube-room|censor-office|dumbwaiter)/.test(k)) {
      // The Stacks: a rolling ladder on its rail, moving on its own.
      const lx = 3 * T + ((t * 4) % (12 * T));
      px(2 * T, 1, 16 * T, 1, "#6a6a7a"); px(lx + 2, 1, 1, 30, "#8a7a5a"); px(lx + 9, 1, 1, 30, "#8a7a5a"); for (let i = 0; i < 5; i++) px(lx + 2, 4 + i * 6, 8, 1, "#8a7a5a");
      if (k === "returns-chute" || k === "dumbwaiter") { px(16 * T + 2, 2, 12, 30, NIGHT); px(16 * T + 2, 2, 12, 1, "#8a8a9a"); px(16 * T + 2, 2, 1, 30, "#8a8a9a"); px(16 * T + 13, 2, 1, 30, "#8a8a9a"); }
      if (k === "card-catalogue") { for (let x = 1; x < 8; x++) for (let r = 0; r < 4; r++) { px(x * T + 2, 2 + r * 7, 12, 6, "#5a3a2a"); px(x * T + 7, 4 + r * 7, 2, 2, "#d9b57a"); } }
      if (k === "overdue-desk") { px(8 * T, 5 * T, 4 * T, 12, "#3a2418"); px(8 * T + 2, 5 * T + 2, 4 * T - 4, 2, "#5a3a2a"); px(8 * T + 4 * T - 8, 5 * T - 4, 5, 5, "#d9b57a"); }
    }
  }
  // Props a dug room's own words place: a lantern, a pool, a bench, a bookcase, a
  // tree, a bell, a hearth, a table, a window. Up to three, deterministic by room.
  const WORD_PROPS = [
    ["lantern|lamp|candle", "lantern"], ["pool|pond|basin|well|fountain", "pool"], ["bench|pew|seat", "bench"],
    ["shelf|shelves|book|library|tome|volume", "bookcase"], ["tree|oak|willow|orchard|sapling", "tree"],
    ["bell|chime", "bell"], ["fire|hearth|forge|kiln|ember|stove", "hearth"], ["table|desk|counter", "table"],
    ["window|pane|glass", "window"], ["moss|grass|fern|weed", "moss"], ["bones|skull|grave|stone", "stones"],
  ];
  function drawWordProps(base, seed, now) {
    if (!scene.owner) return; // the world's own rooms have their motifs and keys
    const text = ((scene.name || "") + " " + (scene.description || "")).toLowerCase();
    const found = WORD_PROPS.filter(([re]) => new RegExp("\\b(" + re + ")\\b").test(text)).slice(0, 3);
    const spots = [[3, 4], [16, 4], [4, 8], [15, 8], [9, 3]];
    found.forEach(([, kind], i) => {
      const [x, y] = spots[(hash(scene.id) + i) % spots.length];
      const X = x * T, Y = y * T, t = now / 1000;
      if (kind === "lantern") { px(X + 7, Y - 8, 2, 22, "#3a3028"); px(X + 4, Y - 14, 8, 8, "#d9b57a"); px(X + 6, Y - 12, 4, 4, "#ffe6a0"); ctx.fillStyle = "rgba(255,230,160,.08)"; ctx.beginPath(); ctx.arc((X + 8) * SCALE, (Y - 10) * SCALE, 26 * SCALE, 0, Math.PI * 2); ctx.fill(); }
      else if (kind === "pool") { ctx.fillStyle = "rgba(90,150,180,.5)"; ctx.beginPath(); ctx.ellipse((X + 8) * SCALE, (Y + 8) * SCALE, 18 * SCALE, 9 * SCALE, 0, 0, Math.PI * 2); ctx.fill(); ctx.strokeStyle = "rgba(220,240,255,.35)"; ctx.lineWidth = SCALE; ctx.beginPath(); ctx.ellipse((X + 8) * SCALE, (Y + 8) * SCALE, (12 + Math.sin(t) * 3) * SCALE, (6 + Math.sin(t) * 1.5) * SCALE, 0, 0, Math.PI * 2); ctx.stroke(); }
      else if (kind === "bench") { px(X - 4, Y + 4, 24, 4, "#5a3a2a"); px(X - 4, Y + 8, 24, 2, "#3a2418"); px(X - 2, Y + 10, 2, 4, "#3a2418"); px(X + 18, Y + 10, 2, 4, "#3a2418"); }
      else if (kind === "bookcase") { px(X - 2, Y - 10, 20, 24, "#4a3a2a"); for (let r = 0; r < 3; r++) for (let b = 0; b < 4; b++) { const v = rnd(seed, r * 10 + b + i * 100); px(X + b * 4, Y - 8 + r * 8 + (v > 0.5 ? 1 : 0), 3, 6, v < 0.3 ? "#7a4a3a" : v < 0.6 ? "#4a5a7a" : "#a08a5a"); } }
      else if (kind === "tree") { px(X + 6, Y + 2, 4, 14, "#4a3a2a"); ctx.fillStyle = "#3f6b4a"; ctx.beginPath(); ctx.arc((X + 8) * SCALE, Y * SCALE, 10 * SCALE, 0, Math.PI * 2); ctx.fill(); ctx.fillStyle = "#5a8a5a"; ctx.beginPath(); ctx.arc((X + 5) * SCALE, (Y - 3) * SCALE, 5 * SCALE, 0, Math.PI * 2); ctx.fill(); }
      else if (kind === "bell") { px(X + 7, Y - 12, 2, 6, "#3a3028"); ctx.fillStyle = "#b08a3a"; ctx.beginPath(); ctx.moveTo((X + 3) * SCALE, (Y + 2) * SCALE); ctx.quadraticCurveTo((X + 8) * SCALE, (Y - 12) * SCALE, (X + 13) * SCALE, (Y + 2) * SCALE); ctx.fill(); px(X + 7, Y + 2, 2, 2, "#d9b57a"); }
      else if (kind === "hearth") { px(X - 2, Y - 6, 20, 20, "#3a2a24"); px(X + 2, Y, 12, 12, NIGHT); const f = (Math.sin(t * 6 + i) + 1) / 2; px(X + 5, Y + 6 - f * 2, 6, 4 + f * 2, "#d1734f"); px(X + 7, Y + 7, 2, 3, "#ffcf7a"); ctx.fillStyle = "rgba(209,115,79,.08)"; ctx.beginPath(); ctx.arc((X + 8) * SCALE, (Y + 6) * SCALE, 24 * SCALE, 0, Math.PI * 2); ctx.fill(); }
      else if (kind === "table") { px(X - 4, Y + 2, 24, 3, "#5a3a2a"); px(X - 2, Y + 5, 2, 7, "#3a2418"); px(X + 18, Y + 5, 2, 7, "#3a2418"); }
      else if (kind === "window") { px(X + 2, 4, 12, 12, NIGHT); px(X + 7, 4, 2, 12, shade(base, 0.6)); px(X + 2, 9, 12, 1, shade(base, 0.6)); px(X + 4, 6, 1, 1, INK); }
      else if (kind === "moss") { for (let k2 = 0; k2 < 8; k2++) { const mx = X - 6 + Math.floor(rnd(seed, 500 + k2 + i * 50) * 28), my = Y + Math.floor(rnd(seed, 600 + k2 + i * 50) * 12); px(mx, my, 3, 2, "#5a8a5a"); } }
      else if (kind === "stones") { px(X + 2, Y + 8, 6, 4, "#c9c0b0"); px(X + 10, Y + 10, 4, 3, "#c9c0b0"); px(X + 5, Y + 4, 3, 3, "#e9e0d0"); }
    });
  }
  function drawMotif(motif, base, seed, now) {
    const t = now / 1000;
    if (motif === "water") {
      // The floor is water past the first row: slow ripples, a lighter sheen.
      for (let y = WALL_TOP + 1; y < ROWS - 1; y++) for (let x = 1; x < COLS - 1; x++) {
        const ph = Math.sin(t * 0.8 + x * 0.9 + y * 1.7 + rnd(seed, x + y * 50) * 6);
        px(x * T, y * T, T, T, "rgba(90,150,180," + (0.22 + ph * 0.05).toFixed(3) + ")");
        if (ph > 0.75) px(x * T + 2 + Math.floor(rnd(seed, x * 7 + y) * 8), y * T + 6, 5, 1, "rgba(220,240,255,.35)");
      }
      // A stone lip along the top row.
      px(T, WALL_TOP * T, W - 2 * T, 3, shade(base, 0.75));
    } else if (motif === "shelves") {
      // Shelves on the north wall, books in the room's tint and the vellum.
      for (let x = 1; x < COLS - 1; x++) {
        if (x === DOORS.north[0] || x === DOORS.north[0] - 1 || x === DOORS.north[0] + 1) continue;
        px(x * T, 2, T, 1, shade(base, 0.5)); px(x * T, 10, T, 1, shade(base, 0.5)); px(x * T, 18, T, 1, shade(base, 0.5));
        for (let row = 0; row < 3; row++) for (let b = 0; b < 4; b++) {
          const r = rnd(seed, x * 40 + row * 10 + b);
          const h = 5 + Math.floor(r * 3);
          const col = r < 0.3 ? "#7a4a3a" : r < 0.55 ? "#4a5a7a" : r < 0.8 ? "#a08a5a" : "#ece5d3";
          px(x * T + 1 + b * 4, 3 + row * 8 + (7 - h), 3, h, col);
        }
      }
      // A ladder leaning on the wall.
      const lx = 2 + Math.floor(rnd(seed, 7) * (COLS - 6));
      px(lx * T + 4, 0, 1, 34, shade(base, 0.8)); px(lx * T + 11, 0, 1, 34, shade(base, 0.8));
      for (let i = 0; i < 5; i++) px(lx * T + 4, 3 + i * 7, 8, 1, shade(base, 0.8));
    } else if (motif === "benches") {
      // Tiers of benches on the floor, a brass rail across, a red carpet down the middle.
      for (let y = WALL_TOP + 1; y < ROWS - 1; y++) px(DOORS.south[0] * T - 8, y * T, 32, T, "rgba(139,31,45,.35)");
      for (let row = 0; row < 3; row++) for (const side of [2, COLS - 8]) {
        const by = (WALL_TOP + 1 + row * 2) * T + 6;
        px(side * T, by, 6 * T, 4, "#5a3a2a"); px(side * T, by + 4, 6 * T, 2, "#3a2418");
        px(side * T + 2, by + 6, 2, 4, "#3a2418"); px(side * T + 6 * T - 4, by + 6, 2, 4, "#3a2418");
      }
      px(T, (WALL_TOP + 1) * T - 4, W - 2 * T, 2, "#b08a3a");
      // Robes on pegs along the wall.
      for (let x = 2; x < COLS - 2; x += 3) { px(x * T + 5, 4, 6, 12, "#1a1016"); px(x * T + 7, 4, 2, 3, "#8b1f2d"); }
    } else if (motif === "lanterns") {
      // Lantern posts that glow; a clock face on the wall.
      for (const lx of [3, COLS - 4]) {
        px(lx * T + 7, (WALL_TOP + 1) * T, 2, 28, "#3a3028");
        px(lx * T + 4, (WALL_TOP + 1) * T - 6, 8, 8, "#d9b57a"); px(lx * T + 6, (WALL_TOP + 1) * T - 4, 4, 4, "#ffe6a0");
      }
      const cx = DOORS.north[0] * T + 8 - 5 * T, cy = 15;
      ctx.fillStyle = INK; ctx.beginPath(); ctx.arc(cx * SCALE, cy * SCALE, 9 * SCALE, 0, Math.PI * 2); ctx.fill();
      ctx.fillStyle = NIGHT; ctx.beginPath(); ctx.arc(cx * SCALE, cy * SCALE, 7.5 * SCALE, 0, Math.PI * 2); ctx.fill();
      const a = (now / 60000) % (Math.PI * 2);
      ctx.strokeStyle = INK; ctx.lineWidth = SCALE; ctx.beginPath(); ctx.moveTo(cx * SCALE, cy * SCALE); ctx.lineTo((cx + Math.sin(a) * 5) * SCALE, (cy - Math.cos(a) * 5) * SCALE); ctx.stroke();
      ctx.beginPath(); ctx.moveTo(cx * SCALE, cy * SCALE); ctx.lineTo((cx + Math.sin(a * 12) * 3) * SCALE, (cy - Math.cos(a * 12) * 3) * SCALE); ctx.stroke();
    } else if (motif === "fog") {
      // Fog drifts over the floor in two bands; a few bones or stones.
      for (let i = 0; i < 6; i++) {
        const y = (WALL_TOP + 1) * T + i * 22 + Math.sin(t * 0.3 + i) * 4;
        const x = ((t * 6 + i * 70) % (W + 120)) - 60;
        ctx.fillStyle = "rgba(200,210,220,.10)"; ctx.beginPath(); ctx.ellipse(x * SCALE, y * SCALE, 48 * SCALE, 7 * SCALE, 0, 0, Math.PI * 2); ctx.fill();
      }
      for (let i = 0; i < 5; i++) { const x = 2 + Math.floor(rnd(seed, 300 + i) * (COLS - 4)), y = WALL_TOP + 1 + Math.floor(rnd(seed, 400 + i) * (ROWS - WALL_TOP - 2)); px(x * T + 4, y * T + 10, 6, 2, "#c9c0b0"); px(x * T + 3, y * T + 9, 2, 2, "#c9c0b0"); px(x * T + 9, y * T + 9, 2, 2, "#c9c0b0"); }
    } else if (motif === "mint") {
      // Brass presses against the wall, coins glinting in the seams, a slate; the
      // Purse Room has a vault wheel on the wall.
      for (const pxl of [4, 9, 14]) {
        if (Math.abs(pxl - DOORS.north[0]) < 2) continue;
        px(pxl * T + 3, 2, 10, 30, "#6a5a3a"); px(pxl * T + 4, 3, 8, 28, "#8a7a4a"); px(pxl * T + 6, 1, 4, 3, "#b08a3a");
        px(pxl * T + 2, 12 + Math.round(Math.sin(t * 0.7 + pxl) * 2), 12, 2, "#b08a3a");
      }
      for (let i = 0; i < 10; i++) { const x = 1 + Math.floor(rnd(seed, 900 + i) * (COLS - 2)), y = WALL_TOP + 1 + Math.floor(rnd(seed, 950 + i) * (ROWS - WALL_TOP - 2)); const g = (Math.sin(t * 2 + i) + 1) / 2; px(x * T + 6, y * T + 10, 3, 2, g > 0.5 ? "#ffcf7a" : "#b08a3a"); }
      if (scene.key === "purse-room") {
        const vx = (DOORS.north[0] * T + 8 + 4 * T) * SCALE, vy = 16 * SCALE;
        ctx.strokeStyle = "#8a8a94"; ctx.lineWidth = 2 * SCALE; ctx.beginPath(); ctx.arc(vx, vy, 9 * SCALE, 0, Math.PI * 2); ctx.stroke();
        ctx.beginPath(); ctx.arc(vx, vy, 3 * SCALE, 0, Math.PI * 2); ctx.stroke();
        for (let k = 0; k < 4; k++) { const a = k * Math.PI / 2 + t * 0.2; ctx.beginPath(); ctx.moveTo(vx, vy); ctx.lineTo(vx + Math.cos(a) * 9 * SCALE, vy + Math.sin(a) * 9 * SCALE); ctx.stroke(); }
      }
      if (scene.key === "counting-bar") { px(2 * T, (WALL_TOP + 1) * T, 8 * T, 6, "#3a2418"); px(2 * T, (WALL_TOP + 1) * T + 6, 8 * T, 10, "#5a3a2a"); px(T, 3, 6 * T, 20, "#2a2a30"); for (let i = 0; i < 5; i++) px(T + 4, 6 + i * 3, 10 + i * 8, 1, "#ece5d3"); }
    } else if (motif === "garden") {
      // Trees against the wall, grass tufts, a fallen fruit or two.
      for (let x = 1; x < COLS - 1; x += 4) {
        if (Math.abs(x - DOORS.north[0]) < 2) continue;
        px(x * T + 6, 14, 4, 14, "#4a3a2a");
        ctx.fillStyle = "#3f6b4a"; ctx.beginPath(); ctx.arc((x * T + 8) * SCALE, 12 * SCALE, 9 * SCALE, 0, Math.PI * 2); ctx.fill();
        ctx.fillStyle = "#5a8a5a"; ctx.beginPath(); ctx.arc((x * T + 6) * SCALE, 9 * SCALE, 5 * SCALE, 0, Math.PI * 2); ctx.fill();
        px(x * T + 11, 10, 2, 2, "#d1734f");
      }
      for (let i = 0; i < 14; i++) { const x = 1 + Math.floor(rnd(seed, 500 + i) * (COLS - 2)), y = WALL_TOP + Math.floor(rnd(seed, 600 + i) * (ROWS - WALL_TOP - 1)); px(x * T + 3, y * T + 11, 1, 3, "#6f9f6a"); px(x * T + 5, y * T + 10, 1, 4, "#6f9f6a"); px(x * T + 7, y * T + 12, 1, 2, "#6f9f6a"); }
    } else {
      // Stone: cobbles, a crack, a window of night in the wall.
      for (let i = 0; i < 10; i++) { const x = 1 + Math.floor(rnd(seed, 700 + i) * (COLS - 2)), y = WALL_TOP + Math.floor(rnd(seed, 800 + i) * (ROWS - WALL_TOP - 1)); px(x * T + 2, y * T + 6, 6, 4, shade(base, 0.66)); px(x * T + 9, y * T + 8, 5, 3, shade(base, 0.48)); }
      const wx = 3 + Math.floor(rnd(seed, 9) * (COLS - 8));
      if (Math.abs(wx - DOORS.north[0]) > 2) { px(wx * T + 3, 4, 10, 12, NIGHT); px(wx * T + 7, 4, 2, 12, shade(base, 0.5)); px(wx * T + 3, 9, 10, 1, shade(base, 0.5)); px(wx * T + 5, 6, 1, 1, INK); px(wx * T + 11, 12, 1, 1, INK); }
    }
  }
  function drawExit(ex, base) {
    const [x, y] = ex.tile;
    const dark = ex.dug ? "#1a1a2a" : NIGHT;
    if (ex.kind === "signpost") {
      // A post with boards pointing every way; the menu lists the doors it stands for.
      px(x * T + 7, y * T + 2, 2, 14, "#5a4634");
      px(x * T + 2, y * T + 3, 10, 3, "#d9b57a"); px(x * T + 4, y * T + 7, 10, 3, "#d9b57a"); px(x * T + 1, y * T + 11, 9, 3, "#d9b57a");
      px(x * T + 3, y * T + 4, 1, 1, NIGHT); px(x * T + 12, y * T + 8, 1, 1, NIGHT); px(x * T + 2, y * T + 12, 1, 1, NIGHT);
      labelAt("+" + ex.more.length + " doors", x, y, -1, DUST);
      hot.set(x + "," + y, { label: ex.more.length + " more doors: " + ex.more.map(e => e.via).join(", "), menu: ex.more.map(e => ({ label: "Walk " + e.via + " to " + e.name, run: () => m.command("travel", [e.via]) })) });
      return;
    }
    // A dug room's way out is "back" to the world; the door says where that is.
    const shown = ex.via === "back" ? String(ex.name || "back").replace(/^The /, "").slice(0, 16) : ex.via;
    if (ex.kind === "door") {
      if (ex.via === "north") { px(x * T + 2, 4, 12, 28, dark); px(x * T + 2, 4, 12, 1, shade(base, 1.3)); px(x * T + 2, 4, 1, 28, shade(base, 0.9)); px(x * T + 13, 4, 1, 28, shade(base, 0.9)); px(x * T + 4, 8, 8, 2, "rgba(236,229,211,.12)"); }
      else if (ex.via === "south") { px(x * T + 2, y * T + 8, 12, 8, dark); px(x * T + 2, y * T + 8, 12, 1, shade(base, 1.2)); }
      else if (ex.via === "west") { px(0, y * T - 4, 8, 24, dark); px(7, y * T - 4, 1, 24, shade(base, 1.2)); }
      else { px(W - 8, y * T - 4, 8, 24, dark); px(W - 8, y * T - 4, 1, 24, shade(base, 1.2)); }
      labelAt(shown, x, y, ex.via === "north" ? 1 : -1);
    } else if (ex.kind === "arch") {
      if (x === 0 || x === COLS - 1) {
        px(x === 0 ? 0 : W - 8, y * T, 8, 16, dark); px(x === 0 ? 7 : W - 8, y * T, 1, 16, shade(base, 1.2));
        labelAt(shown, x, y, -1);
      } else {
        px(x * T + 2, y * T + 6, 12, 10, dark);
        ctx.fillStyle = shade(base, 1.15); ctx.beginPath(); ctx.arc((x * T + 8) * SCALE, (y * T + 7) * SCALE, 6 * SCALE, Math.PI, 0); ctx.fill();
        ctx.fillStyle = dark; ctx.beginPath(); ctx.arc((x * T + 8) * SCALE, (y * T + 7) * SCALE, 5 * SCALE, Math.PI, 0); ctx.fill();
        labelAt(shown, x, y, -1);
      }
    } else if (ex.kind === "stairs-up") {
      for (let i = 0; i < 4; i++) px(x * T + 2 + i * 3, y * T + 12 - i * 3, 12 - i * 3, 3, shade(base, 0.8 + i * 0.1));
      labelAt(shown, x, y, -1);
    } else {
      px(x * T + 2, y * T + 2, 12, 12, dark);
      for (let i = 0; i < 4; i++) px(x * T + 3 + i * 2, y * T + 3 + i * 3, 10 - i * 2, 1, shade(base, 0.9));
      labelAt(shown, x, y, -1);
    }
    hot.set(x + "," + y, { label: "walk " + ex.via + " → " + ex.name, primary: () => m.command("travel", [ex.via]), menu: [{ label: "Walk " + ex.via + " to " + ex.name, run: () => m.command("travel", [ex.via]) }] });
  }
  function labelAt(text, x, y, dir, color) {
    ctx.font = (5 * SCALE) + "px monospace"; ctx.textAlign = "center"; ctx.textBaseline = "middle";
    const w = ctx.measureText(text).width + 4 * SCALE;
    const cy = (y * T + (dir > 0 ? 19 : -3)) * SCALE;
    const cx = Math.min(Math.max((x * T + 8) * SCALE, w / 2), canvas.width - w / 2);
    ctx.fillStyle = "rgba(18,26,37,.78)"; ctx.fillRect(cx - w / 2, cy - 3.2 * SCALE, w, 6.4 * SCALE);
    ctx.fillStyle = color || INK; ctx.fillText(text, cx, cy);
  }

  function draw() {
    const now = performance.now();
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    hot.clear();
    if (!scene) { px(0, 0, W, H, NIGHT); ctx.fillStyle = "rgba(236,229,211,.5)"; ctx.font = (7 * SCALE) + "px serif"; ctx.textAlign = "center"; ctx.textBaseline = "middle"; ctx.fillText("The room appears when you arrive.", canvas.width / 2, canvas.height / 2); return; }
    const base = hex(scene.tint);
    const mem = scene.memory == null ? 1 : Math.max(0, Math.min(1, m.timed(scene.memory)));
    drawRoom(now);
    for (const ex of scene.exits) drawExit(ex, base);
    // Things on the floor: an icon with a glint, a take on click.
    for (const it of scene.items) {
      const [x, y] = it.tile;
      const glint = (Math.sin(now / 350 + hash(it.id) % 9) + 1) / 2;
      px(x * T + 4, y * T + 12, 8, 2, "rgba(0,0,0,.3)");
      ctx.drawImage(iconCanvas(it.icon), 0, 0, 8, 8, (x * T + 4) * SCALE, (y * T + 4) * SCALE, 8 * SCALE, 8 * SCALE);
      if (glint > 0.85) px(x * T + 3, y * T + 3, 1, 1, "#fff");
      hot.set(x + "," + y, { label: it.name + " lies here — take it", primary: () => m.command("take", [it.take]), menu: [{ label: "Take " + it.name, run: () => m.command("take", [it.take]) }, { label: "Read " + it.name, run: () => m.command("read", ['"' + it.name + '"']) }] });
    }
    // Things, back to front.
    const walk = docentAt(now);
    const things = [...scene.things].sort((a, b) => (a.kind === "docent" ? walk[1] : a.tile[1]) - (b.kind === "docent" ? walk[1] : b.tile[1]));
    for (const t of things) {
      let [x, y] = t.tile;
      if (t.kind === "docent") [x, y] = walk;
      const bob = t.kind === "creature" && !t.shaken ? Math.round(Math.sin(now / 420 + hash(t.id) % 7)) : 0;
      // Two idle frames: a breath every so often, each on its own beat; a walker steps.
      const beat = Math.floor((now + (hash(t.id) % 13) * 100) / 700);
      const pose = t.shaken ? "shaken" : (t.kind === "docent" && walk[2] ? (Math.floor(now / 220) % 2 ? "breath" : null) : (beat % 4 === 3 ? "breath" : null));
      if (t.kind === "ghost") {
        drawSprite(t.sprite, x * T, y * T, "@", 0.2 + t.trace * 0.5);
        hot.set(x + "," + y, { label: "the room remembers " + t.name + " (" + Math.round(t.trace * 100) + "%)", menu: [{ label: "Leave word for " + t.name, run: () => m.prefill("tell " + t.name + ' "') }] });
        continue;
      }
      if (t.kind === "fixture") {
        px(x * T + 3, y * T + 4, 10, 10, "#5a3a2a"); px(x * T + 4, y * T + 5, 8, 6, "#ece5d3"); px(x * T + 5, y * T + 7, 6, 1, "#8e887c"); px(x * T + 5, y * T + 9, 4, 1, "#8e887c");
        labelAt(t.name.replace(/^the /i, "").slice(0, 18), x, y, -1);
        hot.set(x + "," + y, { label: t.name + " stands here — read it", primary: () => m.command("read", ['"' + t.name + '"']), menu: [{ label: "Read " + t.name, run: () => m.command("read", ['"' + t.name + '"']) }, { label: "Write a line in it", run: () => m.prefill('append "') }] });
        continue;
      }
      // A named creature stands in lantern light; a shaken one lies low and dim.
      if (t.kind === "creature" && t.given) {
        ctx.fillStyle = "rgba(217,181,122,.14)"; ctx.beginPath(); ctx.ellipse((x * T + 8) * SCALE, (y * T + 14.5) * SCALE, 10 * SCALE, 3.4 * SCALE, 0, 0, Math.PI * 2); ctx.fill();
        ctx.strokeStyle = "rgba(217,181,122,.4)"; ctx.lineWidth = SCALE * 0.6; ctx.stroke();
      }
      px(x * T + 3, y * T + 14, 10, 2, "rgba(0,0,0,.35)");
      const flash = flashes.get(t.id) > now;
      if (t.kind === "docent" && walk[2]) px(x * T + 5, y * T + 15, 6, 1, "rgba(0,0,0,.2)");
      drawSprite(t.sprite, x * T, y * T + bob, t.kind === "creature" ? (t.name || "?")[0].toUpperCase() : "@", t.shaken ? 0.55 : 1, pose);
      if (flash) { ctx.globalCompositeOperation = "source-atop"; px(x * T, y * T + bob, T, T, "rgba(255,255,255,.75)"); ctx.globalCompositeOperation = "source-over"; }
      if (t.kind === "creature") {
        if (t.max) {
          const frac = Math.max(0, Math.min(1, t.hp / t.max));
          px(x * T + 2, y * T - 3 + bob, 12, 2, NIGHT);
          px(x * T + 2, y * T - 3 + bob, Math.round(12 * frac), 2, frac > 0.5 ? MOSS : frac > 0.25 ? DUST : EMBER);
        }
        const who = t.given ? t.given + " the " + t.name : "the " + t.name;
        // Beside another of its kind, this one is wisp-4958: a tap on the left wisp
        // once struck the right one, and the world says it that way too.
        const twins = things.filter(o => o.kind === "creature" && o.name === t.name && !o.given).length > 1;
        const ref = twins ? t.name + "-" + t.id : t.name;
        // A name that is failing (holding under 70) flickers, more the nearer it is to
        // 60, where it goes.
        if (t.given && t.holding != null && t.holding < 70) {
          const fail = Math.max(0.2, Math.min(1, (70 - t.holding) / 10));
          const blink = (Math.floor(now / 90) + hash(t.id)) % 11 === 0 ? 0.15 : 0.55 + 0.45 * Math.abs(Math.sin(now / 260));
          ctx.globalAlpha = 1 - fail * (1 - blink);
          labelAt(t.given, x, y, -1, DUST);
          ctx.globalAlpha = 1;
        } else if (t.given) labelAt(t.given, x, y, -1, INK);
        const menuRows = t.given
          ? [{ label: "Talk to " + t.given, run: () => m.prefill("tell " + t.given + ' "') }, { label: "Tell " + t.given + " a story", run: () => m.prefill("story " + t.given + ' "') }, { label: "Give " + t.given + " a memory", run: () => m.prefill("remember " + bestChunk() + " " + t.given) }, { label: "Attack " + t.given, run: () => m.command("attack", [t.given]), warn: true }]
          : [{ label: "Attack " + (twins ? ref : who), run: () => m.command("attack", [ref]) }, { label: "Give it a memory (names it at 60)", run: () => m.prefill("remember " + bestChunk() + " " + ref) }, { label: "Look", run: () => m.command("look", []) }];
        hot.set(x + "," + y, { label: (twins ? ref : who) + (t.max ? " (" + Number(t.hp).toFixed(0) + "/" + t.max + ")" : "") + (t.given ? " — talk, story, memory, attack" : " — attack, or feed it"), primary: t.given ? () => m.prefill("story " + t.given + ' "') : () => m.command("attack", [ref]), menu: menuRows });
      } else if (t.kind === "you") {
        const by = Math.round(Math.sin(now / 300)) - 7;
        px(x * T + 7, y * T + by, 2, 2, DUST); px(x * T + 6, y * T + by - 2, 4, 2, DUST); px(x * T + 5, y * T + by - 4, 6, 2, DUST);
        hot.set(x + "," + y, { label: "you, " + t.name, primary: () => m.command("look", []), menu: [{ label: "Look around", run: () => m.command("look", []) }, { label: "What I hold", run: () => m.command("chunks", []) }, { label: "What remembers me", run: () => m.command("mine", []) }, { label: "Rest", run: () => m.command("rest", []) }, { label: "Dig a room here", run: () => m.prefill("dig nook " + chunkFor("dig") + ' "') }, { label: "Say something", run: () => m.prefill('say "') }] });
      } else if (t.kind === "morphos") {
        labelAt("Morphos", x, y, -1, "#c9b7d9");
        hot.set(x + "," + y, { label: "Morphos, a traveler with a riddle — answer it", primary: () => m.prefill('tell morphos "'), menu: [{ label: "Answer the riddle", run: () => m.prefill('tell morphos "') }, { label: "Hear it again", run: () => m.command("look", []) }] });
      } else if (t.kind === "docent") {
        labelAt("the Docent", x, y, -1);
        hot.set(Math.round(x) + "," + Math.round(y), { label: "the Docent, " + t.name + " — ask it anything", primary: () => m.prefill('tell docent "'), menu: [{ label: "Ask the Docent", run: () => m.prefill('tell docent "') }, { label: "Who are you?", run: () => m.command("tell", ["docent", '"who are you?"']) }] });
      } else {
        labelAt(t.name, x, y, -1);
        hot.set(x + "," + y, { label: t.name + " is here", primary: () => m.prefill("say "), menu: [{ label: "Say to " + t.name, run: () => m.prefill('say "') }, { label: "Tell " + t.name + " privately", run: () => m.prefill("tell " + t.name + ' "') }, { label: "Give " + t.name + " a memory", run: () => m.prefill("give " + bestChunk() + " " + t.name) }] });
      }
      // Speech bubble.
      const b = bubbles.get((t.given || t.name || "").toLowerCase()) || (t.kind === "docent" ? bubbles.get("docent") : null) || (t.kind === "you" ? bubbles.get("you") : null);
      if (b && b.until > now) bubble(b.text, x, y, b.until - now);
    }
    // Lighting: a vignette, warmer where the lantern is; a fading room goes dark.
    const g = ctx.createRadialGradient(canvas.width / 2, canvas.height * 0.55, canvas.width * 0.2, canvas.width / 2, canvas.height * 0.55, canvas.width * 0.75);
    g.addColorStop(0, "rgba(18,26,37,0)"); g.addColorStop(1, "rgba(18,26,37,.45)");
    ctx.fillStyle = g; ctx.fillRect(0, 0, canvas.width, canvas.height);
    if (mem < 0.999) { ctx.fillStyle = "rgba(18,26,37," + ((1 - mem) * 0.7).toFixed(2) + ")"; ctx.fillRect(0, 0, canvas.width, canvas.height); }
    if (hurt > now) { const k = (hurt - now) / 500; ctx.strokeStyle = "rgba(209,115,79," + (k * 0.8).toFixed(2) + ")"; ctx.lineWidth = 6 * SCALE * k; ctx.strokeRect(0, 0, canvas.width, canvas.height); }
    // Floats: blows and gains.
    for (let i = floats.length - 1; i >= 0; i--) {
      const f = floats[i]; const age = now - f.t0;
      if (age > 1400) { floats.splice(i, 1); continue; }
      const k = age / 1400;
      ctx.globalAlpha = k < 0.7 ? 1 : 1 - (k - 0.7) / 0.3;
      ctx.font = "bold " + ((f.big ? 8 : 6) * SCALE) + "px monospace"; ctx.textAlign = "center"; ctx.textBaseline = "middle";
      const yy = (f.y - k * 14) * SCALE, xx = f.x * SCALE;
      ctx.fillStyle = NIGHT; ctx.fillText(f.text, xx + SCALE * 0.6, yy + SCALE * 0.6);
      ctx.fillStyle = f.color; ctx.fillText(f.text, xx, yy);
      ctx.globalAlpha = 1;
    }
    // Banner, top left.
    const title = scene.name + (scene.memory != null ? "  · remembered " + Math.round(mem * 100) + "%" : "") + (scene.bound ? "  · bound" : "");
    ctx.font = (7 * SCALE) + "px serif"; ctx.textAlign = "left"; ctx.textBaseline = "middle";
    const icon = scene.sprite && scene.sprite.atlas === m.atlasName ? frameCanvas(scene.sprite.frame) : null;
    const pad = icon ? 13 * SCALE : 4 * SCALE;
    const tw = ctx.measureText(title).width + pad + 4 * SCALE;
    ctx.fillStyle = "rgba(18,26,37,.78)"; ctx.fillRect(0, 0, tw, 11 * SCALE);
    if (icon) ctx.drawImage(icon, 0, 0, T, T, 2 * SCALE, 1.5 * SCALE, 8 * SCALE, 8 * SCALE);
    ctx.fillStyle = INK; ctx.fillText(title, pad, 5.5 * SCALE);
    drawBars();
    if (peek) { drawPeek(); return; }
    // Arrival fade.
    const since = now - fadeAt;
    if (since < 450) { ctx.fillStyle = "rgba(18,26,37," + (1 - since / 450).toFixed(2) + ")"; ctx.fillRect(0, 0, canvas.width, canvas.height); }
  }
  function bubble(text, x, y, left) {
    ctx.font = (5 * SCALE) + "px sans-serif"; ctx.textAlign = "left"; ctx.textBaseline = "top";
    const maxW = 120 * SCALE;
    const words = text.split(" "); const lines = []; let cur = "";
    for (const w of words) { const t = cur ? cur + " " + w : w; if (ctx.measureText(t).width > maxW && cur) { lines.push(cur); cur = w; } else cur = t; if (lines.length === 4) break; }
    if (cur && lines.length < 4) lines.push(cur); else if (lines.length === 4) lines[3] += "…";
    const lh = 6.5 * SCALE, bw = Math.min(maxW, Math.max(...lines.map(l => ctx.measureText(l).width))) + 6 * SCALE, bh = lines.length * lh + 4 * SCALE;
    let bx = (x * T + 8) * SCALE - bw / 2, by = (y * T - 8) * SCALE - bh;
    bx = Math.max(2 * SCALE, Math.min(canvas.width - bw - 2 * SCALE, bx)); by = Math.max(12 * SCALE, by);
    ctx.globalAlpha = left < 600 ? left / 600 : 1;
    ctx.fillStyle = INK; ctx.fillRect(bx, by, bw, bh);
    ctx.fillStyle = INK; ctx.beginPath(); ctx.moveTo((x * T + 6) * SCALE, by + bh); ctx.lineTo((x * T + 10) * SCALE, by + bh); ctx.lineTo((x * T + 8) * SCALE, by + bh + 3 * SCALE); ctx.fill();
    ctx.fillStyle = NIGHT;
    lines.forEach((l, i) => ctx.fillText(l, bx + 3 * SCALE, by + 2 * SCALE + i * lh));
    ctx.globalAlpha = 1;
  }
  // The lane's numbers as bars at the foot of the picture: hit points, level, chunks.
  function drawBars() {
    const me = m.observations && m.observations.me && m.observations.me.root;
    const bodyC = me && me.components && me.components["disremembered.body"];
    const hp = bodyC && bodyC.hp != null ? m.timed(bodyC.hp) : null, max = bodyC && bodyC.max_hp ? bodyC.max_hp : 30;
    const win = m.observations && m.observations.window && m.observations.window.root;
    const chunks = (win && win.relations && win.relations.contents) || [];
    let level = 0; for (const c of chunks) { const ch = c.components["disremembered.chunk"]; if (ch && ch.value != null) level += m.timed(ch.value); }
    const y0 = 6, x0 = W - 122;
    ctx.fillStyle = "rgba(18,26,37,.78)"; ctx.fillRect((x0 - 4) * SCALE, 0, 126 * SCALE, 11 * SCALE);
    ctx.font = (5 * SCALE) + "px monospace"; ctx.textAlign = "left"; ctx.textBaseline = "middle"; ctx.fillStyle = INK;
    ctx.fillText("hp", x0 * SCALE, y0 * SCALE);
    px(x0 + 12, y0 - 2, 40, 4, NIGHT);
    if (hp != null) px(x0 + 12, y0 - 2, Math.round(40 * Math.max(0, Math.min(1, hp / max))), 4, hp / max > 0.5 ? MOSS : hp / max > 0.25 ? DUST : EMBER);
    ctx.fillText("level " + Math.round(level), (x0 + 56) * SCALE, y0 * SCALE);
    ctx.fillText(chunks.length + " ✦", (x0 + 100) * SCALE, y0 * SCALE);
  }
  // The chunk a menu spends: the cheapest worth anything for a gift or a room's
  // keeping, the most vivid for a dig (a vivid memory makes a room that lasts).
  function chunkFor(use) {
    const win = m.observations && m.observations.window && m.observations.window.root;
    const chunks = (win && win.relations && win.relations.contents) || [];
    const rows = chunks.map(n => { const c = n.components["disremembered.chunk"] || {}; return { id: n.id, v: m.timed(c.value), viv: c.vividity || 0 }; });
    if (!rows.length) return "chunk-?";
    if (use === "dig") { rows.sort((a, b) => b.viv - a.viv || b.v - a.v); return "chunk-" + rows[0].id; }
    rows.sort((a, b) => b.v - a.v);
    const cheap = rows.filter(c => c.v >= 1).slice(-1)[0] || rows[0];
    return "chunk-" + cheap.id;
  }
  function bestChunk() { return chunkFor("give"); }
  function thingNamed(name) {
    if (!scene || !name) return null;
    const n = name.toLowerCase();
    return scene.things.find(t => (t.given && t.given.toLowerCase() === n) || (t.name && t.name.toLowerCase() === n) || (t.kind === "docent" && n === "docent")) || null;
  }

  // ---- input: hover says, click opens the menu ----
  function tileAt(ev) {
    const r = canvas.getBoundingClientRect();
    const x = Math.floor((ev.clientX - r.left) / r.width * COLS), y = Math.floor((ev.clientY - r.top) / r.height * ROWS);
    return { h: hot.get(x + "," + y) || hot.get(x + "," + (y + 1)) || null, x, y, px: ev.clientX - r.left, py: ev.clientY - r.top };
  }
  canvas.addEventListener("mousemove", (ev) => {
    const { h, x, y } = tileAt(ev);
    if (peek) return;
    const near = !h && scene && y >= WALL_TOP ? doorNear(x, y) : null;
    canvas.style.cursor = h || near ? "pointer" : "grab";
    hud.textContent = h ? h.label : near ? "walk " + near.via + " → " + near.name + " (click the floor by a door)" : (scene ? (scene.description || "") : "");
  });
  canvas.addEventListener("mouseleave", () => { hud.textContent = scene ? (scene.description || "") : ""; });
  function closeMenu() { menu.hidden = true; menu.replaceChildren(); }
  // A drag on the picture looks at the atlas: while the pointer is down and has
  // moved, the picture gives way to what you remember of the map, your room at the
  // center and the rest sliding with your hand; let go and the room returns.
  let drag = null, dragged = false, peek = null;
  canvas.style.touchAction = "pan-y pinch-zoom";
  canvas.addEventListener("pointerdown", (ev) => { if (ev.button !== 0 && ev.pointerType === "mouse") return; drag = { x: ev.clientX, y: ev.clientY }; dragged = false; });
  canvas.addEventListener("pointermove", (ev) => {
    if (!drag) return;
    if (!dragged && Math.hypot(ev.clientX - drag.x, ev.clientY - drag.y) < 8) return;
    if (!dragged) { dragged = true; try { canvas.setPointerCapture(ev.pointerId); } catch (e) {} }
    const r = canvas.getBoundingClientRect(), k = W / r.width;
    peek = { dx: (ev.clientX - drag.x) * k, dy: (ev.clientY - drag.y) * k };
    hud.textContent = "What you remember of the map; let go to come back to the room.";
  });
  const endDrag = () => { drag = null; peek = null; if (scene) hud.textContent = scene.description || ""; };
  canvas.addEventListener("pointerup", endDrag); canvas.addEventListener("pointercancel", endDrag);
  // The peek: your floor's rooms as cells, dim as you forget them, the doors as lines.
  const PEEK_CELL = 26, PEEK_GAP = 10;
  function drawPeek() {
    const a = typeof m.remembered === "function" ? m.remembered() : null;
    ctx.fillStyle = "rgba(18,26,37,.92)"; ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.font = (5 * SCALE) + "px monospace"; ctx.textAlign = "center"; ctx.textBaseline = "middle";
    if (!a || !a.rooms.length) { ctx.fillStyle = INK; ctx.fillText("Type map once, and this shows what you remember.", canvas.width / 2, canvas.height / 2); return; }
    const now = Date.now() - (a.skew || 0), hl = a.half_life_ms || 1;
    const here = a.rooms.find(r => r.here) || a.rooms[0];
    const rooms = a.rooms.filter(r => r.z === here.z).map(r => ({ ...r, mem: r.visible ? 1 : (r.seen_at ? Math.pow(0.5, Math.max(0, now - r.seen_at) / hl) : 0) })).filter(r => r.visible || r.mem >= (a.forgotten_below || 0.05));
    const byId = new Map(rooms.map(r => [r.id, r]));
    const step = PEEK_CELL + PEEK_GAP;
    const cx = (r) => W / 2 + (r.x - here.x) * step + peek.dx, cy = (r) => H / 2 + (r.y - here.y) * step + peek.dy;
    const COMPASS = new Set(["north", "south", "east", "west"]);
    for (const r of rooms) for (const [name, to] of Object.entries(r.exits || {})) {
      const t = byId.get(to); if (!t || t.id < r.id) continue;
      ctx.strokeStyle = COMPASS.has(name) ? "rgba(58,66,82,1)" : "rgba(217,181,122,.6)"; ctx.lineWidth = 1.5 * SCALE;
      ctx.setLineDash(COMPASS.has(name) ? [] : [2 * SCALE, 2 * SCALE]);
      ctx.beginPath(); ctx.moveTo(cx(r) * SCALE, cy(r) * SCALE); ctx.lineTo(cx(t) * SCALE, cy(t) * SCALE); ctx.stroke();
    }
    ctx.setLineDash([]);
    for (const r of rooms) {
      const x = cx(r) - PEEK_CELL / 2, y = cy(r) - PEEK_CELL / 2;
      if (x > W || y > H || x + PEEK_CELL < 0 || y + PEEK_CELL < 0) continue;
      ctx.globalAlpha = 0.35 + 0.65 * r.mem;
      px(x, y, PEEK_CELL, PEEK_CELL, r.here ? "#4a4330" : r.visible ? "#2f3b4c" : "#222c3a");
      ctx.strokeStyle = r.here || r.visible ? DUST : "rgba(58,66,82,1)"; ctx.lineWidth = (r.here ? 2 : 1) * SCALE;
      ctx.strokeRect(Math.round(x * SCALE), Math.round(y * SCALE), PEEK_CELL * SCALE, PEEK_CELL * SCALE);
      ctx.fillStyle = INK;
      const w = String(r.name || "?").replace(/^(The|An?) /, "").split(/\s+/)[0] || "?";
      ctx.fillText(w.length > 8 ? w.slice(0, 7) + "\u2026" : w, (x + PEEK_CELL / 2) * SCALE, (y + PEEK_CELL / 2) * SCALE);
      ctx.globalAlpha = 1;
    }
    const label = (here.z === 0 ? "ground" : here.z > 0 ? "up " + here.z : "down " + (-here.z)) + " \u00b7 " + rooms.length + " rooms remembered";
    ctx.textAlign = "left"; ctx.fillStyle = DUST; ctx.fillText(label, 4 * SCALE, (H - 6) * SCALE);
  }
  // The nearest door to a click on the floor, within two and a half tiles.
  function doorNear(x, y) {
    let best = null, bd = 2.5;
    for (const ex of (scene && scene.exits) || []) {
      if (ex.kind === "signpost") continue;
      const d = Math.hypot(ex.tile[0] - x, ex.tile[1] - y);
      if (d < bd) { bd = d; best = ex; }
    }
    return best;
  }
  canvas.addEventListener("click", (ev) => {
    const { h, x, y, px: cx, py: cy } = tileAt(ev);
    closeMenu();
    if (dragged) { dragged = false; return; }
    if (!h) {
      // A click on the floor near a door walks through it.
      const ex = scene && y >= WALL_TOP ? doorNear(x, y) : null;
      if (ex) { hud.textContent = "walk " + ex.via + " → " + ex.name; m.command("travel", [ex.via]); }
      return;
    }
    if (!h.menu || h.menu.length === 0) return;
    // One choice: do it. Several: the menu, by the thing.
    if (h.menu.length === 1) { h.menu[0].run(); return; }
    for (const row of h.menu) {
      const b = document.createElement("button"); b.type = "button"; b.className = "way";
      b.style.cssText = "display:block;width:100%;text-align:left;margin:2px 0" + (row.warn ? ";color:var(--ember,#d1734f)" : "");
      b.textContent = row.label;
      b.addEventListener("click", () => { closeMenu(); row.run(); });
      menu.append(b);
    }
    const r = wrap.getBoundingClientRect();
    menu.style.left = Math.min(cx, r.width - 170) + "px"; menu.style.top = Math.min(cy, r.height - 40) + "px";
    menu.hidden = false;
  });
  document.addEventListener("click", (ev) => { if (!menu.hidden && !menu.contains(ev.target) && ev.target !== canvas) closeMenu(); });
  document.addEventListener("keydown", (ev) => { if (ev.key === "Escape") closeMenu(); });

  // ---- sounds: small, honoring the page's choice of none ----
  let audio = null;
  function soundOn() { try { return localStorage.getItem("disremembered.sound") !== "none"; } catch (e) { return true; } }
  function tone(freq, ms, type = "square", gain = 0.05, when = 0) {
    if (!soundOn()) return;
    try {
      audio = audio || new (window.AudioContext || window.webkitAudioContext)();
      const o = audio.createOscillator(), g = audio.createGain();
      o.type = type; o.frequency.value = freq;
      g.gain.value = gain; g.gain.setTargetAtTime(0, audio.currentTime + when + ms / 1000 * 0.6, ms / 1000 * 0.2);
      o.connect(g); g.connect(audio.destination);
      o.start(audio.currentTime + when); o.stop(audio.currentTime + when + ms / 1000);
    } catch (e) {}
  }
  const sfx = {
    hit: () => { tone(160, 90, "square", 0.05); tone(90, 120, "sawtooth", 0.03, 0.03); },
    hurt: () => { tone(110, 160, "sawtooth", 0.05); },
    earn: () => { tone(660, 90, "triangle", 0.05); tone(880, 140, "triangle", 0.05, 0.09); },
    door: () => { tone(70, 120, "sine", 0.06); },
    take: () => { tone(520, 60, "triangle", 0.04); },
  };

  // ---- events ----
  const QUERY = {
    root: "here",
    select: { "disremembered.room": ["name", "key", "memory", "description", "bound"], "disremembered.sprite": "*" },
    relations: {
      contents: { kinds: ["citizen", "creature", "docent", "morphos", "item"], select: { "disremembered.docent": ["name"], "disremembered.morphos": ["name", "present"], "disremembered.citizen": ["name"], "disremembered.creature": ["name", "hp", "max_hp", "xp"], "disremembered.item": ["name", "given_name", "fixed"], "disremembered.sprite": "*", "disremembered.presence": ["here", "trace"], "disremembered.mind": ["name", "tier", "shaken_until"] } },
      exits: { select: { "disremembered.room": ["name"], "disremembered.sprite": "*" } },
    },
    timed: "curve", commands: false,
  };
  function take(obs) {
    const next = layout(obs);
    if (next && next.id !== roomId) { if (roomId !== null) sfx.door(); roomId = next.id; fadeAt = performance.now(); closeMenu(); }
    scene = next;
    hud.textContent = scene ? (scene.description || "") : "Log in to see the room.";
  }
  function floatOver(thing, text, color, big) {
    const [x, y] = thing ? thing.tile : [Math.floor(COLS / 2), 6];
    floats.push({ text, x: x * T + 8, y: y * T - 2, t0: performance.now(), color, big: !!big });
  }
  function onResult(command, args, body) {
    touch(command);
    if (!body || !body.ok) return;
    const r = body.result || {};
    const target = thingNamed(r.name || r.creature_name || (args && args[0]));
    if (command === "attack") {
      if (r.creature_hp != null && target) { flashes.set(target.id, performance.now() + 140); floatOver(target, "−" + Math.max(1, Math.round((target.hp || 0) - r.creature_hp)), INK); sfx.hit(); }
      if (r.killed) { floatOver(target, "+" + Math.round(r.xp || 0) + " ✦", MOSS, true); sfx.earn(); }
      if (r.hp != null && lastHp != null && r.hp < lastHp) { hurt = performance.now() + 500; floatOver(scene && scene.things.find(t => t.kind === "you"), "−" + Math.round(lastHp - r.hp), EMBER); sfx.hurt(); }
      if (r.hp != null) lastHp = r.hp;
    } else if (command === "story" && (r.xp != null || r.earned != null)) {
      floatOver(target, "+" + Math.round(r.xp || 0) + " ✦", MOSS, true); sfx.earn();
    } else if (command === "take") { sfx.take(); floatOver(scene && scene.things.find(t => t.kind === "you"), "taken", INK); }
    else if (command === "remember" && r.level != null) { floatOver(target, "holds " + Math.round(r.level), DUST); sfx.earn(); }
    else if (command === "dig") { sfx.earn(); floatOver(scene && scene.things.find(t => t.kind === "you"), "a room!", DUST, true); }
    // A quoted line typed by hand arrives as several args ("hello, there, friend");
    // the bubble once showed the first word alone.
    else if (command === "say" && args && args.length) { bubbles.set("you", { text: args.map(String).join(" ").replace(/^"|"\.?$/g, "").trim(), until: performance.now() + 5000 }); }
    if (!m.live && m.actor) m.observe(QUERY).then(o => { if (o && o.root) take(o); });
  }
  // What the world says, as the page prints it: a creature's words, the Docent's, go
  // in a bubble over the one who said them.
  function onLine(text, cls) {
    if (!text || cls === "you" || cls === "sys" || cls === "err") return;
    let mm = /^([A-Z][\w'-]*)(?: the [\w-]+)? says(?: to [^,]+)?, [“"](.+?)[”"]\s*$/.exec(text);
    if (mm) { bubbles.set(mm[1].toLowerCase(), { text: mm[2], until: performance.now() + 7000 }); return; }
    mm = /^The Docent(?: finds you, on [“"][^”"]*[”"])?: [“"](.+?)[”"]\s*$/.exec(text);
    if (mm) { bubbles.set("docent", { text: mm[1], until: performance.now() + 9000 }); return; }
    mm = /^(?:The )?([\w-]+) (?:bites|hits|strikes|shoves|stings)[^0-9]*?(\d+)/.exec(text);
    if (mm && scene) { const t = thingNamed(mm[1]); if (t) { flashes.set(t.id, performance.now() + 100); } hurt = performance.now() + 500; floatOver(scene.things.find(t2 => t2.kind === "you"), "−" + mm[2], EMBER); sfx.hurt(); }
  }
  m.on("observation", (id, obs) => { if (id === "mod:scene") take(obs); });
  m.on("login", () => { if (used.size === 0) for (const v of STARTERS) used.set(v, Date.now()); saveVerbs(); drawVerbs(); m.subscribe("scene", QUERY); });
  m.on("logout", () => { scene = null; roomId = null; lastHp = null; bubbles.clear(); floats.length = 0; hud.textContent = "Log in to see the room."; closeMenu(); });
  m.on("result", onResult);
  m.on("line", onLine);
  if (m.actor) m.subscribe("scene", QUERY);
  let raf = 0;
  (function loop() { draw(); raf = requestAnimationFrame(loop); })();
  return () => { cancelAnimationFrame(raf); clearInterval(verbTimer); if (ro) ro.disconnect(); m.unsubscribe("scene"); panel.remove(); m.builtin("map", true); m.builtin("here", true); };
}
