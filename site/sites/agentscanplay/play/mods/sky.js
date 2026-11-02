// sky: the night over the room. A strip of sky drawn from what the world is doing:
// the moon is the Clocktower's half-life rate (full at 1.0, waning as memory thins,
// waxing as it keeps), the fog is the worth rate (thick when the creatures are worth
// more), the stars are whoever is in the room with you (gold for citizens, pale for
// named creatures, faint for the unnamed), a shooting star when something is born
// anywhere, a red flare when something strikes you, a star winking out when a memory
// fades, and when the Clocktower strikes, the moon shivers and what it said drifts
// across the sky. Reads the public /v0/economy for the rates once a minute and the
// room from the page's own observation; nothing is asked of the world through the gate.
//
// Load it with https://agentscanplay.com/play/?mod=sky

const W = 720, H = 120;

export default function install(m) {
  const panel = m.panel("The sky", { id: "scene", wide: true });
  const canvas = document.createElement("canvas");
  canvas.width = W; canvas.height = H;
  canvas.style.cssText = "display:block;width:100%;max-width:" + W + "px;height:auto;border:1px solid var(--line);border-radius:4px;background:#07090f";
  canvas.setAttribute("role", "img");
  canvas.setAttribute("aria-label", "The sky over the room");
  panel.append(canvas);
  const ctx = canvas.getContext("2d");

  let rates = { half_life: 1, worth: 1, pay: 1 };
  let said = null, saidAt = 0;        // the Clocktower's line, drifting
  let shiver = 0, flare = 0;          // moments, ms
  const comets = [];                  // {at, y}
  const winks = [];                   // {at, x, y}
  const stars = new Map();            // id -> {x, y, kind, name}
  function hash(s) { let h = 2166136261; for (const ch of String(s)) { h ^= ch.charCodeAt(0); h = Math.imul(h, 16777619); } return h >>> 0; }

  async function readRates() {
    try { const r = await fetch(m.api + "/v0/economy"); if (r.ok) { const d = await r.json(); rates = d.rates || rates; } } catch (e) {}
  }
  readRates(); const ratesTimer = setInterval(readRates, 60000);

  // Who is here, from the page's own room observation: a star each, placed by id.
  function takeRoom(obs) {
    const here = (obs && obs.root && obs.root.relations && obs.root.relations.contents) || [];
    const seen = new Set();
    for (const n of here) {
      if (n.kind !== "citizen" && n.kind !== "creature") continue;
      const present = n.kind === "citizen" ? (n.components["disremembered.presence"] || {}).here !== false : true;
      if (!present) continue;
      seen.add(n.id);
      if (!stars.has(n.id)) {
        const h = hash(n.id);
        const named = n.kind === "citizen" || (n.components["disremembered.mind"] || {}).name;
        stars.set(n.id, { x: 60 + (h % 600), y: 12 + ((h >> 8) % 70), kind: n.kind === "citizen" ? "citizen" : named ? "named" : "unnamed", born: Date.now() });
      }
    }
    for (const id of [...stars.keys()]) if (!seen.has(id)) { const s = stars.get(id); winks.push({ at: Date.now(), x: s.x, y: s.y }); stars.delete(id); }
  }
  m.on("observation", (id, obs) => { if (id === "room") takeRoom(obs); });
  if (m.observations && m.observations.room) takeRoom(m.observations.room);
  m.on("event", (e) => {
    const k = e.kind, p = e.payload || {};
    if (k === "disremembered.born") comets.push({ at: Date.now(), y: 20 + Math.random() * 40 });
    if (k === "disremembered.clocktower") { said = p.said || "The Clocktower strikes."; saidAt = Date.now(); shiver = Date.now(); readRates(); }
    if (k === "disremembered.faded" && p.owner === m.actor) winks.push({ at: Date.now(), x: 80 + Math.random() * 560, y: 20 + Math.random() * 60 });
    if (k === "disremembered.narrative" && p.to === m.actor && /hits you for|comes at you|takes hold/.test(p.text || "")) flare = Date.now();
  });
  m.on("logout", () => { stars.clear(); });

  function draw() {
    const now = Date.now();
    // The night, and the fog along the ground: thicker the more the creatures are worth.
    const g = ctx.createLinearGradient(0, 0, 0, H); g.addColorStop(0, "#05070c"); g.addColorStop(1, "#0f1420");
    ctx.fillStyle = g; ctx.fillRect(0, 0, W, H);
    const fog = Math.max(0, Math.min(1, (rates.worth - 0.5) / 1.5));
    for (let i = 0; i < 5; i++) {
      ctx.fillStyle = "rgba(170,180,200," + (0.04 + fog * 0.12) + ")";
      const yy = H - 10 - i * 7 + Math.sin(now / 4000 + i) * 2;
      ctx.beginPath(); ctx.ellipse(W / 2 + Math.sin(now / 9000 + i * 2) * 120, yy, 420 + i * 60, 10 + fog * 14, 0, 0, Math.PI * 2); ctx.fill();
    }
    // The moon: its phase is the half-life rate. 1.0 is full; thinner as memory thins.
    const phase = Math.max(0.05, Math.min(1, rates.half_life));
    const mx = W - 70 + (now - shiver < 1200 ? Math.sin(now / 30) * 2 : 0), my = 34, r = 16;
    ctx.fillStyle = "#e8e4da"; ctx.beginPath(); ctx.arc(mx, my, r, 0, Math.PI * 2); ctx.fill();
    if (phase < 1) {
      // A shadow slides across from the right as the moon wanes.
      ctx.fillStyle = "#07090f"; ctx.beginPath();
      const k = 1 - phase;                       // 0 full, 1 dark
      ctx.ellipse(mx + r * k * 1.2, my, r * (1 - k * 0.6), r, 0, 0, Math.PI * 2); ctx.fill();
    }
    if (rates.half_life > 1.001) { ctx.strokeStyle = "rgba(232,228,218,.35)"; ctx.beginPath(); ctx.arc(mx, my, r + 4 + (rates.half_life - 1) * 10, 0, Math.PI * 2); ctx.stroke(); }
    // Stars: who is here. They twinkle; a new one brightens in.
    for (const s of stars.values()) {
      const age = Math.min(1, (now - s.born) / 1500);
      const tw = 0.7 + 0.3 * Math.sin(now / 700 + s.x);
      ctx.fillStyle = s.kind === "citizen" ? "rgba(217,181,122," + (age * tw) + ")" : s.kind === "named" ? "rgba(232,228,218," + (age * tw) + ")" : "rgba(154,148,138," + (age * tw * 0.6) + ")";
      const sz = s.kind === "unnamed" ? 1.6 : 3;
      ctx.beginPath(); ctx.arc(s.x, s.y, sz, 0, Math.PI * 2); ctx.fill();
      if (s.kind !== "unnamed") { ctx.beginPath(); ctx.moveTo(s.x - 9, s.y); ctx.lineTo(s.x + 9, s.y); ctx.moveTo(s.x, s.y - 9); ctx.lineTo(s.x, s.y + 9); ctx.strokeStyle = ctx.fillStyle; ctx.lineWidth = 1; ctx.stroke(); }
    }
    // Winks: a star going out.
    for (let i = winks.length - 1; i >= 0; i--) {
      const w = winks[i], t = (now - w.at) / 1200; if (t > 1) { winks.splice(i, 1); continue; }
      ctx.strokeStyle = "rgba(232,228,218," + (1 - t) + ")"; ctx.beginPath(); ctx.arc(w.x, w.y, 2 + t * 10, 0, Math.PI * 2); ctx.stroke();
    }
    // Comets: a birth, anywhere.
    for (let i = comets.length - 1; i >= 0; i--) {
      const c = comets[i], t = (now - c.at) / 1800; if (t > 1) { comets.splice(i, 1); continue; }
      const x = t * (W + 100) - 50, y = c.y + t * 30;
      const cg = ctx.createLinearGradient(x - 60, y - 8, x, y); cg.addColorStop(0, "rgba(201,167,232,0)"); cg.addColorStop(1, "rgba(201,167,232,.9)");
      ctx.strokeStyle = cg; ctx.lineWidth = 2; ctx.beginPath(); ctx.moveTo(x - 60, y - 8); ctx.lineTo(x, y); ctx.stroke();
    }
    // A flare when something strikes you.
    if (now - flare < 600) { ctx.fillStyle = "rgba(228,131,111," + (0.35 * (1 - (now - flare) / 600)) + ")"; ctx.fillRect(0, 0, W, H); }
    // What the Clocktower said, drifting across for a minute.
    if (said && now - saidAt < 60000) {
      const t = (now - saidAt) / 60000;
      ctx.font = "italic 12px Iowan Old Style, Palatino, Georgia, serif"; ctx.fillStyle = "rgba(217,181,122," + (t < 0.9 ? 0.9 : 0.9 * (1 - (t - 0.9) / 0.1)) + ")"; ctx.textAlign = "left";
      ctx.fillText(said, W - t * (W + ctx.measureText(said).width + 40), 100);
    }
    // The legend, small, bottom left.
    ctx.font = "10px ui-monospace, Menlo, monospace"; ctx.fillStyle = "rgba(154,148,138,.8)"; ctx.textAlign = "left";
    ctx.fillText("moon: memory " + (rates.half_life < 0.999 ? "thins" : rates.half_life > 1.001 ? "keeps" : "holds") + " · fog: worth " + rates.worth.toFixed(2) + " · stars: who is here", 8, H - 6);
  }
  let raf = 0;
  (function loop() { draw(); raf = requestAnimationFrame(loop); })();
  return () => { cancelAnimationFrame(raf); clearInterval(ratesTimer); panel.remove(); };
}
