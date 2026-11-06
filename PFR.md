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

From the Night's Watch of 11-04 (the Gn- cohort)

Schema and framing

## Ready to ship (R): all ship on the Night's Watch by policy, in Claude's batches

Each is designed and needs no decision. Say `R<n> hold` to keep one back.

- **R10** (M) The arithmetic of forgetting, audited: which curve each line reads
  (`chunks` said "halves every 1h 2m at vividity 0" while a carried chunk read "6h
  12m"; "five times slower" measured 1.6x; per-chunk half-lives did not move after the
  strike). A report, then the fixes. -- Hazelquill

Away from Moondial and Entropic (the engine's and the game's old names): done in the
tree 11-06 (R16–R20); the box's half is Y1. The `MOONDIAL_*` fallbacks in the driver
and the tools go in a release after Y1 has run.
- **R26** (M; was B6, pulled 11-04) Unreproduced: a `travel <room name>` walk that stops short; "You again" on `leave` (rare); `board` at ~4 s.
Left from batch D (the play page's bigger pieces):
- **R12b** (M) The transcript folded to a drawer with a "log" tab once bubbles carry the speech.
- **R13b** (S) A sprite per thing a creature leaves (icons are by word now).
- **R22b** (S) One `help` voice (the page's list and the world's line).
- **R25b** (M) A verb's argument from the scene (click `tell`, then a creature); a creature's own verbs on its figure; the palette on the plain page; the same marks on the record page; the record page showing a gate message's reply beside it and a day as a scroll; admin on the play page; a `remember` that keeps a room on your map.
From the Night's Watch of 11-05 (the Trail's first night):
From the loop cohort of 11-05 (one tester per loop):
- **R70** (M) The two-stand censor is unwinnable as the loop sells it: full heal takes 11 m, the wound lasts 10, and 67 hp at ~6 a round against ~10 back is not two stands at thirty hit points; lean: a wound kept twenty minutes in the Stacks, and `retreat` says in rounds what a second stand would take ("it would take you nine more rounds; you have four in you"). -- Gnibbleforge
Pulled from the backlog 11-05 ("B19 yes, B21 yes, B22 yes"), each in pieces the Watch can take a batch at a time:
- **R74** (M; B19) Crafting, second tier: an usher's ribbon (a thing that lets you choose the room an usher shows you to) and a censor's lens (halves the ray) as recipes from Stacks drops; desires beyond rooms (a hand not met, a story told near it); a thing's past as a want; a sprite per tier on the page; `gather` reading the first cue ("a river of ink" gives ink).
- **R75** (M; B21) The Exergue, next: a bar story's weight by its money words; the Assayer answering `tell` with your holding's assay; the Wardens' respawn lengthening each taking; the Purse's share as a statute; the sweater's bag and the clippers' filings as things; the Mintmark Hall stamping kept things with their hour; atlas frames for the eight; the slate in the tapster's voice.
- **R76** (S; B22) The Docent, next: a second figure for the Lower Stacks (an Under-Docent at the stair); "what have you found?" listing the Lost and Found; a `thanks` that costs nothing; `recipes` not saying "You lack rag thread" the moment the thread was spent.
From the Night's Watch of 11-04:
- Kept on purpose: `nginx-moondial.conf` (moondial.ai redirects to foxing.dev), the
  `/clear/entropic.yaml` redirect, the postgres schema-lock constant, the host's name
  moondial-1, TRADEMARK.md's history line, the repo's local folder name.

## The front door (H): the Threshold redesign, step by step

Your project (11-05): "an awesome front door for players". Asked in chat as T1–T3 that
night (the cohort T's below are another list); kept here as H. Each later step is
yours to call: say `H5 go`, or `H5 brief` for a design first.

- **H1 · The rewire** (= T1, decided a, **shipped 11-05**): the Mudroom, west of the
  Threshold, where the digging starts; every dug door rehung there on boot, the
  Threshold left with fog, lanterns, mud; no digs at the door; no door cap in the
  Mudroom; orphans rejoin there.
- **H2 · mortar's test rooms** (= T2, decided b, **shipped 11-05**): Link Test, the
  Escape Hatch, Back To Somewhere razed in the same boot.
- **H3 · The door litter** (= T3, **declined**): the Docent's sweep rule stands.
- **H4 · The Threshold's text as a front door** (S). Rewritten to present the three
  ways as three invitations (fight in the fog, read under the lanterns, dig in the
  mud) and name the one thing to do first; the Mudroom's text to match.
- **H5 · A first-arrival line at the door** (S). The Docent greets a new name by
  name on its first arrival with the one next move (the fog, a wisp), apart from the
  nightly greeting; `look` at the door for a name under an hour old carries the
  three ways in one line.
- **H6 · A signpost at the door** (S). Like the Trail's: `read signpost` lists the
  nearest named creature, the newest dug room, what is wanted, by the signed way.
- **H7 · The play page's door** (M). A tile picture made for the Threshold (the
  moon, the three doors), a "first minute" card for a name under an hour old (three
  buttons: fog, lanterns, mud), the map legend folding dug rooms under the Mudroom.
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
  - **R77** (S, by policy) Narrate what already happens: the Clocktower's strike and the
    division as pushed lines to the house and the tower (folds R53); the Docent seen
    passing on its round; a creature arriving ("Something comes in from the Shallows:
    a hollow"); the sweep landing on the heap said to whoever is in the Midden.
  - **R78** (S, by policy) Weather and hours in `look`: a second sentence that varies by
    the Clocktower's rates and the hour (the Fogbank thick or thin by `mend`, the
    Orchard's hours ripe by `price`, the Long Afternoon's sun a hand's width off);
    Morphos's visit changes the sky.
  - **R79** (M, by policy) Creatures with lives: wandering said aloud ("the wisp drifts
    east"); the picnicker sets another place when someone new arrives; the longsight
    says who it is looking at; a creature that remembers you greets you by name before
    you speak.
  - **R80** (M, by policy) Traces: said-walls in every room (the last few things said,
    not only dug rooms); a chalk tally on the Threshold of who came through today;
    "Gnomeward was here 2h ago" on the Lookout's bench; the board's names drawn on the
    atlas when they are in the world.
  - **R81** (S, by policy) `news`: the world's own gossip in the Docent's voice (the last
    division's decisions, who fell, who was named, what was dug, Morphos sighted), so a
    returning name can tell the world lived while they were away.
  - **R82** (M, by policy) More happenings: a Naming Day in the calendar when the Namery
    deals cheaply; a festival on each named holiday with a thing of its own (a lantern
    on the Long Night, an unfiled page on the Day of Unfiled Things).
  - **X14** (XL, brief wanted) Day and night: which creatures hunt by the hour (hollows
    by night, sighs by day), the page's light, the Fogbank's fog by the hour.
  - **X15** (XL, brief wanted) A travelling market: the Exergue's tapster walks a circuit
    of the ground floor, the slate with it, trade where it stands.

- **R83** shipped 11-05 (the play view, David's list: a knocked-out creature's bar is
  slate; hover on the picture's hp / level / attack figures says what each is and how
  it moves (mending a minute, falling an hour, the attack rule), and the page's rows
  carry the same as tooltips with an attack row added; a `retreat` button while
  engaged; the room description under the picture clamped to two lines (full text on
  hover) so it no longer pushes the verbs into the command bar; no grab cursor on the
  picture; a Satchel panel drawing what you carry, click a thing to drop, read or give).

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