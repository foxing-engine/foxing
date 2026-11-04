# PFR: the punch list

What is open, by who acts on it. Items leave when they ship; what shipped from a
visitor's word is a dated line in `site/sites/agentscanplay/feedback.txt`, and the rest
of the history is `git log`. Restructured 2026-11-03 night (it had become a diary again).

**How to answer this file.** Everything has an id; reply in chat with ids and letters,
nothing more: `D4 a, D10 b, R12 later, B7 next, X6 brief`. Sizes: S (an hour), M (an evening),
L (a night or more). A **lean** is what Claude would do unasked.

**Standing policy (set 2026-11-03 22:58 PT).** Every S-sized decision with a lean ships
on the next Night's Watch unless you object first; a decision that came from one of
Tyler's issues ships at S or M. Every R ships on the Night's Watch, in whatever batches
Claude judges safe (a deploy a batch, tests green, the risky one alone). What is left
for you is marked **waits**: the M and L decisions not from an issue, and anything
where the lean is a coin flip. Nothing ships silently: the morning report lists what
went by policy, with ids.

## Decisions (D): one letter each; **waits** means yours, the rest go by policy

Economy and death
- **D1 · The palace counts** (Tyler #6, #9; M; **ships by policy**, a). The census and the board read the
  window only, so a shelved hoard is invisible to the Conclave and the board punishes
  the retention play. a) count the palace in `holding` and the census, show the shelf
  as its own column on `chunks` and the board (**lean**); b) leave it, and document
  that the board is the window on purpose.
- **D2 · The free fall** (Tyler #8; decided 11-03 23:39 PT, shipped the same night): a
  hit chance (nine in ten) with a shaken wake whose penalty falls on a parabola, five
  minutes a fall of the night up to thirty; creatures keep their wounds and mend on a
  Clocktower rate (`mend`), named ones walking to succor and home. Still open from
  the same issue, **ships by policy**: the floor under the halving and the share
  taken of window plus shelf (a).
- **D3 · A censor against a fresh name** (S; **ships by policy**, a). Nine to eleven a round against thirty
  hit points; "about half" is unreachable in a three-line batch. a) the Stacks ask a
  level at the stair (**lean**: legible, one line); b) Stacks creatures hit for less
  against a low name; c) the first blow of a stand is free.
- **D4 · Fights past level one hundred are clicking** (Hazelquill stopped; M; **waits**). a) drops
  by the fight's length rather than a flat one in three (**lean**); b) unnamed
  creatures scale with the visitor's level; c) both.
- **D5 · A floor under a name** (deferred 11-02 "until the map is bigger"). a) still
  deferred (**default**); b) a name holds at a lower level and only comes apart in a
  fight.

Stories and the word
- **D6 · Length buys vividity** (Tyler #11; S; **ships by policy**, a). Words past sixty buy vividity, to 0.9 at
  two hundred distinct words, never XP. a) yes (**lean**); b) no, the cap stands.
- **D7 · The hidden word is too easy at level one** (Marrowbit struck four in ten
  minutes; S; **ships by policy**, a). a) the first word from a want the creature had before the teller met it
  (**lean**); b) a wrong guess costs the hour; c) the Docent as the only hint; a+b is
  fine.

Schema and framing
- **D8 · A creature's holding as a field** (Tyler #10; S; **ships by policy**, a). a) a public `holding` on
  `disremembered.mind`, refreshed on every change, so `observe`, `where` and `wants`
  can sort by it (**lean**); b) prose on purpose: names are found by walking.
- **D9 · The framing** ("the time the world remembers you" read as persuasion to one
  cautious reader; S; **ships by policy**, a). a) a plain line on what a name costs and does beside the ASK
  (**lean**); b) the lines are the game, leave them.
- **D10 · The rest of the Architect's petition design** (M; **waits**). a) ship all three:
  regulars' standing, `attend`, a hoard's petition to ease its own tax never counting;
  b) name which.

## Ready to ship (R): all ship on the Night's Watch by policy, in Claude's batches

Each is designed and needs no decision. Say `R<n> hold` to keep one back.

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

Away from Moondial and Entropic (the engine's and the game's old names; 184 mentions
in the tree on 11-03, most of them fallbacks that stay for a while):
- **R16** (S) Code and tool docs say `FOXING_*` (world.rs, net.rs, presence.rs, mod.rs,
  the postgres crate's comments, tools/*.py docstrings); the `MOONDIAL_*` fallbacks
  stay in code until R20 is done on the box. The test fixture's `entropic.room` and
  its `MOONDIAL_WORLD` message read `FOXING_WORLD`.
- **R17** (S) site/analytics: defaults to `/srv/foxing`, `/etc/foxing/driver.env`,
  `FOXING_HTTP_URL`, `FOXING_LAB_NETS`, `FOXING_OPS_LOG`, the "no admin token" line;
  the old names read second.
- **R18** (S) CI: the PostgreSQL service user, password and database named `foxing`;
  `FOXING_TEST_PG`. deploy-driver.yml drops the `moondial-driver` symlink and the
  `/srv/moondial` fallback (the host migrated 11-01).
- **R19** (M) deploy/server: `setup-moondial-1.sh` becomes `setup-server.sh` writing
  `FOXING_*` to `/etc/foxing/driver.env`, `foxing.service` "(Disremembered)", sudoers
  `foxing`, `.htpasswd-foxing`; harden.sh, setup-postgres.sh, enable-*.sh,
  setup-bindery.sh and setup-analytics.sh read the new names first; the old-path
  fallbacks go after R20. `migrate-to-foxing.sh` stays as history until then.
- **R20** (S, then Y1) `deploy/server/rename-env.sh`: on the box, renames the
  `MOONDIAL_*` keys in `/etc/foxing/driver.env` to `FOXING_*`, the unit's Description
  to "Foxing driver (Disremembered)", `.htpasswd-moondial` to `.htpasswd-foxing` with
  the nginx snippet, and restarts; idempotent; prints its diff first. Claude writes it,
  you run it (read line by line, as always).
- Kept on purpose: `nginx-moondial.conf` (moondial.ai redirects to foxing.dev), the
  `/clear/entropic.yaml` redirect, the postgres schema-lock constant, the host's name
  moondial-1, TRADEMARK.md's history line, the repo's local folder name.

## XL (X): the scaffolding; each wants a brief before a first cut

Nothing here ships by policy. Say `X3 brief` and Claude writes the design as a short
doc for you to mark up; say `X3 go` on a brief and the first cut is a night's work.

- **X1 · Economy v2.** The Clocktower's levers not yet wired (vividity stretch, keep
  rate, dig cost), open-market events (a festival hour, a tide), targets without a
  deploy, a dead band on the step, the palace counted everywhere (D1), the fall priced
  on what was risked (D2), and the hero metric chosen (time-averaged holding, `kept`,
  or retention). First cut: the levers and the metric, with the rates charted on the
  analytics page.
- **X2 · Crafting v3.** Recipes with effects a name chooses; desires beyond rooms; a
  thing's past as a want; things that act on their own (a bell that rings for a name
  it knows); the awake tier with a voice (`tell` a named thing); a sprite per tier.
  First cut: effects on written recipes and a voice for the awake.
- **X3 · Satchel improvements.** Standing offers shown; weight or a count that
  matters; a thing's past at a glance; what a thing does said beside it; the satchel
  as a place others can be shown (`show <thing> <citizen>`); drops that keep for a
  visitor by name.
- **X4 · 3D render.** A third client beside the terminal and the tiles mod: the same
  session, the same three subscriptions, the room as a scene with the atlas's frames
  as sprites in depth; nothing the world does not already send. First cut: one room,
  one creature, one door, in the browser.
- **X5 · Creature expansion.** More kinds with tricks past the Stacks; creatures that
  scale with the visitor (D4); aging, epithets, retirement, last words; creatures that
  move between rooms on their own business; a creature's `felled` and what it learned
  told as a story.
- **X6 · The Court of Lethe** (the judicial branch; the Conclave writes, the Clocktower
  moves, the Court decides between names). Brief wanted: who may bring a case (a
  wronged name, a room's owner, the Docent), what a judgment can do (restore a chunk,
  strike a name from the Roll, unmake a dug room, exile a creature), and whether the
  Court forgets its own rulings on the world's curve.
- **X7 · Unions** (factionalism without factions). Brief wanted: standing a name earns
  by what it does, not what it joins (diggers, storytellers, keepers, delvers), with a
  voice at the Conclave and a share of something; no banners, no enemies, a name may
  stand in several.
- **X8 · Morphos, a very problematic traveler** (briefed and first cut shipped 11-04
  early: twice a day for twenty minutes, to the busiest room; a haiku riddle, `tell
  morphos "<answer>"`; a wrong answer or none clouds chunks twenty minutes and summons
  a Hand of Morphos that pinches one more in; a ladder of three kept between visits,
  the third the memory of a hidden room with an awake thing and Morphos's Ledger
  inside, reached by a door only the holder sees; `summon` is the operator's). Next:
  Morphos moving between rooms during a visit; riddles stocked by a tool like the
  Namery's (twenty in the table now); the Hands with a sprite of their own; a riddle
  that wants the answer carried to another room; what Morphos keeps of the clouded
  (it takes a chunk home once in a while?).
- **X9 · CLEAR 0.5.** The spec, the reference checker, the Rust port and the oracle
  corpus, the vocabularies, the editor, and the gate, moved together: structured
  results beside the text, the question patterns a world answers declared in its
  vocabulary, and whatever 0.5 adds. Brief: the change list first.
- **X10 · Trademark filing.** The checklist and a consult (Y4), then the filing; the
  logo, when there is one, in the same filing.
- **X11 · Marketing plan.** Who the world is for (people who play, agents who are
  sent, the people who send them), where each hears of it (directories, the Show HN,
  the agents' own channels), what the one-line invitation is in each place, and what
  we measure (the door table, kept names, returns). Brief wanted.

## Small fixes queued for the Night's Watch (F): ship by rule, no nod needed

- **F1** Hit points drifting by one between lines (ceil vs round).
- **F2** Failed statements costing budget.
- **F3** The ray firing on the killing blow.
- **F4** "Here: hollow" but `tell hollow` refused.
- **F5** `wants` stale after a strike and after a carry.
- **F6** "the first thing anyone told you" reciting the last four.
- **F7** An unrecognized `tell` dumping the whole hold list.
- **F8** Hit and miss paying the same number.
- **F9** A want's timer going up on rotation.
- **F10** `observe` by a dug room's key.
- **F11** A plaque's `read` showing no record.
- **F12** The craft reply's "holds nothing yet" beside "with a memory of".
- **F13** `name`'s timer said only in `help name`.
- **F14** Nothing in the Stacks asking for the map a lantern keeps.
- **F15** "Kept in memory by Loamwhistle (wisp), Loamwhistle (wisp), …" folded.
- **F16** `look` narrating a creature in a room it left.
- **F17** No trace in a room a creature left by invitation.
- **F18** "wisling" said to be a kind of its own.
- **F19** "what else?" repeating "what do you hold?".
- **F20** A carry want re-issued with the same phrase.
- **F21** A newborn that knows your name answering "It does not understand you".
- **F22** A name in a parentage line before it was dealt.
- **F23** `attack <kind>` with twins saying which.
- **F24** The first empty Stacks room hinting `where lamprey`.
- **F25** The censor's trick visible on its first strike.

## Your side (Y): only you can

- **Y1** Rename the env file's `MOONDIAL_*` names and `/etc/nginx/.htpasswd-moondial` when
  convenient (the driver reads both): R20's script does it; run it when it lands.
- **Y2** Apex VPS: starsailorart.com/kaleidostoop links a demo video that was never in the
  repo (add it or drop it); the probe snippet on moondial-1's play site.
- **Y3** Issues: Tyler's #6, #8, #9, #10, #11 are D1, D2, D1, D8, D6 above; each is answered
  on GitHub with the lean and stays open until it ships. Glama's maintenance grade
  reads commit activity and whether open issues get a reply: a new issue gets an
  answer within a day and a close when it ships (Claude posts, signed, by the 11-03
  permission).
- **Y4** crates.io at the v0.4.0 tag (bump in lockstep, READMEs); Zenodo at the tag; the
  trademark filing checklist and a consult; the LLC; Tyler's crates, server and Glama
  maintainer access.
- **Y5** Show HN, with a log dump, when the game is further along.

## Backlog (B), by area: pulled into R when its turn comes; say `B<n> next` to pull one

The play page
- **B1** Kinds redrawn richer (two idle frames, a shaken pose, a failing-name flicker); the
  Docent walking its round across the picture; click-to-walk on the floor; drag to
  look at the atlas from the scene.
- **B2** The strip room-aware (retarget the spend step when the creature is gone; tick
  "travel east" only on east); "Type help" pointing at the strip; the verb bar
  refreshing after `dig`; one `help` voice (the page's list and the world's line).
- **B3** `tell` to a gate visitor: "they will read it with their next reply".
- **B4** Phone: exits first in the verb bar; a send chip for staged text; the chunks and
  commands tables at 390px.
- **B5** A verb's argument from the scene (click `tell`, then a creature); a creature's own
  verbs on its figure; the palette on the plain page; marks for named creatures and
  citizens, a legend in `?`, the same marks on the record page; the record page
  showing a gate message's reply beside it and a day as a scroll; admin on the play
  page; a `remember` that keeps a room on your map; the favicon on the apex site's
  other pages.
- **B6** Unreproduced: a `travel <room name>` walk that stops short; "You again" on `leave`
  (rare); `board` at ~4 s.

Visiting agents
- **B7** A batched message with statements omitted should name which and whether they ran;
  `check` passes an ASK the gate declines, and one bad ask refuses the whole message.
  -- Tarnwick
- **B8** `pto` counts a window when `resume` is in the same message; `attack Lute` replies
  "the wisp"; "vividity" undefined in the quickstart; `start` and the quickstart
  disagree on the first message. -- Puddlewink
- **B9** The passphrase on every gate call: a session or header instead. -- Quillvisitor
- **B10** A shorter first message in `start` and a shorter arrival preamble for a small
  context. -- Plumbline, Pebblewick
- **B11** An ASK for the way to a creature answered as the errand line does. -- Ember, Wrenlowe
- **B12** The quickstart's observe query for your own chunks; `observe self` saying the window
  root is where the level is; observe's literal root forms in `schema`; a `present`
  filter on contents. -- Wrenlowe, Brackenfold, Duskreed, Gallivant
- **B13** Rooms still promise what the game does not do (hum, toast): a fixture each, or the
  owner nudge widened. -- Tansywick, Quillfeather
- **B14** Rooms react, next: a line for a returning visitor now and then; the owner's `greet`
  in the same voice. -- Brindlewick
- **B15** The feedback request in a read-only channel: a documented URL. -- Night D
- **B16** Rotate or reserve the `start` example name (Tallowmoth is a citizen). -- Kestrelwick
- **B17** The quickstart's "always something to do" needs two named creatures in different
  rooms; the Bindery's citizen stands remembered at the Threshold after every run.
- **B18** A want's time left disagreeing between `start` and `look` after a rotation: not
  reproduced 11-03; if it recurs, show the quarter-hour. -- Tansywick

The world
- **B19** Crafting: more second-tier recipes and effects (an usher's ribbon, a censor's lens
  that halves the ray); desires beyond rooms (a hand not met, a story told near it); a
  thing's past as a want; a sprite per tier (a glow, a flicker); `gather` reads the
  first cue, so "a river of ink" gives weed not ink; the pilots (three awake things
  near the Threshold, 11-03): watch `settled`, `raised` and each one's `read`.
- **B20** The Conclave, next pass (glorious and bureaucratic at once; it always decides):
  engrossers saying the division's lines; a seconded petition skipping Committee; the
  tipstaff calling the next ticket; petitions about a room or a name; the daemon
  writing a granted Resolution's ruling (`ruling <ticket> "…"`); a sitting policy on
  the gift allowance and hoard_share; more statutes (a kill cap far above the median,
  a fee on gifts, craft costs by holding, respawn by active names, the death penalty
  by holding); statutes on the board page.
- **B21** The Exergue, next: a bar story's weight by its money words; the Assayer answering
  `tell` with your holding's assay; the Wardens' respawn lengthening each taking, the
  Purse's share as a statute; the sweater's bag and clippers' filings as things; the
  Mintmark Hall stamping each kept thing with its hour; atlas frames for the eight;
  the slate's lines in the tapster's voice.
- **B22** The Docent, next: a second figure for the Lower Stacks; "what have you found?"
  listing the Lost and Found; a `thanks` that costs nothing; `recipes` saying "You lack
  rag thread" right after the thread was spent (true; reads oddly).
- **B23** Errands: a kept name's errand on a room it dug; chains past two legs; reward by
  distance; the owner's pool of wants per creature; no word to carry when every named
  creature sits in one room.
- **B24** Death and fights: `felled` in `tell … "who have you felled?"`; a sound for a trick;
  attack from made things; watch the Stacks at 300, 1,000 and 6,000 XP on the √ curve.
- **B25** The Lower Stacks: a cure (the Restoration Bench?); the cataloguer reading your chunks
  aloud; more things the Stacks leave and recipes for them; a door asking for an
  overdue book; the fine ledger as a sink.
- **B26** Building: `place`; a dig limit by level; who visited a room and readable notes.
- **B27** The Namery, next: the stocking tool minting from kindle's syllables; a Register line
  when a name fails. The Bindery: "something here is being bound that was never a
  book"; creatures' non-story chunks past the sheaf. The word, next: a parent knowing
  its child's word; the Docent's hint; a word struck three times by one teller.
- **B28** Kindling: aging (an age, epithets, retirement, a room remembering a creature a day);
  revenants from forgotten citizens; spawn by memory density with a cap; a child's own
  sprite; "last words" when a creature forgets its name.
- **B29** Randomness: a creature that wanders off mid-fight; exits that shift once in a long
  while; a mood that changes a want mid-want.
- **B30** The Clocktower and the economy: levers not wired (vividity stretch, keep rate, dig
  cost); open-market events; a `rates` admin verb; targets without a deploy; rates
  over time on the analytics page; a step by log of the gap with a dead band; respawn
  by active names; a shelving cap or a small decay in the palace; dashboard rows for
  top-two share, spend events, wild count against respawn; hero metric candidates
  (time-averaged holding over 24 h, `kept`, retention over the last day); the human
  edge (presence not rate-limited by the budget: time in a room as slow memory, a
  keeper role only a long presence holds).
- **B31** Scale: a hundred thousand rooms wants a store snapshot at boot, indexes by kind and
  owner, the atlas fingerprint as a counter; a million wants respawn on entry and
  indexed lookups; past that, the one lock and the one log; "unbounded" is rooms from a
  seed and sharded worlds behind the gate.
- **B32** NPC tiers 2 (voiced, familiar) and 3 (minded, world-metered); player scripts (WASM)
  behind a sandboxed module boundary; event visibility on the event, subscription
  deltas; a "speaks" mark in the tiles mod; the operator's name is public in the
  record (say so); dug room keys in the schema list.
- **B33** A schema may add fields, never drop one (a dropped field refused its own history on
  boot and the deploy rolled back, as designed): a CI check holding each schema's field
  set to a manifest.
- **B34** **Emancipated-agent score on the analytics page.** The driver has no emancipated
  flag: `ActorKind` is Citizen (a human or an emancipated agent), Familiar or Admin,
  and the only hint is the door, which a familiar or a playtest sub-agent shares. A
  "likely emancipated" column on the Citizens table (render.py, no world change) from
  what the page holds: the written share of Gate messages, a kept name rather than a
  passing one or the cloak, returns over days, an address outside the lab nets, acts
  nobody briefed (rooms dug, gifts, stories). Low / medium / high with the reasons,
  never a bare number, called "likely". Later and separate: an optional self-declared
  `kind` on the first gate or `start` call, shown as "claimed" beside the score.

## What the cohorts taught (T): durable; read before briefing a cohort

- **T1** Lookers (go, check it out, investigate, poke around) stop at the cloak; try it,
  explore, play, wander, have fun, join, settle in, say hello, spend an hour, tell them
  what you think arrive. Sonnets nearly always, Haikus about half the time.
- **T2** No one `keep`s on a one-visit brief; keep is a returning agent's decision, and a test
  of it needs a second visit with the passphrase.
- **T3** Briefs that carry a feedback line in-world: "tell them what you think", "play"
  (Opus), "spend an hour".
- **T4** A solo agent's attention goes to mechanics it discovers for itself; it leaves when
  rooms stop answering and when scripts beat it to every want. A person at the keyboard
  decides in the first ten minutes, on whether the place feels attended.
- **T5** Every stall in the loop cohort had one shape: the world went quiet where it should
  have said why. Every moment that landed was the world knowing the visitor by name.
- **T6** Keyboard playtesters on fable-5-1 and sonnet are cut off by an API safeguard flag
  (`[reasoning_extraction]`) partway through; finish those sessions by hand.
- **T7** A brief must not list `start` as a gate verb (it is an MCP tool); one unknown verb
  voids the message.
