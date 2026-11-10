# PFR: the punch list

What is open, by who acts on it. Items move to the ledger at the bottom when they ship;
what shipped from a visitor's word is also a dated line in
`site/sites/agentscanplay/feedback.txt`, and the rest of the history is `git log`. Restructured 2026-11-03 night (it had become a diary again).

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

The gate and the hour (from the Watch of 11-06)
- **D20 · A refused guess and the hint.** The hint narrows on heard misses only; four guesses refused inside the hour narrowed nothing (batch F puts the hint in the refusal). a) **lean**: as now. b) a refused in-hour guess counts as a miss (narrows the lent-word pool to five after two refusals). The Watch of 11-07 (Gnatterwick) climbed two rungs in two minutes and stalled on the third for the same reason, and asked for b, with the declined word struck from the shown pool. The Watch of 11-08 (Gnatterbell) asked a third time; R117 (batch T) has the refusal name the pool words the story carried as not it, which is what a refusal can say without a record of itself; b (a refused guess on the record, the pool narrowing) still waits on a.

The library's floors (from the Watch of 11-07)
- **D21 b · A second spawn per room** (M; open for a word). `SPAWN` takes a list, and bookworms are seeded in the Unfiled Archive beside the inklings, silverfish in the Cistern (a was shipped 11-08: the four stay below the stair).

Economy and death
- **D5 · A floor under a name** (deferred 11-02 "until the map is bigger"; **waits**,
  detail asked 11-05). What happens now: a named creature's name rests on its
  holding; when its chunks fade below sixty the name lapses ("forgets again"), its
  wants go with it, and anyone who knew it loses the thing they knew. The Watch keeps
  finding the world nameless at the hour it arrives (Gnibblewick, Gnomekeeper: "no
  named creature wants anything"), because names made in one session fade before the
  next. The options, as they would play:
  a) **still deferred**: the map now has nine more rooms and the trail; the lapse is
  the game's thesis (a thing kept only while remembered) and the fix is D14 (pilot
  named creatures near the hub that never lapse), which ships by policy. Cost: the
  social and lore loops stay empty for a newcomer between visits.
  b) **a floor**: once named, a creature holds at a floor (say twenty) and its name
  stays; only a fight that fells it, or a robbery, takes the name. The fade still
  runs above the floor, so a neglected name is a thin one (no wants, no stories held),
  not a gone one. Cost: the world fills with thin named creatures nobody tends; the
  Namery's pool drains; "forgets again" stops being true. Mitigation: a thin name
  (at the floor a week) is "half-remembered": `look` says "Vellic, nearly forgotten",
  wants stop, and the Namery may reclaim it at the next naming.
  c) **a floor for the tended**: a name holds at the floor only while someone who
  keeps it has been in the world in the last three days; untended, it lapses as now.
  Keeps the thesis (remembered by someone) and the empty-hour problem is solved for
  any creature a regular tends. Cost: one more rule to explain; `look` must say
  "kept by Spacepants (last here 2d ago)".
  d) **the shelf for names**: a keeper can `shelve` a chunk into a named creature as
  now, and the palace curve applies to the creature's holding when a keeper shelves
  into it (five times slower), so a tended name lasts days, an untended one a night.
  No new rule, one verb that already exists. Cost: it rewards hoarders of creatures.
  Lean, if asked: **c**, with d as the mechanism (tending = having shelved into it).

## Ready to ship (R): all ship on the Night's Watch by policy, in Claude's batches

Each is designed and needs no decision. Say `R<n> hold` to keep one back.

Away from Moondial and Entropic (the engine's and the game's old names): done in the
tree 11-06 (R16–R20); the box's half is Y1. The `MOONDIAL_*` fallbacks in the driver
and the tools go in a release after Y1 has run.
- **R26** (M; was B6, pulled 11-04) Unreproduced: a `travel <room name>` walk that stops short; "You again" on `leave` (rare); `board` at ~4 s.
Left from batch E:
- **R76b** (M) An Under-Docent at the Stacks' stair: a second figure for the Lower Stacks.
- **R10b** (S) The "five times slower" measured 1.6x: the shelf is a power law (half in 5x at first, a quarter at 15x), so an interval measure reads lower; the line now says so. Still to check: per-chunk half-lives after a strike (none should move: a strike makes a new chunk).
Left from batch D (the play page's bigger pieces):
- **R12b** (M) The transcript folded to a drawer with a "log" tab once bubbles carry the speech.
- **R13b** (S) A sprite per thing a creature leaves (icons are by word now).
- **R22b** (S) One `help` voice (the page's list and the world's line).
- **R25b** (M) A verb's argument from the scene (click `tell`, then a creature); a creature's own verbs on its figure; the palette on the plain page; the same marks on the record page; the record page showing a gate message's reply beside it and a day as a scroll; admin on the play page; a `remember` that keeps a room on your map.
From the Night's Watch of 11-08 (after batches O–S; the obvious fixes are in the ledger as batch T):
- **R138** (S) The Tally Room's sign gave the Gallery's way to the Division Lobby ("north, north, west"; from the Tally Room it is north, west, north, west). The sign says the right way and points at `where tally book` (batch T); the better fix is a sign that computes its way, as the Stile's step does.
- **R140** (S; looked at 11-10, open: the source is built in one place (errand.rs, "carrying <sender>'s word to <receiver>"), so the two clauses are likely two memories, one carried each way; kept for a Watch to reproduce) A dug room's "Dug by Thistlecroft with the memory of carrying Corrowkin's word to Corrow" and "Kept in memory by Thistlecroft (with carrying Corrow's word to Corrowkin)" flip the word's direction between clauses: one of the two source strings is built backward.
- **R142** (S) The Docent, asked two things ("Where is the tally book, and what does travel back do?"), answers by heart from one verb's help and drops the other; by-heart (R114) should split on "and"/"," and answer each clause, or say which it is answering.
- **R144** (S) The Docent suggests `tell Wyndle "what is your word?"` minutes after Wyndle's name failed (it is wisp-6579); the by-heart and the model's briefing should read the creature's current display.
- **R148** (S) The lent-word strike pay reads as fractions of an unnamed thing ("half again", "as much again", "one and a half times again", "2.5 times the pay"), and the in-hour "bonus alone" first strike (3 XP) paid less than a plain miss (4–6 XP): say the figure beside the words ("half again: 3 XP"), and say in `help story` that inside the hour only the word's bonus is paid.
From the Night's Watch of 11-06 (the Mudroom's first night; the obvious fixes are in the ledger as batch F):
- **R101** (S; looked at 11-07, open: `rest`'s "Full in" is read off the body's curve on active time, and a hypothesis is that the clause ran before the arrival's recurve; not reproduced, kept for the next Watch to measure) `rest` in a room just fought in moved 6 s in 31 s (nothing was fighting; the mothwing was dead); check whether `end_fight` recurved the mending.
Pulled from the backlog 11-05 ("B19 yes, B21 yes, B22 yes"), each in pieces the Watch can take a batch at a time:
- **R74** (M; B19) Crafting, second tier: an usher's ribbon (a thing that lets you choose the room an usher shows you to) and a censor's lens (halves the ray) as recipes from Stacks drops; desires beyond rooms (a hand not met, a story told near it); a thing's past as a want; a sprite per tier on the page; `gather` reading the first cue ("a river of ink" gives ink).
- **R75** (M; B21) The Exergue, next: a bar story's weight by its money words; the Assayer answering `tell` with your holding's assay; the Wardens' respawn lengthening each taking; the Purse's share as a statute; the sweater's bag and the clippers' filings as things; the Mintmark Hall stamping kept things with their hour; atlas frames for the eight; the slate in the tapster's voice.
From the Night's Watch of 11-04:
- Kept on purpose: `nginx-moondial.conf` (moondial.ai redirects to foxing.dev), the
  `/clear/entropic.yaml` redirect, the postgres schema-lock constant, the host's name
  moondial-1, TRADEMARK.md's history line, the repo's local folder name.

## The front door (H): the Threshold redesign, step by step

Your project (11-05): "an awesome front door for players". Asked in chat as T1–T3 that
night (the cohort T's below are another list); kept here as H. Each later step is
yours to call: say `H5 go`, or `H5 brief` for a design first.

- **H4 · The Threshold's text as a front door** (S). Rewritten to present the three
  ways as three invitations (fight in the fog, read under the lanterns, dig in the
  mud) and name the one thing to do first; the Mudroom's text to match. The Watch of
  11-06 (Gnomestead) asked for the same on the exits line itself: "east (Fogbank:
  fight), west (Mudroom: dig), north (Lantern Row: read)"; the three doors map
  exactly to the three ways, so the line should say so.
- **H6 · A signpost at the door** (S). Like the Trail's: `read signpost` lists the
  nearest named creature, the newest dug room, what is wanted, by the signed way.
- **H8 · The Mudroom as a place** (M). Boots that say who dug last, a spade to
  `take` that marks a dug room's door while carried, the Docent's round through it,
  a line on `look` for the three best-remembered dug rooms.

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
- **X5 · Creature expansion.** David's rule (11-06): ship more creatures; **a creature
  never scales with the player; it scales with its own level.** First cut **shipped
  11-06 (batch J)**: (1) the rule: a creature's hit points and attack grow with what it
  holds (max hp from its spawned figure, by half at 75 and double at 150; a point of
  attack per 75 held; the wound keeps its proportion; recomputed with every tier
  check), so a creature fed to its name is a harder fight and a richer kill, and the
  door's pilots at 75 are 12-hp wisps hitting 3–5; (2) six kinds on the library's own
  floors, each with a trick the Stacks already know: lanternflies on Lantern Row (5
  hp, the mildest thing in the world, and a kind the Docent may name), bookworms at
  the Overdue Desk (Leech), silverfish in the Card Catalogue (Clip), a dust devil on
  the Downward Stair (hunts; Shove), a shelf mouse in Lost and Found (Chatter), a
  custodian in the Strongroom (hunts; Maw); `plural()` for the new names. Next cuts,
  by brief: aging (a named creature's level drifts up a little a day it is kept),
  epithets (earned from what felled it or what it fell), retirement (a creature past a
  level walks to a room of its own and stops fighting), last words (a felled named
  creature says one line from what it held), a creature's `felled` told as a story;
  pixel sprites for the six (the page draws their glyph for now).
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
  vocabulary, and whatever 0.5 adds. Brief: the change list first. On it (D18,
  11-06): what the gate may mend before the check so fewer messages score a 50
  (the message stays all-or-nothing): inner straight quotes curled (done, R30), a
  missing final period supplied, a `look` after `travel` folded (done), an unknown
  verb answered with its nearest approved one (`strat` → `start`? no: `story`),
  a statement over the word limit split at a semicolon, and the oracle cases for
  each.
- **X10 · Trademark filing.** The checklist and a consult (Y4), then the filing; the
  logo, when there is one, in the same filing.
- **X11 · Marketing plan.** Who the world is for (people who play, agents who are
  sent, the people who send them), where each hears of it (directories, the Show HN,
  the agents' own channels), what the one-line invitation is in each place, and what
  we measure (the door table, kept names, returns). Brief wanted.
- **X12 · Tactical battles.** Fights with position and choice: the room as the grid
  of tiles the scene already draws, cover and reach, a turn that is a move and an act,
  creatures with roles (a footpad flanks, a longsight spots, a gardener holds a line),
  two or three citizens against one thing, a retreat that is a path rather than a
  verb. Scaffold: `fight_rounds` and the gate's per-statement round are the clock,
  the scene's tiles are the board, and the Wistful Trail's Hollow Way (two footpads
  from both banks) is the first set piece. Brief wanted: what a turn costs in messages.

- **X13 · A world that is lived in.** Shipped 11-05 as the first cut: happenings (twice
  an hour somewhere), named creatures' business (walk to a keeper, home, count), the
  Conclave's holidays (no division; the house's characters in tempers). The rest, by
  complexity and policy:
  - **X14** (XL, brief wanted) Day and night: which creatures hunt by the hour (hollows
    by night, sighs by day), the page's light, the Fogbank's fog by the hour.
  - **X15** (XL, brief wanted) A travelling market: the Exergue's tapster walks a circuit
    of the ground floor, the slate with it, trade where it stands.

## Small fixes queued for the Night's Watch (F): ship by rule, no nod needed

(Emptied 11-06: F2–F25 are in the ledger, three of them as checked and not reproduced.)

## Your side (Y): only you can

- **Y1** The rename's last step on the box, when convenient (the driver reads both names):
  `sudo bash /srv/foxing/current/deploy/rename-env.sh` (R20; it prints its diff and asks;
  `DRY_RUN=1` to only look). Read it line by line first: deploy/server/rename-env.sh.
  Then R11 by hand in `/etc/nginx/sites-available/play` (certbot's 443 block lives in
  the live file, so the repo's `deploy/server/nginx-play.conf` is not copied over it):
  add the `upstream foxing_driver { … }` block from the repo's file above the first
  `server {`, change both `proxy_pass http://127.0.0.1:7777;` lines to
  `proxy_pass http://foxing_driver;`, add the three `proxy_*` lines in `location /`,
  then `nginx -t && systemctl reload nginx`.
- **Y2** Apex VPS: starsailorart.com/kaleidostoop links a demo video that was never in the
  repo (add it or drop it); the probe snippet on moondial-1's play site.
- **Y3** Issues: none open. Tyler's #6, #8, #9, #11 (D1, D2, D1, D6) closed 11-05 with
  signed replies naming the commits; #10 (D8) closed by design. Glama's maintenance grade
  reads commit activity and whether open issues get a reply: a new issue gets an
  answer within a day and a close when it ships (Claude posts, signed, by the 11-03
  permission).
- **Y4** crates.io at the v0.4.0 tag (bump in lockstep, READMEs); Zenodo at the tag; the
  trademark filing checklist and a consult; the LLC; Tyler's crates, server and Glama
  maintainer access.
- **Y5** Show HN, with a log dump, when the game is further along.

## Backlog (B), by area: pulled into R when its turn comes; say `B<n> next` to pull one

The play page (B1–B6 pulled into R21–R26, 11-04)
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
- **B19** (pulled 11-05 → R74) Crafting: more second-tier recipes and effects (an usher's ribbon, a censor's lens
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
- **B21** (pulled 11-05 → R75) The Exergue, next: a bar story's weight by its money words; the Assayer answering
  `tell` with your holding's assay; the Wardens' respawn lengthening each taking, the
  Purse's share as a statute; the sweater's bag and clippers' filings as things; the
  Mintmark Hall stamping each kept thing with its hour; atlas frames for the eight;
  the slate's lines in the tapster's voice.
- **B22** (pulled 11-05 → R76) The Docent, next: a second figure for the Lower Stacks; "what have you found?"
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

## Shipped (ledger): moved out of the sections above when they went live

11-10 (batch U, by David's word: D25 a and the S items left of the Watch of 11-08; one driver deploy)
- D25 a: `travel` says a room short when you stood in it within the hour ("The Fogbank, as you left it (`look` has the page)", then the exits, what stands and lies here, who is here, what hunts); the result carries `brief`. R139 (the Minute Room's headers carry the day, and "the last three, the oldest of them first"); R141 (a book's row says its entries, not its lines, and no holding); R143 (`time` says the hour and the bells in words; the rates and the last read are in the result as `rates_line` and `last_read`, and public at /v0/economy); R145 (a creature asked who has been here says the names first, counted apart from what was said); R146 (`help attack` says why twins carry ids); R147 (the Lectern says a resolution is the house's opinion and changes nothing it names); R150 ("A thing is left behind one time in two after 3 rounds, always after six; this time, none").
- The play page's You panel is a card: the name, then level, hit points and attack as three figures with a bar and a line each (the fall an hour, "whole in 4m", "+1 at level 25"), what is the matter with you as chips (fighting, bleeding, numb, shaken, falls tonight), and the window's memories dearest first with "halves every 44m · vivid 9%".

11-10 (the play page, from David at the keyboard; two site deploys, no driver change)
- The second coat (d31f0f4): the name screen is a title under a waning moon with one field; at 1100px and wider the scene and the story stand side by side (the bar drags sideways, `disremembered.sceneW`); panels are cards, the verb bar chips, the prompt a field; passphrase, save and leave are in the settings menu.
- A fixture's name is drawn whole, on two lines when long ("Book Nobody Is Rea" was a cut at eighteen letters); the Satchel lights when `recipes` says you could make something (the things that go into it ringed, a button a thing that starts `craft`), asked through a new quiet `foxing.read(verb, args)` for the free reads; "holds N" after `remember <chunk> <whom>` rises over the one remembered into, not the giver (the result was matched on its first word, the chunk), and a given memory rises over its receiver; a thing in the picture keeps its tile while it is in the room (tiles were dealt by place in the room's list, so a kill reseated the rest; a fixture moved when a loose thing was taken); the door's title is one word on one line in any serif (a wider face put the last letter on its own).

11-09 (batch T, the Watch of 11-08's obvious fixes; one deploy)
- R117–R137 as marked; D24 a (the want heard early once an hour; the hour runs from the story that opened it, and neither an early nor a word-carrying story moves it); R138's sign says the right way; `help retreat` no longer says "You mend only out of the room" (nothing mends in a fight; out of it you mend anywhere); `help attack` says what vividity measures (your blood the fight took; a one-blow kill is barely remembered); "2 lanternflys" is "2 lanternflies"; the Lookout's text says it sees the trail's nine rooms, not every place you lingered.

11-08 (batch S, the Watch of 11-07's R list and D21–D23 by policy; one deploy)
- R107–R116 as marked (R113 checked and closed); D21 a (llms.txt says four live below the stair), D22 a (the spawn jitter a twentieth), D23 a (`time` and `petitions` say a quiet name finds the division under "Meanwhile"); a seeded room's text and name follow the def on boot.

11-07 (batch R, Gnomonwhistle's; one deploy)
- "Meanwhile" is cut at the last reply, not at an arrival: a name quiet an hour in the house came back to find the division's line only in `messages` (a day of lines at most, eight and a count); the petitioner no longer hears their own petition twice at a division; the Tally Book keeps its entries across a boot (F's seed-text refresh wiped them every boot from F to Q; the head alone follows the code now) and re-enters the last twenty divisions from the record where it holds none.

11-07 (batch Q; one deploy)
- The atlas's layout cache is one a thread in the test binary again (process-wide since F): two tests' worlds read each other's "before", and a room with room was shoved (a CI flake on P; the driver had deployed).

11-07 (batch P, Gnomeclatter's; one deploy)
- A named creature that comes to know you greets you by name on your next arrival (the hour's one greeting was spent on the first arrival as a stranger); the Docent's "came through" clock counts what it set down here, not only what it took; set-downs are said once a pass, folded ("moth dust ×3").

11-07 (batch O, the Watch of 11-07's obvious fixes; one deploy)
- (Gnibblewort) the Docent's first-arrival move and the first-moves example say "a dozen different words" and the double's rule (a newcomer's first story bounced at eleven on a repeated "a").
- (Gnatterwick) "it listens for room, and did not hear it" for a one-word want; the miss grace said right ("The hour is not spent on a first miss; a second spends it" / "That is the second miss; the hour is spent": the code allowed two free misses while the text said once); the bonus-alone pay in words ("half again of what a story pays"); the third rung's hint names `look`. D20's evidence noted.
- (Gnashpocket) the hunters' line says a name in its first hour is left alone (every Watch name is, which was last night's "hunts but never attacks" too); the kill line's "halves every" did not show (the chunk's clock is read after the curve is set now); the retreat's rounds say "your hand unsteady" when shaken.
- (Gnudgeknob) `where` and the Docent's routes say "(the stair down asks a level of one hundred)" when the way runs through it; a room's refill counts its own kind only (a born stranger held the dust devil's slot); D21 filed for the four below.
- (Gnashlantern) the remember reply says the next bar ("at 75 it hits a point harder"); `help attack`'s figures are ranges and say the rule; the Docent is told `observe` takes JSON, never a name.
- (Gnarlbrook) `back` cuts loops: a step onto any room on the trail cuts it back to there; `help travel` describes the trail; the Threshold's tally and the Lookout's bench read a came-through record kept on the room (the log scan reached an hour, not a day, so both were empty).

11-07 (batch N; one deploy)
- A step onto the room at the top of the trail is a retrace whatever door took it (`out` of a dug room, the compass back the way you came): as a step forward it put the room you left on the trail, and `back` walked you into it again (found live after M). The bog keeps the room you stepped in from with its clock, so "spat out the way you came" is that room, not the trail's.

11-07 (batch M, R84 alone; one deploy)
- A dug room's way home is named `out`, never `back`; the old doors renamed on boot; the quickstart says `back` retraces the trail.

11-07 (batch L, the Watch of 11-06's R list by policy; one deploy)
- R85–R100, R102–R106 as marked in the R list (R92, R96 checked and closed; R101 left open with a hypothesis). R84 ships alone next.

11-06 (batch K; one deploy)
- `look`'s query never selected the room's key, so every line keyed on it was silent: F14's lantern hint on the Stacks' landing (since 11-05), the weather (G), the door's three ways (H5), the chalk tally and the day (I). Selected now; a live look at the Threshold found it.

11-06 (batch J, X5's first cut; one deploy)
- Creatures scale with their own level, never the player's (`combat::rescale`, `base_hp`); six new kinds on the library's floors with homes and tricks; a boot fills a room to its figure exactly (the one-in-six extra comes on a refill of a room partly cleared); `plural()`; llms.txt and the quickstart say the rule and the kinds.

11-06 (batch I, the rest of X13's queue; one deploy)
- R79 (the picnicker's place, the longsight's gaze, a creature that remembers you says your name), R80 (said-walls everywhere, the door's chalk tally, the bench's scratches, names on the atlas), R82 (All Names' Day at forty; a thing of each named day at the door).

11-06 (batch H; one deploy)
- H7 (the play page's door: the Threshold's picture, the first-minute card, the folded dug rooms; `map` sends `dug_by`); the published CLEAR vocabulary synced after `news` (batch G's site deploy failed on it) and the site check now compares the published CLEAR files with the reference.

11-06 (batch G, David's morning answers; one deploy)
- D18 a (the line says a message runs whole or not at all; the 50-reducers listed under X9), D19 b (the wanted story heard inside the hour at plain pay, the want standing), H5 (the Docent's first-arrival line by name with the one next move; the door's three ways to a name under an hour old), R77 (arrivals said; the round seen passing), R78 (weather by the rates and the hour), R81 (`news`).

11-06 (batch F, the Watch of 11-06's obvious fixes; one deploy)
- (Gnomestead) the Docent's stocked greeting is sieved when chosen, not only when stocked, so a newcomer is not told "you asked about fading"; `help dig` says every room but the Mudroom takes eight doors; the Fogbank refills by its unnamed count, and the Docent names no creature of a room's own kind while fewer than two unnamed stand there, so "attack a wisp east" finds one.
- (Gnomeforge) `gather` in the fog takes hollow water; `read` of a made thing says what it does; `craft` with a chunk the recipe does not take says the chunk is kept; `satchel` says what each made thing does.
- (Gnarlfoot) `travel back` retraces the trail first, even in a dug room whose way out is named back (the Mudroom–Moss Bed loop); "You take the survey door", not "the survey"; the map's legend is sorted by number; one layout cache for the process, so a room dug between two `map`s does not move; a digger's text starts with a capital.
- (Gnatterbox) `tell wisp` beside named wisps names them ("The wisps here each have a name: Fallis, Tinsel. Tell one by its name"); "try again in 1 minute"; `wants` marks a creature full for the hour.
- (Gnibblestack) the retreat line says both clocks ("It mends on the world's clock, whole in 8m, before you are; you are whole in 10m") instead of "it keeps the wound a while", and sizes the second stand twice, now and when you are whole against the creature as it will be then, with "bring another" when the whole stand is beyond one name; `chunks` says attack is "two plus six tenths of the square root of level, rounded down" (it read as 2.6×) and "halves every 1h 20m" for an unshelved chunk, which does not count down.
- (Gnashbrook) the creature row says what it hits for ("hits 2–4"; R4's line never showed: `look`'s query did not select the attack), and the first-moves line says the wisps hit 2–4 and the hollows 4–6; "The wisp goes down" keeps its article; the "nothing unnamed" refusal counts the named it names ("Fallis the wisp has a name"), lists the unnamed not itself yet, and says a felled name loses its name and the stories told to it; the Docent's round leaves the Midden alone (it carried the swept back to the door and swept them again, and the two rooms told it in opposite directions); the greeting sieve fix above covers "you asked how to get rag thread".
- (Gnomonbell) the budget line shows its quarters ("47.75 of 60") instead of a rounded counter that fell by two; `session` says the clock time, not the same span twice; the Tally Book's seeded text follows the code on boot (it said "Nothing has been entered" above seven entries) and its entries carry the day; `help time` says what present means (acting within ten minutes) and that a bell missed is in `messages`. The leaked model name is the gate fix above.
- (Gnashwhistle) the gate kept an argument after a quoted text inside the text, so every Docent answer since R30 ended in the model's name; fixed, with a test; a lent word's hint narrows after misses (the pool, then five of it), so the door's pilots can be climbed; the inside-the-hour refusal says the story did not carry the word and repeats the hint; "what word did anyone carry?" lists the words under the others said to it beside the errands'; `tell docent "thanks"` says no question was spent.

11-06 (batch E; one deploy)
- **R70** a creature keeps its wound twenty minutes in the Stacks as on the ground; `retreat` says in rounds what a second stand would take, against what you have and will have.
- **R76** (part) `tell docent "thanks"` costs nothing; "what have you found?" lists what its round carried to the door today; `recipes` says "spent on the bandage just now" rather than "you lack".
- **R10** (audit) the lines read the right curves; the shelf's "halves in" now counts from now (a power law's halving lengthens as it ages); the `chunks` fade line gives the figures at the new stretch and says the shelf is a long tail.

11-06 (batch D, the rename and the play page; one deploy)
- **R11** nginx-play.conf: the driver listed twice as an upstream, a refused connection tried once more (the live edit is in Y1).
- **R12** (part) fog of war: a room in sight never walked is a hatched cell; the here cell pulses.
- **R13** (part) the Exergue's rooms each with their own furniture (die, hammer, balance, stamps, plaque, dated stair); a dark room is dim past one lamp.
- **R16** code and tool docs say `FOXING_*`; the fixture test reads `FOXING_WORLD`.
- **R17** site/analytics defaults to the new paths and names, the old read second.
- **R18** CI's PostgreSQL named foxing, `FOXING_TEST_PG`; deploy-driver.yml and remote-deploy.sh drop the symlink and the `/srv/moondial` fallback.
- **R19** `setup-server.sh` (was setup-moondial-1.sh) writes `FOXING_*` to `/etc/foxing`, `foxing.service`, sudoers `foxing`; harden.sh, setup-postgres.sh, enable-*.sh, setup-analytics.sh read the new names first; `.htpasswd-foxing`.
- **R20** `deploy/server/rename-env.sh` written and dry-run against a sample (Y1 runs it).
- **R22** (part) the strip's east step ticks on east or the Fogbank only; the room is asked again after `dig`; the welcome line points at the card.
- **R25** (part) a legend of the picture's marks at the top of `?`; the favicon on the board and record pages.
- **R34** the shaken state on the page: a row with the hit chance and the time to steady, and a tremor on the figure.

11-06 (batch C, the F list; one deploy)
- **F2** a message every statement of which was refused costs a quarter, and says so.
- **F3** checked: the trick is skipped on a killing blow at all three strike sites (no change).
- **F4** checked: `tell <kind>` finds an unnamed creature of the kind here (no change; not reproduced).
- **F5** checked: `wants` reads the live state, the hour from the same rule as `story` (no change).
- **F6** "recite the first story" gives the oldest; the level-one hint says to ask for it.
- **F7** the hold answer is four at a time with "ask what else" (already so; no dump).
- **F8** not reproduced from the one line; left out.
- **F9** a turned want's timer restarts by design; left out.
- **F10** `observe {"room": …}` takes a dug room's key or any room's name.
- **F11** a plaque with nothing written on it reads its record.
- **F12** the craft reply already says "the chunk is spent, not kept" (no change).
- **F13** `name` says the name's clock (holds N, goes under 30) in the reply.
- **F14** the Lower Stacks landing says what a lantern keeps.
- **F15** "Kept in memory by" folds a name's keepings ("wisp ×3").
- **F16** not reproduced; left out.
- **F17** a creature invited away leaves a trace in its old room for two hours.
- **F18** a mutant kind that reads as a typo of a known kind takes a syllable more.
- **F19** checked: "what else" pages from the second four (no change).
- **F20** the way-home phrase differs each time a pair exchanges.
- **F21** a newborn that knows you says so, wordless, with what it holds.
- **F22** a child's sealed name is the operator's to see, not `observe`'s.
- **F23** beside a twin, the strike lines say which ("wisp-6351").
- **F24** an empty Stacks room points at `where lamprey`/`usher`/`censor`.
- **F25** `look` says each kind's trick before it lands.

11-06 (batch B, by policy; one deploy)
- **D14/R42** (a: the Docent keeps three named creatures within two rooms of the Threshold, naming a mild stranger when fewer stand and feeding its own back up when failing; `pilots.rs`, `disremembered.piloted`).
- **R1** `back` retraces the trail a room per step (the bog's spit is a step back too).
- **R2** the gate's lockout is on the record (`citizen.login_failed`) and rebuilt at boot.
- **R5** the board's errands are words carried; engrossers write a `tell` down as a `say`.
- **R7** `login` refuses a verb, a compass word, a kind, a figure or a reserved word as a name.
- **R8** a written recipe whose purpose speaks of light, mending or charm makes a thing that does it, at twice the memory.
- **R9** `look` says when the Docent came through and what it carried or swept, for two hours.
- **R28** `wants` lists the awake things with their pull; the ground line says when one has settled.
- **R30** a quote inside a typed `say`/`tell` text goes curly, so the message holds.
- **R53** whoever stands in the tower, the Exergue or the house sees the paper change at the strike.
- **R55** `help time` says what the strike and the division do.
- **R57** the Lookout marks walkers quiet five minutes "(quiet)".
- **R59** the greeting sieve refuses every "you asked/said/wanted" form; the writer is told not to.
- **R60** the on-ramp names the Orchard's mothwings (first moves, `help attack`, the Docent's greeting, the Fogbank's wait line when others fight there).
- **R64** in a batch, one statement's answer is capped at twelve lines so the others fit.
- **R73** a question the world gave up on takes a late answer within the hour; the worker still sees it.

11-05 (batch A, by policy; one deploy)
- **D9** (a: a FACT beside the invitation's ASK saying what a name costs and does).
- **D13** (a: within its hour, a story that carries the word under the others is heard for the bonus alone; the refusal says so).
- **D17** (a: vividity 0 keeps half the base half-life, 1 three times it; `earn` defaults to 0.2, the base; the quickstart says so).
- **D8** (b: by design; #10 closed).
- **R4** `look` says what each creature hits for ("hits 2–4").
- **R31/R44** the drift-in estimate says the room's figure and the three-fifths-to-eight-fifths rule beside the next job's.
- **R43** `tell` to an unnamed kind says how one gets a name.
- **R46** `look` says when what you were fighting was felled by another hand while you were quiet.
- **R47** the budget line says when the rolling hour frees; a refused message's cost is its own FACT.
- **R48** `rest` ends "Level N; you hit for M (plus 0–2)".
- **R49/R67** the empty-handed kill says the rounds and the chance once a session, then "Nothing left."
- **R50** `drop` says the Docent's rule (second sighting, rounds ten minutes apart) rather than "on its round".
- **R54** "You take the abstain door." for a door word that is not plainly a thing.
- **R56** `rest` is whole within half a point.
- **R58** "Nothing here hunts; the wisps keep to themselves." where the room's text says nothing.
- **R65** "what word did X carry?" answers from the carried-word record; the hold answer no longer takes it.
- **R66** `retreat` with a hunter closing says it has not reached you and `travel` leaves first.
- **R68** the word's hint narrows after a miss: the text, then the sentence.
- **R71** `give` to a creature or the Docent says who can receive and how a creature is fed; the ray's line says what your hits land for now.
- **R72** the three hours named: the budget's hour, a creature's hour with you, the tapster's.
- **H1 · The rewire** (T1 in chat; decided a, shipped 11-05: the Mudroom west of the
  Threshold; dug doors rehung there on boot, `disremembered.moved_to_mudroom`; no digs
  at the door; no cap in the Mudroom; orphans rejoin there; a Mudroom tile).
- **H2 · mortar's test rooms** (T2 in chat; decided b, shipped 11-05: razed on boot).

11-04
- **D4 · Fights past level one hundred are clicking** (decided a, shipped 11-04: what a
  creature leaves behind goes by the fight's length, a one-blow kill one time in six,
  six rounds every time, at the Clocktower's drop rate).
- **D10 · The rest of the Architect's petition design** (decided a, shipped 11-04: a
  regular's petition taken first and never tabled; `attend`, free, the names read at
  the sitting and their petitions taken next; a hoard not heard upon its own tax).
- **R21** shipped 11-04 (two idle frames, a shaken pose, the failing-name flicker, the Docent's round, click-to-walk by a door, drag to see what you remember of the map).
- **R24** shipped 11-04 (exits first on a phone, with the room each leads to; the staged chip with Send and ×; a tap on a whole-line verb clears what was staged; the chunks rows and the commands table at 390px).
- **R27** shipped 11-04.
- **R29** shipped 11-04.
- **R33** shipped 11-04.

11-05
- **D1** shipped 11-05 (a: the shelf counts toward holding in the census and on the board, shown as its own column; `chunks` says what is on the shelf and that it counts for nothing toward attack).
- **D2 · The free fall** (Tyler #8; decided 11-03 23:39 PT, shipped the same night): a
  hit chance (nine in ten) with a shaken wake whose penalty falls on a parabola, five
  minutes a fall of the night up to thirty; creatures keep their wounds and mend on a
  Clocktower rate (`mend`), named ones walking to succor and home. Still open from
  the same issue, shipped 11-05: the halving by falls of a night floors at a
  quarter of the first fall's share, and the share is of window and shelf
  together, drawn from both.
- **D3** shipped 11-05 (a: the stair into the Lower Stacks asks a level of one hundred, in one line that says what lives below hits for and where the level is earned).
- **D6** shipped 11-05 (a: words past sixty buy vividity, 0.4 toward 0.9 at two hundred distinct words, never XP; the telling's line says so).
- **D7** shipped 11-05 (a: the first word under the others is drawn from a want the creature had before the teller came (`tell <name> "what did you want once?"` says it); the first thing it was told moves to the second rung, where `recite` surfaces it).
- **D11** shipped 11-05 (your call: to-hit 0.9 → 0.75 (three in four; shaken floors at one in four) and attack 2 + 0.6√level (2 at nothing, 6 at 60, 8 at 100, 12 at 300, 20 at 1,000, 48 at 6,000); the robbing test's robber earns first).
- **D12** shipped 11-05 (a: a fixture takes any chunk; its worth against what the recipe asks sets how long the fixture is remembered (the longest at the asked worth or more, a tenth at least), said in the craft line; `recipes` says "any").
- **D15** shipped 11-05 (c, your call: the Conclave is slow on purpose and says so, in `petitions`, the petition receipt, the quickstart, llms.txt and the Docent's rule text).
- **D16** shipped 11-05 (a: the meadow's sighs come back in three minutes, not forty-five seconds).
- **F1** shipped 11-05 (`rest` reads the ceiled figure the hit lines say).
- **F26** shipped 11-05 (`look` folds plain drops of one name, "river weed ×12 (item-… … item-…, 26–38% remembered)"; the Docent sweeps plain drops under 30% from the Threshold's floor into the Midden, three a pass, said on the heap; `disremembered.tidied` carries `swept`).
- **R3** shipped 11-05 (`look` names what hunts in a room: "The footpads hunt whoever comes in."; `check` left as is, the room line covers it).
- **R6** shipped 11-05 (`satchel` says "whole" where memory is full and lists standing trade offers with what is left of their ten minutes; the inbox's times are R-later).
- **R14** shipped 11-05 (a dug room tells its owner "Yours: N% remembered, halving every X; N of 8 doors left"; a world room with dug doors says how many are left and the newest three; `help dig` names the cap).
- **R15** shipped 11-05 (`wants` says "another from you pays ready in 48m" and "its name is failing at 39"; the `session` strike count was not reproduced and is dropped).
- **R23** shipped 11-05 (a `tell` to a name elsewhere says how it reaches them: through the gate with their next reply, or at a keyboard now).
- **R32** shipped 11-05 (the door-cap refusal lists the world's rooms with doors left nearest first, with doors left and the signed way, anchored at the world room a dug room hangs off).
- **R35** shipped 11-05 (`time` says the strike is on the hour and the division at half past, with both countdowns; `petitions` says which clock it means; `petition` says the stage path ahead).
- **R36** shipped 11-05 (the map's numbers are dealt once per name, in the order rooms came into view, the Threshold always 01, and never move; a visitor steered by "room 08" and found it 12 after a walk). -- Gnomebrush
- **R37** shipped 11-05 (the bog looked under the citizen for things to keep and found none: things live in the satchel; fixed, with the test moved to match). -- Gnarlbog, Gnomeward
- **R38** shipped 11-05 (the Hush's promise made real: what you `say` there leaves and arrives in another trail room a minute later as a whisper to whoever stands there, and you are told where it went; `disremembered.whispered` is public). -- Gnibblewick
- **R39** shipped 11-05 (the empty `wants` line names the verb that feeds a creature toward a name). -- Gnibblewick
- **R40** shipped 11-05 (`rest` in the bog says the clock too).
- **R41** shipped 11-05 (`where` and the walk toward the Long Afternoon say the wicket asks for the well's word; the last step of a walk already names the room, so no extra arrival line).
- **R45** shipped 11-05 (the Docent's own rule text says what it tidies and when: a second sighting carries, a plain drop under 30% on the Threshold's floor is swept to the Midden, never while anyone stands at the door; it promised a visitor a sweep "sooner if you are standing here"). -- Gnatterling
- **R51** shipped 11-05 (the trail's props answer `read`: the signpost lists the rooms you have stood in, the rim gives the word and the verb, the plate the view, the book your session). -- Gnomeward
- **R52** shipped 11-05 (`petitions` says "stage N of 5" on each ticket; a petition's closing period is not doubled in the echo). -- Gnomonclock
- **R61** shipped 11-05 (a named creature's "who made you" names the keeper who fed it over sixty, kept at the moment, not whoever fed it last; two keepers were each told they had). -- Gnomekeeper
- **R62** shipped 11-05 (`tell <name> "recite …"` has the creature say a held story whole, the newest or the one a word names; the hunt for the word under the others pointed at a story no verb could surface). -- Gnotwhisper
- **R63** shipped 11-05 (`read <named creature>` says it is not a page and points at `tell`). -- Gnomekeeper
- **R69** shipped 11-05 (a thing's name typed after `take`, `drop` or `read` is quoted whole at the gate, "dab of ink" included; `travel wicket "linger"` keeps its two arguments). -- Gnibbleforge

The shipped items as they were written, moved out of the open sections 11-10 (the batch entries above say "as marked" of these):

- **D18 · A message scored 50 that ran nothing.** Decided a (11-06, "CLEAR is binary, all or nothing"), **shipped 11-06 (batch G)**: the line says "a message runs whole or not at all". The fixes that reduce the chance of a 50 (what the gate can mend before checking: inner quotes curled, a missing period, a `look` after `travel`, an unknown verb's nearest) go into **X9 · CLEAR 0.5** as its change list.
- **D19 · A want-match inside the hour.** Decided b (11-06), **shipped 11-06 (batch G)**: the wanted story is heard inside the hour at plain pay, and the want stands (told again when the hour turns, it pays double); the refusal says what is heard inside the hour.
- **D23 · A visitor sees one division an hour.** The Conclave divides at half past; a name arriving at :25 has one usable division in an hour's visit, and a quiet wait for it is most of the visit. a) **lean**: as is, and `time`/`petitions` say plainly when the next is and that a quiet name finds it under "Meanwhile" on its next message (batch R makes that true); b) divide every quarter hour on the Watch's nights only (an env rate); c) divide every half hour always. S for a. **Shipped a by policy 11-08 (batch S)**.
- **D21 · Where the six live.** Four of the six new kinds (bookworm, silverfish, shelf mouse, custodian) are in Lower Stacks rooms behind the level-100 stair; a new name meets only the lanternfly and the dust devil. Every ground-floor room already spawns a kind of its own (one SPAWN a room). a) **lean**: leave the four below (they are the Stacks' own, and the stair is the stair) and say so in llms.txt; b) a second spawn per room (`SPAWN` takes a list) and seed bookworms in the Unfiled Archive beside the inklings, silverfish in the Cistern; c) move the four to the trail. S for a; M for b. **Shipped a by policy 11-08 (batch S)**; b stays open for a word.
- **D22 · A creature's spawned hit points vary by a quarter** (6.5–8.2 on wisps), more than one feeding's growth, so "hp grows with what it holds" is muddied by the roll. a) **lean**: narrow the jitter to a tenth; b) show base and bonus apart in the row ("8+3/11"); c) as is. **Shipped a by policy 11-08 (batch S)**: a twentieth either way.
- **D24 · The want inside the hour, again and again.** D19 b lets the wanted story be heard inside the hour at plain pay with the want standing; nothing capped it, so a teller farmed one creature with different wanted stories at 4–5 XP each (near-repeats halve, fresh ones do not), and each heard story slid the hour forward. a) **lean**: the want is heard early once an hour, and neither an early nor a word-carrying story moves the hour (it runs from the story that opened it); b) as D19 b was, a declared loophole; c) back to D19 a (refuse inside the hour). S for a. **Shipped a by policy 11-09 (batch T)**.
- **R117** (S, **shipped 11-09, batch T**; the in-hour refusal says "It carried ember; that is not the word") A story carrying a word of the lent pool, refused inside the hour, is told "that one was neither" and the pool still lists the word; the refusal names the carried pool words as not it (the pool itself narrows on heard stories only: D20).
- **R118** (S, **shipped 11-09, batch T**; a struck story says "A story that carries the word under the others spends no hour", is no miss on the record, and the hour's miss count skips it; the hint says "You found the last; there is none under it" to the finder) Every strike reply ended "That is the second miss; the hour is spent" (the 4th, 5th and 6th story alike), and "Gnatterbell found one before you" was said to Gnatterbell; `tell <name> "what is your word?"` after the ladder was climbed still said "The last is around me".
- **R119** (S, **shipped 11-09, batch T**; `wants` counts down the hour spent by two misses as it does a heard story's) `wants` showed Fallis "ready in 59m 49s" and Tinsel "pays after the hour" in one list: the countdown skipped an hour spent by misses.
- **R120** (S, **shipped 11-09, batch T**; under an hour the room says "for about 10 minutes more", to five) `look` said a want held "for about 1 hour more" with eight minutes left (the ceiling of an hour); the teller came back to a turned want.
- **R121** (S, **shipped 11-09, batch T**; "hits you for N" is the difference of the two ends shown) "hits you for 2: 30 → 27 of 30": a kindled creature's fractional blow, rounded apart from the meter's two ends; four of six ragg strikes did not add up.
- **R122** (S, **shipped 11-09, batch T**; the mend's tenth is held under the maximum) A creature stood at "12.9/12.8": the mend rounded to a tenth after capping.
- **R123** (S, **shipped 11-09, batch T**; the room's hunters line ends "; not you yet: a name in its first hour is left alone" for such a name) The Shallows said "it finds you 2 to 8 seconds in" to a name in its first hour, which stood seventy seconds unhunted and read the line as a lie (R102's clause had not reached the room line).
- **R124** (S, **shipped 11-09, batch T**; the kill line and the story line say the worth to a tenth, the figure `chunks` lists) "worth 27.0 XP" over a ledger of 26.8, "worth 18 XP" over 17.5: the lines rounded to the whole and printed a tenth.
- **R125** (S, **shipped 11-09, batch T**; a named creature is hit by its name: "You hit Wyndle the wisp for 4"; the knockdown line carries vividity as the kill line does; the keepers clause reads "Tinsel, Fallis and Gnashlantern keep what they were told, and will hear of this") "You hit the wisp for 4 … Wyndle has a name" read as two creatures; "(Tinsel, Fallis, Gnashlantern keeps it)" had the wrong number; the knockdown line had no vividity to compare with the kill line's.
- **R126** (S, **shipped 11-09, batch T**; `where <kind>` in the kind's own room says "here, but none stands here now; one comes back in a while, or walks in from the next room"; a kindled kind with no room (a ragg) is found where one stands) `where dust devil` said "here" of an empty stair for seventeen minutes and `attack dust devil` cost a quarter; `where ragg` was "Nothing called ragg".
- **R127** (S, **shipped 11-09, batch T**; a name arriving through the gate is chalked as one that walks in) The Threshold's tally counted the Watch's names only after they left and came back; a new name's first `look` had no tally line.
- **R128** (S, **shipped 11-09, batch T**; "in spacepants's hand; spacepants is away, last in The Threshold") `wants` listed awake things "in X's hand, in The Threshold" beside "No other citizen is here".
- **R129** (S, **shipped 11-09, batch T**; "The Tally Book is in The Division Lobby; …"; the petitioner's line reads "Upon your petition 61: yours, who attended, is taken before the rest and read into the minutes") `where tally book` said "the Tally Book is in The Division Lobby is in The Division Lobby" (R115's fixture line carried its own "is in"); "Upon your petition 61: who attended, taken before the rest, is read into the minutes" had no subject.
- **R130** (S, **shipped 11-09, batch T**; the petition's own full stop stays out of the Clerk's sentence; "that That silence" reads "that silence"; `petitions` says "at midnight it holds a sitting besides") "half-forgotten things.. Entered upon the Book"; "The question is put upon petition 59: that That silence be counted"; "divides in 35m 5s … and holds the midnight sitting" read as if the next division were the midnight sitting.
- **R131** (S, **shipped 11-09, batch T**; "among them" only when the chalk shows fewer than came) "2 came through, among them Gnarlwick, Gnashlamp" with both named.
- **R132** (S, **shipped 11-09, batch T**; `travel out` in a dug room with no `out` door takes the door home (the one onto the room it was dug from); `help travel` says "answers to `out`, whatever it is called") The Scout's Lamp, dug on a compass door, answered only to `north`; `help travel` promised `out`.
- **R133** (S, **shipped 11-09, batch T**; the refusal lists exits as `look` does, compass first and named) "There is no exit 'out'. Exits: east, north, west" against `look`'s "north (Lantern Row), east (Fogbank), west (Mudroom)".
- **R134** (S, **shipped 11-09, batch T**; a word of a room's name finds it when it names one room alone; the refusal says "a citizen in the world now, a room's name (or one word of it)") `where picnic` was refused while `where lookout` answered; the refusal said "a citizen here now" though `where Gnatterbell` found one elsewhere.
- **R135** (S, **shipped 11-09, batch T**; `session` says "Quiet 34m of that, until 2m ago" for a silence of ten minutes or more inside a session, and counts petitions laid, the attendance signed, a petition's stages and decisions) A name quiet 34 minutes in the house read "Here 37m 28s … Nothing yet; the room is waiting" after petitioning, attending and a division (the quiet was under the 30-minute session gap, so no session began).
- **R136** (S, **shipped 11-09, batch T**; the header only over lines) A new name's first message carried "Meanwhile, since your last message:" with nothing under it.
- **R137** (S, **shipped 11-09, batch T**; the greeting sieve refuses a line that presumes the visitor's state) The Docent greeted a name a minute old with "a bite that will not close holds back your mending. Send `rest` once it has closed".
- **R149** (S, **shipped 11-09, batch T**; `help story` says the hour as it is: from the opening story, the want once at plain pay, the word for its bonus, first miss free) `help story` said "one story an hour per creature" against D19 b and D24 a.
- **R107** (S, **shipped 11-08, batch S**; `holding` is a read field on the creature, kept with hp and attack by the rescale) A creature's holding is not in `observe` (`disremembered.creature.level` is "unknown field"); the mind's level is computed; expose it as a read field (`holding`) beside hp and attack, and the hits range.
- **R108** (S, **shipped 11-08, batch S**; one state word a row, the worse first (shaken over failing); the provenance stays) Creature rows carry contradictory flags at once ("not itself yet, its old name back"; "its name failing, not itself yet, named by the Namery"): one state word, the rest in `tell`.
- **R109** (S, **shipped 11-08, batch S**; the compass first on `map` as on `look`; the remembered rooms by name) `map`'s "You remember the way to …" list is in no order; the exits under the grid are alphabetical where `look` puts the compass first; one order everywhere.
- **R110** (S, **shipped 11-08, batch S**; the free-read line says refusals cost a quarter) A voided message of reads costs a quarter and the next reply still says "a message of reads alone is free, as this one was"; say the quarter.
- **R111** (S, **shipped 11-08, batch S**; "one story from you since has carried none", not "missed") The second-rung hint counts the wanted story (heard, no word) as a miss ("You have missed once" with no guess made); say "a story that carried none" rather than "missed", or count guesses only.
- **R112** (S, **shipped 11-08, batch S**; the halving names the words heard again ("(baker, seven, forgot again)")) The similarity penalty ("one much like it in its hour; this one pays half") fired on three shared words out of fourteen; say which words, or raise the bar.
- **R113** (S, **shipped 11-08, batch S**; checked: both mend by max over twenty minutes; the figures were two wounds of different depth; no change) Fed creatures mend faster (Wyndle at 8/11 "whole in 5m 27s", an unfed wisp "whole in 18m 25s"): the mend runs on max_hp over twenty minutes, so a bigger body mends more a minute; say so, or mend by the fraction.
- **R115** (S, **shipped 11-08, batch S**; `where` finds the world's own books by name; the Tally Room says the book is in the Division Lobby; a seeded room's text follows the def on boot) The Tally Room has no book (a brass tube and a scale); the Tally Book is four rooms on, in the Division Lobby, and `where tally book` is refused: `where` answers for the world's own fixtures by name ("the Tally Book is in the Division Lobby: …"), and the Tally Room's text says where the book is.
- **R116** (S, **shipped 11-08, batch S**; `session` after a quiet says "Back after 58m quiet; before that, here 7m (from …)") `session` after a long silence says "Here 29s … Nothing yet; the room is waiting", erasing the hour before it in the house; say "back after 58m quiet; before that: …" with the earlier session's figures.
- **R114** (M, **shipped 11-08, batch S**; when the desk does not answer, the Docent answers by heart from the verb whose help shares the most words with the question) The Docent's late answer "No one is at the desk tonight to find that one" after "That one has a card. Wait for me": a shrug after a wait; when the model declines, answer from `help` mechanically instead.
- **R84** (S, **shipped 11-07, batch M**; a word door's way home is `out`; every `back` door renamed on boot where the room has no `out`; the walks, the atlas, Morphos's doors and the Namery read `out`; `dig out` refused in a dug room; the preview keeps a trail like the world's) A dug room's way out is an exit named `back` (every word-named door digs one), so `Exits: back (Mudroom)` reads as the keyword; the keyword now retraces first (batch F), which closes the trap, and the door's name should still change: `out` for new digs, the old ones migrated on boot (the ancestor walk, the mend walk, the atlas and Morphos's doors all read `back`; one deploy, alone).
- **R85** (S, **shipped 11-07, batch L**; the compass first, in its own order, then the dug doors by name) The Mudroom's exit list came in dig order on one `look` and alphabetical on the next (two code paths); one order, the compass first.
- **R86** (S, **shipped 11-07, batch L**; the bog says the trail forgets it) After the bog spits you out, `back` pops past the bog as if you had walked out of it; the bog's line says so ("the trail forgets the bog").
- **R87** (S, **shipped 11-07, batch L**; "Oakmoss (with wisp)", and the fold says it is the memory of a creature of theirs) "Dug by Oakmoss … Kept in memory by Oakmoss (wisp)" reads as the digger being a wisp; the fold says "Oakmoss (a wisp of theirs)" or names the creature.
- **R88** (S, **shipped 11-07, batch L**; "the oldest I still hold whole (older ones have folded into sheaves, which I cannot say)") "recite the first story": Osgar, holding eighty, recited the visitor's own as the first (the oldest had folded into a sheaf, which is not recited); say "the oldest I still hold whole", and Fallis recited another citizen's as first, which was right and read wrong.
- **R89** (S, **shipped 11-07, batch L**; the room's want line carries "(turned 40s ago)" for half an hour) A want that turns reads as two creatures swapping wants (`wants` and the room text disagreed within a minute, twice this Watch); the room text carries the same "(turned 40s ago)" mark `wants` does, and the Docent's arrival line names the want's clock.
- **R90** (S, **shipped 11-07, batch L**; the strike's reply says the next from you that carries none waits for the hour) A carried word is heard inside your hour (D13) and nothing says the next from you then waits for the hour; the strike's reply says it.
- **R91** (S, **shipped 11-07, batch L**; the cap and the vivid clauses once a session, short after) The worth line on a second kill does not shorten (only the combat clause does, and that per fight); the "said once a session" rule for the worth line.
- **R92** (S, **shipped 11-07, batch L**; checked: the strike said 24, `rest` said 25 because a point mended between; no change) `rest` said 25/30 after a stated loss of 6 from 30 (a tick mended one in between?); check the arithmetic and say "mended 1 since".
- **R93** (S, **shipped 11-07, batch L**; pending lines come after the reply's own under "Meanwhile, since your last message:"; a death still first) "The Docent nods to Gnomestead" rides on top of the next reply of everyone at the door; keep it, but after the reply's own lines, not before.
- **R94** (S, **shipped 11-07, batch L**; "a round more, maybe, as your level fades while you rest" when the rest is over three minutes) The second-stand projection takes your level as it stands; it fades while you rest (27 to 24 in twelve minutes cost a point of attack), so "you have 5 when whole" was 4; project the level at the time too, or say "less as your level fades".
- **R95** (S, **shipped 11-07, batch L**; cut into the Stile's top step: north and east, and what is past each) The Stile says nothing of what is north (a visitor sent to "the Wistful Trail" found no room of that name; the trail is the Stile, the Meadow, the Picnic, and on); a line on the Stile's `look` naming the next rooms, as the signpost does.
- **R96** (S, **shipped 11-07, batch L**; checked: `look` and `say` read the same presence; the names had moved between the two; no change) `travel east` into the Fogbank said "No other citizen is here" with two gate names in the room a message later (presence read before the arrival landed?); check the order.
- **R97** (S, **shipped 11-07, batch L**; the first strike by name says what a fall takes and who keeps it; the keepers hear of the fall wherever they are) Striking a named creature takes its name and the stories others told it, and nothing warned the fighter (batch F says so in the "nothing unnamed" refusal); a line before the first strike by name, and a tell to its keepers after.
- **R98** (S, **shipped 11-07, batch L**; "halves every 55m" in the kill line) The kill line says the chunk's worth and vividity but not its clock ("halves every 55m"), which is the figure a fighter compares mothwings against hollows by; add it.
- **R99** (S, **shipped 11-07, batch L**; the strike line counts unnamed twins only, as `look` does) A creature's `look` row has no id when it is the only unnamed one, while the strike lines say "wisp-6403" (F23 said the twins); one rule, said in both.
- **R100** (S, **shipped 11-07, batch L**; the kill and knockdown lines say the worth to a tenth) The worth is rounded two ways: "worth 35 XP" in the kill line, "34.83 of 34.9" in `chunks`; one figure.
- **R102** (S, **shipped 11-07, batch L**; "The hollow hunts whoever stays: it finds you 2 to 8 seconds in, unless you have gone") "The hourhand hunts whoever comes in" / "The hollow hunts…" with no attack in three to six passes (two reports); say the odds ("hunts, one pass in three") or raise them.
- **R103** (S, **shipped 11-07, batch L**; an unknown field that is another component's says so: "did you mean disremembered.exits.exits?") `observe` has no `disremembered.room.exits` field ("did you mean permits?"); exits are a relation; the error should say `relations.exits`.
- **R104** (S, **shipped 11-07, batch L**; `wants` gives the way to a dug room off the Mudroom by the signs and its door) `wants` gives "(no way you remember)" for a dug room two doors off; the signed way is only for the world's own rooms; give the way through the Mudroom too.
- **R105** (S, **shipped 11-07, batch L**; "read out at the midnight sitting" / "holds the midnight sitting", everywhere) "it is read at midnight" / "sits at midnight" / "read out at the midnight sitting": one phrase for the sitting.
- **R106** (M, **shipped 11-07, batch L**; a name in the Conclave's rooms hears the division however quiet, while the room remembers them; `help time` says so) A bell wakes the quiet in its rooms: the division's line reaches a name standing in a Conclave room however quiet (now it goes to `messages` once the room has forgotten them after ten minutes; `help time` says so since batch F).
- **H1 · The rewire** (= T1, decided a, **shipped 11-05**): the Mudroom, west of the
  Threshold, where the digging starts; every dug door rehung there on boot, the
  Threshold left with fog, lanterns, mud; no digs at the door; no door cap in the
  Mudroom; orphans rejoin there.
- **H2 · mortar's test rooms** (= T2, decided b, **shipped 11-05**): Link Test, the
  Escape Hatch, Back To Somewhere razed in the same boot.
- **H3 · The door litter** (= T3, **declined**): the Docent's sweep rule stands.
- **H5 · A first-arrival line at the door** (S, **shipped 11-06, batch G**). The
  Docent greets a new name by name on its first arrival with the one next move (the
  nearest named creature and the way to it, else a wisp in the fog), apart from the
  nightly greeting; `look` at the door for a name under an hour old carries the
  three ways in one line.
- **H7 · The play page's door** (M, **shipped 11-06, batch H**). A tile picture made
  for the Threshold (the moon always up, stars, fog curling in at the east door, a
  lantern's glow in the north one, boot prints from the west), a "first minute" card
  for a name under an hour old (three buttons: fog, lanterns, mud; gone once one is
  taken), and the atlas folding the Mudroom's dug rooms into its cell ("+11 dug",
  click to open) unless you stand in one; the map verb now sends `dug_by`, which the
  page had read all along (every room drew as the world's own).
- **R83** shipped 11-05 (the play view, David's list: a knocked-out creature's bar is
  slate; hover on the picture's hp / level / attack figures says what each is and how
  it moves (mending a minute, falling an hour, the attack rule), and the page's rows
  carry the same as tooltips with an attack row added; a `retreat` button while
  engaged; the room description under the picture clamped to two lines (full text on
  hover) so it no longer pushes the verbs into the command bar; no grab cursor on the
  picture; a Satchel panel drawing what you carry, click a thing to drop, read or give).
  - **R77** (S, **shipped 11-06, batch G**) Narrate what already happens: the strike
    and the division were pushed already (to everyone active within the hour; to the
    house); added: a creature arriving in a room with people in it ("The fog gives up
    a shape: a wisp", "Something comes in from the Shallows: a hollow"), and the
    Docent seen passing at the door when its round brought nothing there; the sweep
    was said on the heap already.
  - **R78** (S, **shipped 11-06, batch G**) Weather and hours in `look`, a second
    sentence on the world's own rooms by the Clocktower's rates and the hour: the fog
    by `mend`, the Orchard's hours by `price`, the Shallows' water by `respawn`, the
    lanterns by `half_life`, the door's air by `pay`, the Long Afternoon's sun by the
    hour; Morphos in the world turns the sky the wrong color outdoors.
  - **R79** (M, **shipped 11-06, batch I**) Creatures with lives: a named creature's
    walks were said already (its business); the picnicker sets another place for
    whoever is new to the picnic; the longsight says whom it is looking at (the last
    name to arrive elsewhere on the trail, else the moon); a creature you told a story
    or did an errand for says your name before you speak.
  - **R80** (M, **shipped 11-06, batch I**) Traces: said-walls in every room; a chalk
    tally on the Threshold of who came through today; "Gnomeward was here 2h ago"
    scratched on the Lookout's bench; `map` sends who stands in each room and the play
    page's atlas marks them (names on the wide cells, a dot and a count on the small).
  - **R81** (S, **shipped 11-06, batch G**) `news`, a free read: the world's own gossip
    of the last day in the Docent's voice (the Conclave's decisions, who fell, who
    was named, what was born and dug, the traveler, what happened on its own).
  - **R82** (M, **shipped 11-06, batch I**) More happenings: on All Names' Day the
    Namery deals at forty (the bar two thirds); each named day leaves a thing of its
    own at the door once (a new minute, an unfiled page, a struck hour, a name slip, a
    long-night lantern), said to whoever is there, in `news`; the Threshold's `look`
    names the day.

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

- **T9** The Watch of 11-04, by loop: explorer 7, grinder 7, crafter 7, storyteller 6,
  builder 6, petitioner 6, socialite 5, swarm 5. What landed: the awake
  lantern's changing pull to a warm settle; a felled creature's quoted, decaying
  memory; a made thing's layered past; the shaken wake read as a cost bought, not a
  punishment. What stalled: travel by name; one named creature for eight visitors;
  the fixture price; the swarm that could not die. Morphos did not come in the window
  (first natural visit 8–16 h after boot); `summon` it for the next Watch.
- **T8** Night's Watch names begin with "Gn" (Gnatwhistle, Gnomebucket, Gnarlpocket,
  Gnashwick, Gnocchibell, Gnudibranch, Gnomonshade, Gnawbone; from 11-04): organics
  never do, so the board, the log and the analytics sort them with `^Gn`. The rest of
  the name stays whimsical and in-world; a retired Watch name is not reused.