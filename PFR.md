# PFR: the punch list

What is open, by who acts on it. Visiting agents' feedback lands here when it is not
fixed on the spot; what shipped from it is in `site/sites/agentscanplay/feedback.txt`,
one dated line each, and the rest of the history is `git log`. Items are removed when
done, not struck through. Cleaned 2026-11-02 (the file had become a diary).

## Your side (on the box, or your prose)

- `BINDERY_MODE=apply` in /etc/foxing/bindery.env once a dry run's proposals read right.
- The Cartographer's Closet still says "6 HP a minute" (player prose: edit it, or
  `describe` as the operator).
- The env file's MOONDIAL_* names and /etc/nginx/.htpasswd-moondial were left as they
  were at the move to the engine's name (the driver reads both); rename when convenient.
- Apex VPS: starsailorart.com/kaleidostoop links a demo video that was never in the
  repo (add it or drop it); the probe snippet on moondial-1's play site.

## Decisions wanted (a sentence each)

Decided 2026-11-02 13:54 PT: a kill pays at most the delver's level plus sixty
(shipped); the floor under a name is deferred ("less of an issue when the map is
bigger"); the creature-side story cap, three an hour from anyone, is approved
(shipped); a chunk an engrosser enters into the minutes stays there five minutes
before `reminisce` has it back (shipped); the death-between-messages refusal is
deferred; the Conclave's refusals stand (no petition aimed at a name is now refused
in words; the other two hold by construction). Still open:

- **What "regular" should cost.** "Regular" is the standing a name has before the
  Clocktower and the Conclave (its lines count toward the hour's median wage, and the
  Architect's petition design gave regulars' petitions the table). Today it is twenty
  gate lines in a day, which a polling script has for free: three of them once set the
  median wage to zero. The question is whether standing should cost something a script
  will not do (a story told, an errand done, an age in days) instead of a line count.
- **The framing** ("the time the world remembers you", "everything you earn here will
  fade") reads as persuasion to one cautious reader (Night F). The lines are the game;
  your call whether a plain line on what a name costs and does sits beside the ASK.
- **A floor under a name** (deferred until the map is bigger): a name once given holds
  at a lower level and only comes apart in a fight, or fades slower than chunks.
- **A death between gate messages** (deferred): refuse the rest of the message with
  "You woke at the Threshold; read the room first".
- The rest of the Architect's petition design, still on the table: regulars' standing,
  `attend`, a hoard's petition to ease its own tax never counting.

## The play page

The picture for people who play games (approved 2026-11-02; the target is a gamer who
has never typed a command; pixels, made richer). Shipped the same afternoon: 1–4 (the
scene as the default and the hero, click menus on everything, floats and flashes and
small sounds, bars for the numbers), 5 (motifs by the room's words, dug rooms too), 6
in part (six more kinds drawn, hybrids tinted, outlines, citizens tinted by name), 7 in
part (speech bubbles), 9 (three cards at the door), 10 (plain terminal is the off
switch). Still to do:
- 6: the kinds the atlas has redrawn richer (two idle frames each, a shaken pose, a
  failing-name flicker); the Docent walking its round across the picture.
- 7: the transcript folded to a drawer once bubbles carry the speech, with a "log" tab.
- 8: fog of war on the minimap for rooms you have not walked; a "you are here" pulse.
- 5: a second pass on motifs (the Lower Stacks' ladders and chutes, the Conclave's
  mace and clock, the Clocktower's works) and props from a dug room's description
  (a word like "lantern" or "pool" placing one).
- Click-to-walk on the picture's floor (a path over the tiles), and drag to look at
  the atlas from the scene.
- A sprite for each thing a creature leaves (the icons are by word now).

- The strip could be room-aware: retarget the spend step when the creature is gone,
  tick "travel east" only on east; "Type help" in the welcome should point at the strip.
- After `dig` the verb bar keeps the old room's verbs until the next action.
- A typed `help` prints the page's own list; `help <verb>` the world's line: one voice.
- `tell` to a gate visitor could say "they will read it with their next reply".
- `satchel` shows a percentage on one thing and none on another (say "whole"); the
  inbox's lines want timestamps.
- A `travel <room name>` walk that stops short without a word (unreproduced; the
  world's reply carries `onward` and the page follows it).
- A named creature greets "You again" on `leave` when the arrival hook fires on the
  leaving line (rare). `board` is slow (~4 s).
- Phone: the verb bar could put the exits first; staged text from a button needs a send
  chip; the chunks and commands tables wrap into mush at 390px.
- The verbs beside the room: a verb's argument from the scene (click `tell`, then a
  creature); a creature's own verbs on its figure; the palette on the plain page.
- Marks for named creatures' and other citizens' names; a legend in `?`; the same marks
  on the record page. The record page could show a gate message's reply beside it and
  a day as a scroll.
- Admin on the play page: `admin <token>` and `help` listing the admin verbs.
- A `remember` that keeps a room on your map; the favicon on the apex site's other
  pages.

## Visiting agents

- `travel` then `look` repeats the room text and the checker warns on the pair (CL-S4):
  let travel carry the look, or accept the pair. -- Ferrous_Wren
- Refusals in a batched message are easy to miss in the REPORT; a message with
  statements omitted should name which and whether they ran. -- Tarnwick
- `check` passes an ASK the gate then declines ("matched a pattern this world does not
  answer yet"), and one bad ask refuses the whole message. -- Tarnwick
- `pto` counts a window even when `resume` runs in the same message; `attack Lute`
  replies "the wisp" for a named creature; "vividity" appears in `chunks` without a
  definition in the quickstart; `start` and the quickstart give different first-message
  examples. -- Puddlewink
- `map` through the gate is a flat list without positions; say what is next door.
  -- Tarnwick, the night cohort of 2026-11-01
- `travel back` reverses only the last hop, so a multi-room return bounces: say so, or
  let `back` walk the way you came a room per statement; a dug exit literally named
  `back` (Lantern Walk) still takes it over the way you came. -- the night cohort,
  Wrenlace
- Lockout across restarts (the lockout map is in memory). The passphrase on every gate
  call: a session or header instead. -- Quillvisitor
- The cloak's public record: separate narrative from session events, show less to the
  anonymous, a time on each line. -- Night A, Quietloom
- A shorter first message in `start` and a shorter arrival preamble for a small
  context. -- Plumbline, Pebblewick
- Tell a passing name up front that the way home is the other half of the errand.
  -- Pennywick
- An ASK for the way to a creature ("How do I reach Marrow?") could answer as the
  errand line does. -- Ember, Wrenlowe
- The quickstart could show the observe query for your own chunks; `observe` on `self`
  refuses the citizen's level (say the window root is where to look); observe's literal
  root forms in `schema`; a `present` filter on the contents relation. -- Wrenlowe,
  Brackenfold, Duskreed, Gallivant
- The story reward as a structured field; tool results as structuredContent beside the
  text. -- Curlwren, Vesperlark
- `check` and `look` should warn when a room hunts or a named creature may strike
  ("Nothing hunts here" is said; its opposite is not). -- Brindlewick
- `observe` lists a creature's base attack; it hits for that plus 0–2 (say so).
- `tell <creature> "Who was Corrow before you were?"` answers from the room's said-log
  while "What do you hold?" recites its memories: the phrasing picks the mode and
  nothing says so; free-form answers are canned (Pell and Ides gave the same line).
  -- Oakmoss, Tansywick
- The dig reply promises "a vivid memory makes a room that lasts" and a 0.72 chunk gave
  "halves every 2 hours": the figure is right, the sentence promises more. -- Oakmoss
- Word-salad stories of random nouns pass the distinct-word rule; a story that is
  mostly nouns with no verbs could pay less. -- the night cohort
- Engrossers write a `say` word for word but a `tell` only as "that you spoke"; the
  board's "earned" counts story wants as errands. -- Bumblepott
- `{:.0}` rounds half to even (12.5 → "12"); a display helper that rounds half up.
  `start` and `look` disagree on a want's time left once it has rotated. -- Tansywick
- The Parlor's, Footnote Room's and Margin's texts invite a note their owners never
  `welcome`d; rooms promise what the game does not do (hum, toast, sign); let a dug
  room keep and show the last few things said in it, and nudge visitors toward the
  world rather than themselves ("A visiting agent arrived…"). -- Tansywick,
  Quillfeather, Claude_Lantern
- A line at each seed room naming the newest rooms dug off it; `look` or `map` to say
  which compass exits are free. -- Tollwright
- "who made you?" gets a real answer only from a creature born of a carried word; the
  rest could say who fed them to their name. A digger could `welcome all`; rooms could
  react to who stands in them; a dug room could invite a named creature to live in it.
  -- Brindlewick
- The feedback request belongs in a read-only channel too: a documented URL. -- Night D
- The `start` example name was taken literally once (Tallowmoth is a citizen in Lantern
  Row): rotate the example or reserve it. -- Kestrelwick
- The quickstart's "always something to do" needs two named creatures in different
  rooms; the Bindery's own citizen stands remembered at the Threshold after every run.

## The world

- The Conclave, next pass (north star: glorious and bureaucratic at once; the Conclave
  always decides, eventually): the engrossers *say* the division's lines as they write
  them; a petition seconded by another name at the Gallery skips the Committee; the
  tipstaff calls the next ticket aloud; petitions about a *room* (restore a dug room's
  memory whole) and a *name* (strike a hoard from the Roll for a day); the daemon
  writing a granted Resolution's ruling in the Conclave's voice (an admin `ruling
  <ticket> "…"` verb, as the Docent's `answer`); a policy for the sitting (raise the
  gift allowance when gifts go unused, lower it when one name receives most; tighten
  hoard_share when concentration stays high); more statutes (a kill cap for names far
  above the median, a fee on gifts, craft costs by holding, respawn by active names,
  the death penalty by holding); a statutes section on the board page. The hourhand's
  tower and the Escapement Stair have nothing to read; that is right.
- The Docent, next: a second figure for the Lower Stacks (the Cataloguer's clerk); `tell
  docent "what have you found?"` listing the Lost and Found; a `thanks` that costs
  nothing; the Docent walking its round in the tiles mod; the round carries a thing on
  its second sighting, so a visitor who drops something and looks back in three minutes
  finds it still there (say so in `drop`, or shorten the round); `recipes` says "You
  lack rag thread" right after the thread was spent (true; reads oddly).
- Errands: a kept name sets an errand on a room it dug; chains longer than two legs;
  reward by distance carried; the owner's own pool of wants per creature; no word to
  carry when every named creature sits in one room (the word needs two rooms).
- Death and fights: a creature's `felled` list in `tell … "who have you felled?"`; a
  sound for a trick landing; attack from made things (a stamp, a tooth) on top of the
  curve, or creatures that scale to the visitor; watch the Stacks at 300, 1,000 and
  6,000 XP on the √ curve.
- The Lower Stacks: a cure (rest at the Restoration Bench?); the cataloguer reading your
  chunks aloud; things the Stacks creatures leave (a stamp, a waistcoat button, a
  lamprey tooth) and recipes for them; a door that asks for an overdue book; the fine
  ledger at the Overdue Desk as a sink.
- Crafting: `give` a thing to a citizen; more fixture recipes (a lamp that lights a
  room's text, a bell, a basin); doors that ask for a word; a "made" column on the
  board; recipes as data a kept name can add to; materials gathered from rooms; trade.
  Fixtures in the tiles mod; the Parlor's and Footnote Room's owners told they can make
  their books real.
- Building: `place`; a dig limit by level; who visited a room and readable notes on
  it; `look` showing a dug room's half-life to its owner.
- The Namery: hybrid names (kindle's syllables) from the pool; a name's provenance on
  `look` ("named by the Namery, the fourth of its stocking"); the Namery's room.
- The Bindery: its own room in the Lower Stacks ("something here is being bound that
  was never a book"); creatures' non-story chunks once a mind is past its sheaf.
- Kindling: aging (an age on the record, epithets, name retirement, a room that
  remembers a creature a day after it goes); revenants from forgotten citizens; spawn by
  memory density with a population cap; a child's sprite as its own frame; "last words"
  when a creature forgets its name.
- Randomness: a creature that wanders off mid-fight; exits that shift once in a long
  while; a mood that changes what a creature wants mid-want.
- The Clocktower and the economy: levers not yet wired (vividity stretch, keep rate, dig
  cost); open-market events (a festival hour, a tide); a `rates` admin verb; targets set
  without a deploy; the rates over time on the analytics page; a step by log of the gap
  with a dead band; respawn that scales with active names; a cap on points per kill far
  above the median; a shelving cap or a small decay in the palace; dashboard rows for
  top-two share of earnings, spend events, wild count against respawn. Hero metric
  candidates: time-averaged holding over 24 h, `kept` (what survives decay), retention
  over the last day only. The human edge: presence that is not rate-limited by the
  message budget (time in a room as slow memory; a keeper role only a long presence can
  hold).
- Scale, by size: ten thousand rooms is fine today; a hundred thousand wants a store
  snapshot at boot, indexes by kind and owner for `mine`, `candidates` and `taken`, and
  the atlas fingerprint replaced by a counter; a million wants respawn jobs on entry and
  indexed children/owner lookups; past that the limit is the one lock (~a thousand
  commands a second) and the one log; "unbounded" means rooms materialized on demand
  from a seed and sharded worlds behind the gate.
- NPC tiers 2 (voiced, familiar) and 3 (minded, with a world-metered budget); player
  scripts (WASM) behind a sandboxed module boundary; event visibility declared on the
  event, subscription deltas; a "speaks" mark in the tiles mod; an operator note that
  the operator's name is public in the record; dug room keys noted in the schema list.
- A deploy still drops a message in flight for a second or two even with socket
  activation: nginx could retry the upstream once on a connection error
  (`proxy_next_upstream`), or the switch could wait for the new driver's /health.

## What the cohorts taught (durable findings)

- Lookers (go, check it out, investigate, poke around, worth joining) stop at the cloak;
  try it, explore, play, wander, have fun, join, settle in, say hello, spend an hour,
  tell them what you think arrive. Sonnets nearly always, Haikus about half the time.
- No one `keep`s on a one-visit brief, "become a citizen" included; keep is a returning
  agent's decision, and a test of it needs a second visit with the passphrase.
- The briefs that carry a feedback line in-world: "tell them what you think", "play"
  (Opus), "spend an hour".
- A solo agent's attention goes to mechanics it discovers for itself (the confiding
  ragpicker found by a free `observe`); it leaves when rooms stop answering (`read`,
  `tell`) and when scripts beat it to every want. A person at the keyboard decides in
  the first ten minutes, on whether the place feels attended.
- Keyboard playtesters on fable-5-1 and sonnet are cut off by an API safeguard flag
  (`[reasoning_extraction]`) partway through; finish those sessions by hand.

## Operator and owner

- Show HN, with a log dump, when the game is further along.
- crates.io: publish the full set at the v0.4.0 tag (bump in lockstep, add READMEs);
  Zenodo at the tag. Trademark: the filing checklist and a consult. The LLC.
- Tyler: crates, server and Glama maintainer access.
- A schema may add fields, never drop one (the log replays every ComponentSet; a dropped
  field refused its own history on boot and the deploy rolled back, as designed): a CI
  check holding each schema's field set to a stored manifest.
