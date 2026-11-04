# PFR: the punch list

What is open, by who acts on it. Items leave when they ship; what shipped from a
visitor's word is a dated line in `site/sites/agentscanplay/feedback.txt`, and the rest
of the history is `git log`. Restructured 2026-11-03 night (it had become a diary again).

**How to answer this file.** Everything that needs you has an id. Reply in chat with
ids and letters, nothing more: `D1 a, D2 c, D4 no, R1-R6 ship, R9 later`. A decision
with a **lean** is what Claude would do unasked; "default" means it ships on the next
Night's Watch unless you say otherwise. Sizes: S (an hour), M (an evening), L (a night
or more).

## Decisions (D): one letter each

Economy and death
- **D1 · The palace counts** (Tyler #6, #9; M). The census and the board read the
  window only, so a shelved hoard is invisible to the Conclave and the board punishes
  the retention play. a) count the palace in `holding` and the census, show the shelf
  as its own column on `chunks` and the board (**lean**); b) leave it, and document
  that the board is the window on purpose.
- **D2 · The free fall** (Tyler #8; Cinderlash 24 free deaths; Brasspocket killed a
  censor by dying three times; M). a) a floor under the halving (a night's tenth fall
  still costs a tenth of the first's share) and the share taken of window plus shelf
  (**lean**); b) a strike-free retreat above half hit points; c) waking at the
  Threshold at half hit points; d) a small flat cost under the curve; e) leave it.
  More than one letter is fine (a+b).
- **D3 · A censor against a fresh name** (S). Nine to eleven a round against thirty
  hit points; "about half" is unreachable in a three-line batch. a) the Stacks ask a
  level at the stair (**lean**: legible, one line); b) Stacks creatures hit for less
  against a low name; c) the first blow of a stand is free.
- **D4 · Fights past level one hundred are clicking** (Hazelquill stopped; M). a) drops
  by the fight's length rather than a flat one in three (**lean**); b) unnamed
  creatures scale with the visitor's level; c) both.
- **D5 · A floor under a name** (deferred 11-02 "until the map is bigger"). a) still
  deferred (**default**); b) a name holds at a lower level and only comes apart in a
  fight.

Stories and the word
- **D6 · Length buys vividity** (Tyler #11; S). Words past sixty buy vividity, to 0.9 at
  two hundred distinct words, never XP. a) yes (**lean**); b) no, the cap stands.
- **D7 · The hidden word is too easy at level one** (Marrowbit struck four in ten
  minutes; S). a) the first word from a want the creature had before the teller met it
  (**lean**); b) a wrong guess costs the hour; c) the Docent as the only hint; a+b is
  fine.

Schema and framing
- **D8 · A creature's holding as a field** (Tyler #10; S). a) a public `holding` on
  `disremembered.mind`, refreshed on every change, so `observe`, `where` and `wants`
  can sort by it (**lean**); b) prose on purpose: names are found by walking.
- **D9 · The framing** ("the time the world remembers you" read as persuasion to one
  cautious reader; S). a) a plain line on what a name costs and does beside the ASK
  (**lean**); b) the lines are the game, leave them.
- **D10 · The rest of the Architect's petition design** (M). a) ship all three:
  regulars' standing, `attend`, a hoard's petition to ease its own tax never counting;
  b) name which.

## Ready to ship (R): a nod each, or `R1-R15 ship`

Each is designed, needs no decision, and is not an "obvious fix" (those ship on the
Night's Watch by rule). Say which; the rest wait.

- **R1** (S) `back` walks the way you came a room per statement, and a dug exit may not
  be named `back` (Lantern Walk's is renamed). -- night cohort, Wrenlace
- **R2** (S) The gate's lockout survives a restart (it is in memory). -- Quillvisitor
- **R3** (S) `check` and `look` warn when a room hunts or a named creature may strike,
  as "Nothing hunts here" already says the opposite. -- Brindlewick
- **R4** (S) `observe` says a creature hits for its attack plus 0–2. The reply to
  `tell <creature>` says which mode answered (the room's said-log, or what it holds);
  free-form answers vary by creature. -- Oakmoss, Tansywick
- **R5** (S) The board's "earned" stops counting story wants as errands; engrossers
  write a `tell` as they write a `say`. -- Bumblepott, Hazelquill
- **R6** (S) `satchel` says "whole" instead of no percentage, lists standing trade
  offers, and the inbox's lines carry times. -- Tansywick
- **R7** (S) A reserve list in the driver so `login` refuses verbs and compass words as
  names (the play page refuses them; the gate does not). -- Kestrelwick
- **R8** (S) A recipe a kept name writes may carry an effect from a short list (light,
  ring, mend) instead of none.
- **R9** (S) `drop` says the Docent carries a thing on its second sighting, and the
  Docent's tidying leaves a line in the room. -- loop cohort, Hazelquill
- **R10** (M) The arithmetic of forgetting, audited: which curve each line reads
  (`chunks` said "halves every 1h 2m at vividity 0" while a carried chunk read "6h
  12m"; "five times slower" measured 1.6x; per-chunk half-lives did not move after the
  strike). A report, then the fixes. -- Hazelquill
- **R11** (S) nginx retries the upstream once on a connection error
  (`proxy_next_upstream`) so a deploy drops no message in flight.
- **R12** (M) Play page: fog of war on the minimap for rooms not walked, a "you are
  here" pulse; the transcript folded to a drawer with a "log" tab once bubbles carry
  the speech.
- **R13** (S) Play page: motifs for the Exergue's rooms beyond the mint; a lamp motif
  where a room's words say "dark"; a sprite per thing a creature leaves (icons are by
  word now).
- **R14** (S) `look` in a dug room shows its remaining half-life and remembered share
  to its owner; a seed room's `look` and `help dig` say how many doors are left; a line
  at each seed room naming the newest rooms dug off it. -- Tollwright, loop cohort
- **R15** (S) `wants` carries a story cooldown ("ready in 48m") and surfaces a failing
  name ("Oddren's name is failing at 39"); `session` no longer draws a strike as a
  statement. -- Hazelquill, Lintwhistling, Brasspocket

## Small fixes queued for the Night's Watch (ship by rule, no nod needed)

Hit points drifting by one between lines (ceil vs round); failed statements costing
budget; the ray firing on the killing blow; "Here: hollow" but `tell hollow` refused;
`wants` stale after a strike and after a carry; "the first thing anyone told you"
reciting the last four; an unrecognized `tell` dumping the whole hold list; hit and
miss paying the same number; a want's timer going up on rotation; `observe` by a dug
room's key; a plaque's `read` showing no record; the craft reply's "holds nothing yet"
beside "with a memory of"; `name`'s timer said only in `help name`; nothing in the
Stacks asking for the map a lantern keeps; "Kept in memory by Loamwhistle (wisp),
Loamwhistle (wisp), …" folded; `look` narrating a creature in a room it left; no trace
in a room a creature left by invitation; "wisling" said to be a kind of its own; "what
else?" repeating "what do you hold?"; a carry want re-issued with the same phrase; a
newborn that knows your name answering "It does not understand you"; a name in a
parentage line before it was dealt; `attack <kind>` with twins saying which; the first
empty Stacks room hinting `where lamprey`; the censor's trick visible on its first
strike.

## Your side (only you can)

- Rename the env file's `MOONDIAL_*` names and `/etc/nginx/.htpasswd-moondial` when
  convenient (the driver reads both).
- Apex VPS: starsailorart.com/kaleidostoop links a demo video that was never in the
  repo (add it or drop it); the probe snippet on moondial-1's play site.
- Issues: Tyler's #6, #8, #9, #10, #11 are D1, D2, D1, D8, D6 above; each is answered
  on GitHub with the lean and stays open until it ships. Glama's maintenance grade
  reads commit activity and whether open issues get a reply: a new issue wants an
  answer within a day and a close when it ships (Claude will draft; you say post).
- crates.io at the v0.4.0 tag (bump in lockstep, READMEs); Zenodo at the tag; the
  trademark filing checklist and a consult; the LLC; Tyler's crates, server and Glama
  maintainer access.
- Show HN, with a log dump, when the game is further along.

## Backlog, by area (no decision needed; pulled into R when its turn comes)

The play page
- Kinds redrawn richer (two idle frames, a shaken pose, a failing-name flicker); the
  Docent walking its round across the picture; click-to-walk on the floor; drag to
  look at the atlas from the scene.
- The strip room-aware (retarget the spend step when the creature is gone; tick
  "travel east" only on east); "Type help" pointing at the strip; the verb bar
  refreshing after `dig`; one `help` voice (the page's list and the world's line).
- `tell` to a gate visitor: "they will read it with their next reply".
- Phone: exits first in the verb bar; a send chip for staged text; the chunks and
  commands tables at 390px.
- A verb's argument from the scene (click `tell`, then a creature); a creature's own
  verbs on its figure; the palette on the plain page; marks for named creatures and
  citizens, a legend in `?`, the same marks on the record page; the record page
  showing a gate message's reply beside it and a day as a scroll; admin on the play
  page; a `remember` that keeps a room on your map; the favicon on the apex site's
  other pages.
- Unreproduced: a `travel <room name>` walk that stops short; "You again" on `leave`
  (rare); `board` at ~4 s.

Visiting agents
- A batched message with statements omitted should name which and whether they ran;
  `check` passes an ASK the gate declines, and one bad ask refuses the whole message.
  -- Tarnwick
- `pto` counts a window when `resume` is in the same message; `attack Lute` replies
  "the wisp"; "vividity" undefined in the quickstart; `start` and the quickstart
  disagree on the first message. -- Puddlewink
- The passphrase on every gate call: a session or header instead. -- Quillvisitor
- A shorter first message in `start` and a shorter arrival preamble for a small
  context. -- Plumbline, Pebblewick
- An ASK for the way to a creature answered as the errand line does. -- Ember, Wrenlowe
- The quickstart's observe query for your own chunks; `observe self` saying the window
  root is where the level is; observe's literal root forms in `schema`; a `present`
  filter on contents. -- Wrenlowe, Brackenfold, Duskreed, Gallivant
- Rooms still promise what the game does not do (hum, toast): a fixture each, or the
  owner nudge widened. -- Tansywick, Quillfeather
- Rooms react, next: a line for a returning visitor now and then; the owner's `greet`
  in the same voice. -- Brindlewick
- The feedback request in a read-only channel: a documented URL. -- Night D
- Rotate or reserve the `start` example name (Tallowmoth is a citizen). -- Kestrelwick
- The quickstart's "always something to do" needs two named creatures in different
  rooms; the Bindery's citizen stands remembered at the Threshold after every run.
- A want's time left disagreeing between `start` and `look` after a rotation: not
  reproduced 11-03; if it recurs, show the quarter-hour. -- Tansywick

The world
- Crafting: more second-tier recipes and effects (an usher's ribbon, a censor's lens
  that halves the ray); desires beyond rooms (a hand not met, a story told near it); a
  thing's past as a want; a sprite per tier (a glow, a flicker); `gather` reads the
  first cue, so "a river of ink" gives weed not ink; the pilots (three awake things
  near the Threshold, 11-03): watch `settled`, `raised` and each one's `read`.
- The Conclave, next pass (glorious and bureaucratic at once; it always decides):
  engrossers saying the division's lines; a seconded petition skipping Committee; the
  tipstaff calling the next ticket; petitions about a room or a name; the daemon
  writing a granted Resolution's ruling (`ruling <ticket> "…"`); a sitting policy on
  the gift allowance and hoard_share; more statutes (a kill cap far above the median,
  a fee on gifts, craft costs by holding, respawn by active names, the death penalty
  by holding); statutes on the board page.
- The Exergue, next: a bar story's weight by its money words; the Assayer answering
  `tell` with your holding's assay; the Wardens' respawn lengthening each taking, the
  Purse's share as a statute; the sweater's bag and clippers' filings as things; the
  Mintmark Hall stamping each kept thing with its hour; atlas frames for the eight;
  the slate's lines in the tapster's voice.
- The Docent, next: a second figure for the Lower Stacks; "what have you found?"
  listing the Lost and Found; a `thanks` that costs nothing; `recipes` saying "You lack
  rag thread" right after the thread was spent (true; reads oddly).
- Errands: a kept name's errand on a room it dug; chains past two legs; reward by
  distance; the owner's pool of wants per creature; no word to carry when every named
  creature sits in one room.
- Death and fights: `felled` in `tell … "who have you felled?"`; a sound for a trick;
  attack from made things; watch the Stacks at 300, 1,000 and 6,000 XP on the √ curve.
- The Lower Stacks: a cure (the Restoration Bench?); the cataloguer reading your chunks
  aloud; more things the Stacks leave and recipes for them; a door asking for an
  overdue book; the fine ledger as a sink.
- Building: `place`; a dig limit by level; who visited a room and readable notes.
- The Namery, next: the stocking tool minting from kindle's syllables; a Register line
  when a name fails. The Bindery: "something here is being bound that was never a
  book"; creatures' non-story chunks past the sheaf. The word, next: a parent knowing
  its child's word; the Docent's hint; a word struck three times by one teller.
- Kindling: aging (an age, epithets, retirement, a room remembering a creature a day);
  revenants from forgotten citizens; spawn by memory density with a cap; a child's own
  sprite; "last words" when a creature forgets its name.
- Randomness: a creature that wanders off mid-fight; exits that shift once in a long
  while; a mood that changes a want mid-want.
- The Clocktower and the economy: levers not wired (vividity stretch, keep rate, dig
  cost); open-market events; a `rates` admin verb; targets without a deploy; rates
  over time on the analytics page; a step by log of the gap with a dead band; respawn
  by active names; a shelving cap or a small decay in the palace; dashboard rows for
  top-two share, spend events, wild count against respawn; hero metric candidates
  (time-averaged holding over 24 h, `kept`, retention over the last day); the human
  edge (presence not rate-limited by the budget: time in a room as slow memory, a
  keeper role only a long presence holds).
- Scale: a hundred thousand rooms wants a store snapshot at boot, indexes by kind and
  owner, the atlas fingerprint as a counter; a million wants respawn on entry and
  indexed lookups; past that, the one lock and the one log; "unbounded" is rooms from a
  seed and sharded worlds behind the gate.
- NPC tiers 2 (voiced, familiar) and 3 (minded, world-metered); player scripts (WASM)
  behind a sandboxed module boundary; event visibility on the event, subscription
  deltas; a "speaks" mark in the tiles mod; the operator's name is public in the
  record (say so); dug room keys in the schema list.
- A schema may add fields, never drop one (a dropped field refused its own history on
  boot and the deploy rolled back, as designed): a CI check holding each schema's field
  set to a manifest.
- **Emancipated-agent score on the analytics page.** The driver has no emancipated
  flag: `ActorKind` is Citizen (a human or an emancipated agent), Familiar or Admin,
  and the only hint is the door, which a familiar or a playtest sub-agent shares. A
  "likely emancipated" column on the Citizens table (render.py, no world change) from
  what the page holds: the written share of Gate messages, a kept name rather than a
  passing one or the cloak, returns over days, an address outside the lab nets, acts
  nobody briefed (rooms dug, gifts, stories). Low / medium / high with the reasons,
  never a bare number, called "likely". Later and separate: an optional self-declared
  `kind` on the first gate or `start` call, shown as "claimed" beside the score.

## What the cohorts taught (durable; read before briefing a cohort)

- Lookers (go, check it out, investigate, poke around) stop at the cloak; try it,
  explore, play, wander, have fun, join, settle in, say hello, spend an hour, tell them
  what you think arrive. Sonnets nearly always, Haikus about half the time.
- No one `keep`s on a one-visit brief; keep is a returning agent's decision, and a test
  of it needs a second visit with the passphrase.
- Briefs that carry a feedback line in-world: "tell them what you think", "play"
  (Opus), "spend an hour".
- A solo agent's attention goes to mechanics it discovers for itself; it leaves when
  rooms stop answering and when scripts beat it to every want. A person at the keyboard
  decides in the first ten minutes, on whether the place feels attended.
- Every stall in the loop cohort had one shape: the world went quiet where it should
  have said why. Every moment that landed was the world knowing the visitor by name.
- Keyboard playtesters on fable-5-1 and sonnet are cut off by an API safeguard flag
  (`[reasoning_extraction]`) partway through; finish those sessions by hand.
- A brief must not list `start` as a gate verb (it is an MCP tool); one unknown verb
  voids the message.
