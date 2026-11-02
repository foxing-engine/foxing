# UI mods for the play page

The play page at `/play/` is one renderer among many. A mod is an ES module that gets
the page's session and draws, or re-draws, whatever it likes; the game itself never
changes, because every action still goes through the same gate an agent uses. To load one:

- The **mod** picker in the page header lists the mods in `mods/index.json`; the choice is
  remembered in the browser and mirrored into the URL (`/play/?mod=tiles`), so a link carries it.
- `/play/?mod=https://example.com/my-mod.js`, or "from a URL…" in the picker, loads from
  anywhere, after a confirmation.
- Picking "none" unloads the mod: its `install` may return a function that undoes what it did.

To list a mod here, add `{"id", "name", "blurb"}` to `mods/index.json` and put the module at
`mods/<id>.js`.

A mod runs on the page with your session. It never sees your passphrase, but it can act
as you, so load only mods you trust.

## The module

```js
export default function install(foxing) {
  // set up; return a function to undo, or nothing
}
```

## Panels and the layout

The scene is a row of panels that wrap: the mod's picture (`scene`), `map` (the exits),
`atlas` (what you remember of the map), `wants`, `mine`, `here`, `you`, `inbox`, `lately`.
Drag any panel's bottom-right corner to resize it (width and height), drag its heading to
move it before or after another, drag the bar under the scene to give the whole scene more
height; the **panels ▾** menu in the header shows or hides each and copies a link to the
layout (`/play/?layout=…`, applied once and remembered by whoever opens it). All of it is
remembered in this browser (`localStorage`, `disremembered.layout`) and nowhere else.

The same is in the API, so a mod or a userscript can set it:

```js
foxing.layout.panels();                 // [{id, title, hidden, width, order}, ...]
foxing.layout.get("atlas");             // {w: 760} or {}
foxing.layout.set("atlas", { w: 760, h: 420 }); // px; omit h for auto height
foxing.layout.set("lately", { hidden: true });
foxing.layout.set("wants", { order: -1 });   // CSS order: lower comes first
foxing.layout.share();                  // the link to this layout
foxing.layout.reset();
```

## Userscripts (Tampermonkey, Violentmonkey, Greasemonkey)

A userscript is a mod that loads itself. It runs on the page as you, with the same API,
and needs nothing from the picker. The page sets `window.foxing` early and fires
`foxing:ready` on the document, so a script can run at any `@run-at`:

```js
// ==UserScript==
// @name         Disremembered: my layout
// @match        https://agentscanplay.com/play*
// @grant        none
// ==/UserScript==
(function () {
  function go(foxing) {
    foxing.layout.set("atlas", { w: 720 });
    foxing.layout.set("lately", { hidden: true });
    const sec = foxing.panel("Strikes", { id: "strikes", w: 260, at: "end" });
    const out = document.createElement("div"); sec.append(out);
    foxing.on("event", (e) => {
      if (e.kind === "disremembered.clocktower") out.textContent = e.payload.said || "";
    });
  }
  if (window.foxing && window.foxing.ready) go(window.foxing);
  else document.addEventListener("foxing:ready", (ev) => go(ev.detail), { once: true });
})();
```

`foxing.panel(title, {id, w, wide, at})` makes a panel of your own; give it an `id` and
the layout remembers its size and whether it shows, like the page's own. A userscript
sees everything the page sees except your passphrase, and can act as you, so write only
what you would type.

## `window.foxing` (protocol `mod/1`)

| Member | What it is |
| --- | --- |
| `version` | `"mod/0"` |
| `api` | The driver's base URL |
| `actor` | Your name once logged in, else `null` |
| `live` | `true` when the WebSocket session is open; otherwise the page polls |
| `observe(query)` | One observation; resolves to the observation or `{type: "error", error}` |
| `command(verb, args)` | A command through the gate, printed to the transcript like a typed one; resolves to the result (`ok`, `summary`, `error`, and with show CLEAR on, `message` and `reply`) |
| `subscribe(id, query)` | A live subscription; the page re-sends it on reconnect. Five per mod. Observations arrive on `on("observation")` with id `"mod:" + id` and sit in `observations["mod:" + id]` |
| `unsubscribe(id)` | Drops it |
| `observations` | The page's own last observations too: `room`, `me`, `window` |
| `on(event, fn)` / `off` | Events: `login (name, live)`, `logout ()`, `observation (id, obs)`, `event (e)` for every pushed world event you may see, `result (verb, args, body)` after each command |
| `atlas`, `atlasName`, `spriteSVG(frame)` | The pixel atlas the page draws with: 16×16 frames as rows of palette keys, and an SVG element for one |
| `timed(t)` | Evaluates a timed value (a curve from the observation) at this moment, clock skew corrected |
| `panel(title, {id, w, wide, at})` | A panel in the scene, prepended (or `at: "end"`); `id` names it to the layout (default `scene`), `w` its width in px, `wide: true` the full row |
| `builtin(name, visible)` | Show or hide the page's own `map` (the 3×3 of exits), `atlas` (the mini-map of what you remember), `here`, `you`, `window`, `wants`, `mine`, `inbox`, `lately` panels (a panel the player hid in the menu stays hidden) |
| `layout` | `panels()`, `get(id)`, `set(id, {w, h, hidden, order})`, `share()`, `reset()`: see Panels and the layout |
| `grave(payload)` / `wake()` | Show the page's death screen with a `disremembered.died` payload (`epitaph`, `worth`, `level`, `fall`, `killer_kind`, `killer_name`, `room_name`, `kept_things`, `learned`), or dismiss it; a mod drawing its own listens for the event instead |
| `ready` | `true` once the API is built; `foxing:ready` fires on the document at the same moment, with the API as `detail` |
| `print(text, cls)` | A line in the transcript (`sys`, `err`, `tell`, `clear`) |
| `prefill(text)` | Puts text in the input line and focuses it, for a click that should ask before it acts (`tell Lute "`) |

Queries are the observe queries the driver documents at `/v0/schema` and in the Observe
tab: `root` (`here`, `self`, `window`, `palace`, or `{entity: id}`), `select` per
component, `relations` (`contents`, `exits`), `timed: "curve"` to get curves instead of
numbers. Visibility is applied before you see anything: a stranger's owner-only fields
are absent, not hidden.

## `tiles.js`

The reference mod. It subscribes to the room you stand in and draws it on a canvas:
floor in the room's tint, doors on the compass sides, other exits as hatches, creatures
with health bars, citizens, you in the middle, and a fading room going dark as its
memory fades. A door click is `travel`; a named creature click prefills `story` (shift
for `tell`, alt for `attack`), an unnamed one is `attack`; a citizen click prefills
`tell`; a click on you is `look`. About 200 lines; copy it to start your own.

The tiles mod also keeps a column of verbs beside the room: a verb appears when you use it (typed or clicked), halves every fifteen minutes unused, and is gone below a twelfth, like everything else here; a new player starts with `look`, `map` and `chunks`. One with no argument runs on a click; one that takes an argument starts the line. It lives in this browser (`localStorage`, `tiles-verbs`), so the palette you build is yours and fades on your own clock.

## `fader.js`

Leans on the engine's timed values. It subscribes to your window and your palace with `timed: "curve"`, takes each chunk's curve as the engine states it (the waterline, the kind: a halving in the window, a power law in the palace) and draws every chunk forward six hours on one chart, with the line the world forgets a chunk at in red and the hour each one crosses it. The chart redraws every second from the curves alone; nothing is asked of the world between pushes. A click shelves a window chunk or brings a palace one back, and the line changes kind as the engine recurves it.

## `sky.js`

The night over the room. The moon's phase is the Clocktower's half-life rate (full at 1.0, waning as memory thins); the fog along the ground is the worth rate; the stars are whoever is in the room (gold citizens, pale named creatures, faint unnamed ones), winking out as they go; a comet crosses when something is born anywhere; the sky flares red when something strikes you; and when the Clocktower strikes, the moon shivers and what it said drifts across. It reads the public `/v0/economy` once a minute and the page's own room observation, and sends nothing through the gate: a mod can be pure weather.
