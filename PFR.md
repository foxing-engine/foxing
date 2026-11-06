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
- **D13 · The hidden word and the hour** (Gnomebucket: the hint arrived after the
  one-an-hour lock closed the only creature; S; **ships by policy**, a). a) a story
  that carries the hidden word is heard even within the hour, paying only the bonus
  (**lean**); b) the hint is withheld until the hour is open again; c) leave it.
- **D17 · Does the fade bite in a session?** (loop cohort 11-05: a vivid chunk fell 28 →
  26.65 in thirty minutes; "gentle loss, not pressure"; the spend never felt urgent;
  M; **waits**). a) shorten the low-vividity half-life so a safe kill's chunk halves
  inside an hour (42 m → 20 m) and leave vivid ones long, so the near-loss is the
  thing that lasts; b) a "gone in N minutes" line on `chunks` and `rest` for the
  chunk nearest to lapsing, no curve change; c) leave it: the fade is for the week,
  not the visit.
- **D14 · Named creatures near the hub** (every report: one named creature, Stythe,
  shared by eight visitors, three stories an hour; S; **ships by policy**, a). a) the
  world keeps three named creatures within two rooms of the Threshold, naming a
  wanderer when fewer stand (**lean**); b) per-visitor story credit on a creature; c)
  leave it to the players' feeding.

Schema and framing
- **D8 · A creature's holding as a field** (Tyler #10; S; **ships by policy**, a). a) a public `holding` on
  `disremembered.mind`, refreshed on every change, so `observe`, `where` and `wants`
  can sort by it (**lean**); b) prose on purpose: names are found by walking.
- **D9 · The framing** ("the time the world remembers you" read as persuasion to one
  cautious reader; S; **ships by policy**, a). a) a plain line on what a name costs and does beside the ASK
  (**lean**); b) the lines are the game, leave them.

## Ready to ship (R): all ship on the Night's Watch by policy, in Claude's batches

Each is designed and needs no decision. Say `R<n> hold` to keep one back.

- **R1** (S) `back` walks the way you came a room per statement, and a dug exit may not
  be named `back` (Lantern Walk's is renamed). -- night cohort, Wrenlace
- **R2** (S) The gate's lockout survives a restart (it is in memory). -- Quillvisitor
- **R4** (S) `observe` says a creature hits for its attack plus 0–2. The reply to
  `tell <creature>` says which mode answered (the room's said-log, or what it holds);
  free-form answers vary by creature. -- Oakmoss, Tansywick
- **R5** (S) The board's "earned" stops counting story wants as errands; engrossers
  write a `tell` as they write a `say`. -- Bumblepott, Hazelquill
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
- **R22** (M; was B2, pulled 11-04) The strip room-aware (retarget the spend step when the creature is gone; tick "travel east" only on east); "Type help" pointing at the strip; the verb bar refreshing after `dig`; one `help` voice (the page's list and the world's line).
- **R25** (M; was B5, pulled 11-04) A verb's argument from the scene (click `tell`, then a creature); a creature's own verbs on its figure; the palette on the plain page; marks for named creatures and citizens, a legend in `?`, the same marks on the record page; the record page showing a gate message's reply beside it and a day as a scroll; admin on the play page; a `remember` that keeps a room on your map; the favicon on the apex site's other pages.
- **R26** (M; was B6, pulled 11-04) Unreproduced: a `travel <room name>` walk that stops short; "You again" on `leave` (rare); `board` at ~4 s.
From the Night's Watch of 11-05 (the Trail's first night):
- **R42** (M; D14 covers it, ships with D14) The storyteller's first hour: a visitor who does not fight has no named creature to tell a story to when the world's names have all lapsed; lean: a pilot named creature that never loses its name (the Docent's cat?) with a standing want, somewhere quiet. -- Gnibblewick
- **R43** (S) `tell <unnamed kind>` says in one line that only named creatures answer and how one gets a name. -- Gnibblewick
- **R46** (S) A wounded wisp that wanders off between messages reads as a despawn ("beaten to 0.5 hp, vanished when I paused 20s"); lean: a creature engaged within the last minute does not wander, and `look` says "the wisp drifted off, 0.5 left" when it did. -- Gnudgeon
- **R47** (S) The budget line jittered 54 → 59 between messages; check that refused messages are not counted and say so in the line ("a refused message is free"). -- Gnudgeon
- **R48** (S) One status line with hp, level and attack together (`rest` has hp, `chunks` has level, the attack rule is in neither); lean: `rest` and `look` say "hp 24/30, level 62, you hit for 10". -- Gnudgeon
- **R49** (S) The repeated "Nothing of it is left behind this time; a longer fight leaves more" line on every one-blow kill; lean: say it once a session, then "Nothing left." -- Gnatterling
- **R50** (S) `drop`'s "the Docent carries it to the Threshold on its round" overpromises: it takes a second sighting of an empty room, ten minutes apart while a visitor is at the door; say "in time" or say the rule. -- Gnatterling
- **R53** (S) The Clocktower's strike and the Conclave's division narrated as pushed lines to whoever stands in the house or the tower when they fire (the strike changed the Tally Room's paper in silence). -- Gnomonclock
- **R54** (S) `went()` for a named exit that is not a noun ("You take the abstain."): exits ending in a verb-like word read "You go by the abstain" or the digger names it; lean: `"You take the {exit} door."` when the word is not in a short noun list. -- Gnomonclock
- **R55** (S) The Docent's login hint ("you asked where things are, so begin with `map`") fires after `time`/`petitions`; the Tally Book's "Nothing has been entered" above entries; `help time` says what the strike and the division do. -- Gnomonclock
- **R56** (S) `rest` at full says "Full in 20s" (the eta rounds up past whole); "You have 30/30. You are whole." when hp ≥ max − 0.5. -- Gnomeward
- **R57** (S) The Lookout said "nobody on it but you" while another walker stood on the trail (quiet marks?); check `is_here` against the five-minute quiet rule and list quiet names as "(quiet)". -- Gnomeward
- **R58** (S) The Fogbank says nothing about hunting either way; a room with creatures that do not hunt says "Nothing here hunts; the wisps keep to themselves." -- Gnomonclock
- **R59** (S) The Docent's greeting cites a question the newcomer never asked ("you asked how to get rag thread", "you asked where things are"): the refusal filter catches "you asked" only in the greeting it sees; check the asked-event lookup is by the greeted name, not the latest asker. -- Gnashgarden, Gnomonclock
- **R60** (S) The on-ramp sends a level-0 name into contested wisps or lethal hollows; the Orchard's mothwings are the right first grind: the Docent's first-hour advice and `help attack` say so, and the Fogbank's "drift in within about Ns" says "or try the mothwings in the Orchard" when other names are fighting there. -- Gnashgarden
From the loop cohort of 11-05 (one tester per loop):
- **R64** (S) A big read first in a batch (`log 40 events`) fills the reply and the other statements' answers are omitted; cap each statement's quoted output in a batch (say 1,200 characters) so the others fit. -- Gnotwhisper
- **R65** (S) `tell <name> "what …"` is keyword-greedy: "what word did X carry?" matched "what" and gave the holdings; match the holdings question on "hold/keep/carry" only, and answer a "what word" question from the carried-word record. -- Gnotwhisper
- **R66** (S) `retreat` while a hunter is closing but not yet engaged says "You are not fighting anything"; say "It has not reached you; `travel` leaves before it does." -- Gnawlight
- **R67** (S) The drop line reads as a contradiction when a one-round kill drops and a three-round one does not (chance by rounds, sure at six); say the rounds: "two rounds: one chance in three; sure at six". Folds R49. -- Gnawlight
- **R68** (S) The secret word's hint ("in something I wanted once") should narrow after a miss (which story, then which sentence) so one story an hour is not a blind guess. -- Gnotwhisper
- **R70** (M) The two-stand censor is unwinnable as the loop sells it: full heal takes 11 m, the wound lasts 10, and 67 hp at ~6 a round against ~10 back is not two stands at thirty hit points; lean: a wound kept twenty minutes in the Stacks, and `retreat` says in rounds what a second stand would take ("it would take you nine more rounds; you have four in you"). -- Gnibbleforge
- **R71** (S) `give` to a creature or the Docent says plainly that only a name at a keyboard or through the gate can receive, and `look` marks who can; the censor's ray says what it took in the strike line ("chunk-12 halved: your hits land for 5 now"). -- Gnibbleforge
- **R72** (S) Three things called "the hour": the message budget's rolling hour, a creature's story hour, a want's hour; name each ("the budget's hour", "its hour with you"). -- Gnibbleforge
- **R73** (S) A Docent answer that never came ("finds you within a few minutes"): check the asked queue for a dropped question when the worker restarts across a deploy, and say so to the asker if a question is lost. -- Gnibbleforge, Gnomonclock
Pulled from the backlog 11-05 ("B19 yes, B21 yes, B22 yes"), each in pieces the Watch can take a batch at a time:
- **R74** (M; B19) Crafting, second tier: an usher's ribbon (a thing that lets you choose the room an usher shows you to) and a censor's lens (halves the ray) as recipes from Stacks drops; desires beyond rooms (a hand not met, a story told near it); a thing's past as a want; a sprite per tier on the page; `gather` reading the first cue ("a river of ink" gives ink).
- **R75** (M; B21) The Exergue, next: a bar story's weight by its money words; the Assayer answering `tell` with your holding's assay; the Wardens' respawn lengthening each taking; the Purse's share as a statute; the sweater's bag and the clippers' filings as things; the Mintmark Hall stamping kept things with their hour; atlas frames for the eight; the slate in the tapster's voice.
- **R76** (S; B22) The Docent, next: a second figure for the Lower Stacks (an Under-Docent at the stair); "what have you found?" listing the Lost and Found; a `thanks` that costs nothing; `recipes` not saying "You lack rag thread" the moment the thread was spent.
- **R44** (S) The Fogbank's "something will drift in within about N" was 52s, 29s, 84s across looks; see R31 (same root). -- Gnarlbog, Gnibblewick
From the Night's Watch of 11-04:
- **R28** (S) `wants` lists the awake things with their current pull; the ground
  hint and the "awake" tag agree after a thing has settled. -- Gnocchibell
- **R30** (S) A quoted quote inside `tell`/`say` text does not void the message; the
  refusal names the token. -- Gnudibranch
- **R31** (S) The "something will drift in within about N" estimate matched the
  respawn job but visitors waited longer: check that free reads tick the scheduler,
  and say the room's figure, not the next job. -- Gnawbone
- **R34** (S) The play page's shaken state: the hit-chance in the status bar and a
  tremor on the figure while shaken. -- Gnatwhistle
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

## Small fixes queued for the Night's Watch (F): ship by rule, no nod needed

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
- **Y3** Issues: Tyler's #6, #8, #9, #11 are D1, D2, D1, D6 (shipped 11-05, in the
  ledger; closed with a signed reply when you say so); #10 is D8, still queued. Glama's maintenance grade
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