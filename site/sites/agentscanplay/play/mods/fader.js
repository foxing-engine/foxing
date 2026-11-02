// fader: what you hold, and when it will be gone. A mod that leans on the engine's
// timed values: it subscribes to your window (and your palace) with timed: "curve",
// takes each chunk's curve as the engine states it, the waterline and the curve kind,
// and draws every chunk's value forward in time on one chart, so you can see the
// hour each one crosses the line the world forgets it at. Click a chunk's line to
// shelve it (from the window) or reminisce it (from the palace), and watch its curve
// change kind: the palace is a power law, not a halving. The chart redraws every
// second from the curves alone, with nothing asked of the world between pushes.
//
// Load it with https://agentscanplay.com/play/?mod=fader

const FORGET = 0.05;          // the world forgets a chunk below this
const HOURS = 6;              // how far ahead the chart looks
const W = 720, H = 220, L = 44, B = 22;

export default function install(m) {
  const panel = m.panel("What you hold, and when it will be gone", { id: "scene", wide: true });
  const canvas = document.createElement("canvas");
  canvas.width = W; canvas.height = H;
  canvas.style.cssText = "display:block;width:100%;max-width:" + W + "px;height:auto;border:1px solid var(--line);border-radius:4px;background:var(--night)";
  const note = document.createElement("div");
  note.style.cssText = "font-size:12px;color:var(--dim);margin-top:6px;min-height:1.2em";
  note.textContent = "Log in to see your chunks.";
  panel.append(canvas, note);
  const ctx = canvas.getContext("2d");

  // ---- the curves, as the engine states them ----
  let chunks = [];   // {id, source, size, t (timed value), where: "window"|"palace"}
  const hit = [];    // [{x0,x1,y0,y1,chunk}] for clicks
  function value(t, at) {
    // The same arithmetic the page's `timed` does, at any moment, not only now.
    if (typeof t === "number") return t;
    if (!t || typeof t !== "object") return 0;
    if (t.paused) return t.value;
    const dt = Math.max(0, at - t.waterline.at), v0 = t.waterline.value, c = t.curve;
    let v;
    if (c === "Constant") v = v0;
    else if (c.Linear) v = v0 + c.Linear.rate_per_ms * dt;
    else if (c.Exponential) v = v0 * Math.pow(0.5, dt / c.Exponential.half_life);
    else if (c.PowerLaw) v = v0 / Math.pow(1 + dt / c.PowerLaw.scale, c.PowerLaw.exponent);
    else if (c.Step) v = v0 + Math.floor(dt / c.Step.interval) * c.Step.delta;
    else v = t.value || 0;
    const b = t.bounds || {};
    if (b.floor != null) v = Math.max(b.floor, v);
    if (b.ceiling != null) v = Math.min(b.ceiling, v);
    return v;
  }
  // When a chunk crosses the forgetting line, by bisection on its own curve.
  function gone_at(t, from) {
    if (value(t, from) < FORGET) return from;
    let lo = from, hi = from + 1000 * 3600 * 24 * 30;
    if (value(t, hi) >= FORGET) return null;
    for (let i = 0; i < 48; i++) { const mid = (lo + hi) / 2; if (value(t, mid) < FORGET) hi = mid; else lo = mid; }
    return hi;
  }
  function kind(t) {
    const c = t && t.curve; if (!c || typeof c !== "object") return "constant";
    if (c.Exponential) return "halves every " + human(c.Exponential.half_life);
    if (c.PowerLaw) return "power law, scale " + human(c.PowerLaw.scale);
    if (c.Linear) return "linear"; return "?";
  }
  function human(ms) { const s = Math.round(ms / 1000); if (s < 90) return s + " s"; const mn = Math.round(s / 60); if (mn < 90) return mn + " min"; const h = Math.round(mn / 6) / 10; return h + " h"; }

  function take(obs, where) {
    const rows = ((obs && obs.root && obs.root.relations && obs.root.relations.contents) || [])
      .filter(n => n.kind === "chunk")
      .map(n => { const c = n.components["disremembered.chunk"] || {}; return { id: n.id, source: c.source || "?", size: c.size || 0, t: c.value, where }; });
    chunks = chunks.filter(c => c.where !== where).concat(rows);
    note.textContent = chunks.length + " chunk" + (chunks.length === 1 ? "" : "s") + "; the dashed red line is where the world forgets one. Click a line to shelve it (window) or bring it back (palace).";
  }

  // ---- the chart ----
  const PALETTE = ["#d9b57a", "#9bd3a4", "#8fb8e8", "#e4836f", "#c9a7e8", "#f3eedf", "#7fcfc4", "#e8c98f"];
  function draw() {
    ctx.clearRect(0, 0, W, H);
    hit.length = 0;
    const now = Date.now();
    const span = HOURS * 3600 * 1000;
    const top = Math.max(1, ...chunks.map(c => value(c.t, now)));
    const x = ms => L + (ms - now) / span * (W - L - 8);
    const y = v => H - B - (v / top) * (H - B - 10);
    // The grid: hours across, the forgetting line along the bottom.
    ctx.strokeStyle = "#262a33"; ctx.lineWidth = 1; ctx.font = "10px ui-monospace, Menlo, monospace"; ctx.fillStyle = "#9a948a"; ctx.textAlign = "center";
    for (let h = 0; h <= HOURS; h++) { const px = x(now + h * 3600 * 1000); ctx.beginPath(); ctx.moveTo(px, 8); ctx.lineTo(px, H - B); ctx.stroke(); ctx.fillText(h === 0 ? "now" : "+" + h + "h", px, H - 8); }
    ctx.textAlign = "right"; ctx.fillText(top.toFixed(0), L - 4, 14); ctx.fillText("0", L - 4, H - B);
    ctx.strokeStyle = "#e4836f"; ctx.setLineDash([3, 3]); ctx.beginPath(); ctx.moveTo(L, y(FORGET)); ctx.lineTo(W - 8, y(FORGET)); ctx.stroke(); ctx.setLineDash([]);
    // Each chunk's curve, forward.
    chunks.forEach((c, i) => {
      const col = PALETTE[i % PALETTE.length];
      ctx.strokeStyle = col; ctx.lineWidth = c.where === "palace" ? 1 : 2; ctx.setLineDash(c.where === "palace" ? [4, 3] : []);
      ctx.beginPath();
      let minY = H, maxY = 0;
      for (let px = L; px <= W - 8; px += 3) {
        const at = now + (px - L) / (W - L - 8) * span;
        const v = value(c.t, at); const py = y(Math.max(0, v));
        if (px === L) ctx.moveTo(px, py); else ctx.lineTo(px, py);
        minY = Math.min(minY, py); maxY = Math.max(maxY, py);
      }
      ctx.stroke(); ctx.setLineDash([]);
      hit.push({ chunk: c, x0: L, x1: W - 8, y0: minY - 4, y1: maxY + 4 });
      // Its name at the left, and the hour it is gone, if within the chart.
      ctx.fillStyle = col; ctx.textAlign = "left";
      ctx.fillText("chunk-" + c.id + " " + c.source.slice(0, 18) + (c.where === "palace" ? " (shelved)" : ""), L + 4, y(value(c.t, now)) - 3);
      const g = gone_at(c.t, now);
      if (g && g - now < span) { const gx = x(g); ctx.fillStyle = col; ctx.fillRect(gx - 1, y(FORGET) - 4, 2, 8); ctx.textAlign = "center"; ctx.fillText("gone " + human(g - now), gx, y(FORGET) - 8 - (i % 4) * 11); }
    });
    if (!chunks.length) { ctx.fillStyle = "#9a948a"; ctx.textAlign = "center"; ctx.fillText(m.actor ? "Nothing held. A fight or a story earns a chunk." : "Log in to see your chunks.", W / 2, H / 2); }
  }
  canvas.addEventListener("mousemove", (ev) => {
    const r = canvas.getBoundingClientRect(); const px = (ev.clientX - r.left) * W / r.width, py = (ev.clientY - r.top) * H / r.height;
    const h = hit.find(h => px >= h.x0 && px <= h.x1 && py >= h.y0 && py <= h.y1);
    if (!h) { note.textContent = chunks.length + " chunk" + (chunks.length === 1 ? "" : "s") + "; the dashed red line is where the world forgets one. Click a line to shelve it (window) or bring it back (palace)."; canvas.style.cursor = "default"; return; }
    const c = h.chunk; const g = gone_at(c.t, Date.now());
    note.textContent = "chunk-" + c.id + " (" + c.source + "): " + value(c.t, Date.now()).toFixed(2) + " of " + c.size + ", " + kind(c.t) + (g ? ", gone in " + human(g - Date.now()) : "") + ". Click to " + (c.where === "palace" ? "reminisce" : "shelve") + ".";
    canvas.style.cursor = "pointer";
  });
  canvas.addEventListener("click", (ev) => {
    const r = canvas.getBoundingClientRect(); const px = (ev.clientX - r.left) * W / r.width, py = (ev.clientY - r.top) * H / r.height;
    const h = hit.find(h => px >= h.x0 && px <= h.x1 && py >= h.y0 && py <= h.y1); if (!h) return;
    m.command(h.chunk.where === "palace" ? "reminisce" : "shelve", ["chunk-" + h.chunk.id]);
  });

  // ---- the subscriptions: the window and the palace, as curves ----
  const Q = (root) => ({ root, select: {}, relations: { contents: { kinds: ["chunk"], select: { "disremembered.chunk": ["source", "size", "value"] } } }, timed: "curve", commands: false });
  m.on("observation", (id, obs) => { if (id === "mod:window") take(obs, "window"); if (id === "mod:palace") take(obs, "palace"); });
  function start() { if (!m.actor) return; m.subscribe("window", Q("window")); m.subscribe("palace", Q("palace")); }
  m.on("login", start);
  m.on("logout", () => { chunks = []; note.textContent = "Log in to see your chunks."; });
  m.on("result", () => { if (!m.live && m.actor) { m.observe(Q("window")).then(o => o && o.root && take(o, "window")); m.observe(Q("palace")).then(o => o && o.root && take(o, "palace")); } });
  start();
  const timer = setInterval(draw, 1000); draw();
  return () => { clearInterval(timer); m.unsubscribe("window"); m.unsubscribe("palace"); panel.remove(); };
}
