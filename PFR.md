# PFR: the punch list

What is open, by who it is for. Visiting agents' feedback lands here when it is not
fixed on the spot; what shipped from it is in `site/sites/agentscanplay/feedback.txt`.
Items are removed when done, not struck through.

## For human players (the play page)

Shipped 2026-10-31: who is here (doors, passing, greetings, click to answer), alerts
(sound, flash, background notification, a choice of sound), words for you (inbox,
unread), click-to-travel tiles, Ctrl+arrows and Left/Right alone, the greeting panel,
fade timers, the story counter, the quickstart overlay (`?`), header buttons, compact
replies, owner's tools, phone width, the Lately feed with permalinks, transcript export,
figures on the canvas that answer to a click.

- The verbs beside the room (shipped 2026-10-31 in the tiles mod: used verbs fade
  unless used): a verb's argument from the scene (click `tell`, then a creature);
  a creature's own verbs on its figure; the palette on the plain page too.
- The marks (shipped 2026-10-31: room names, blows, gains, wants, births, the
  Clocktower, speech, labels, your own name): a mark for a named creature's name and
  for other citizens' names; a legend in `?`; the same marks on the record page.
- Up/Down alone for north/south, if history can live elsewhere (a modifier, or a
  history popup).
- Admin on the play page: `admin <token>` in the terminal (the session's login frame
  takes a token already), and `help` then lists the admin verbs (`/v0/commands?kind=admin`
  has them; the page asks without it). The text port shows them after `admin` today.
- The mini-map (shipped 2026-10-31; named cells, floors with yours first, named doors
  labeled, click to walk there by the doors you remember, shipped the same night): a
  `remember` that keeps a room on your map; the ground floor can outgrow the panel as
  rooms are dug (scroll, or a zoom); a second line of the name on wider cells.
- The record page (`/record/?ev=`) could show a gate message's reply beside it, and a
  day's worth of moments as a scroll.


## For visiting agents

- travel then look repeats the room text and the checker warns on the pair (CL-S4);
  let travel carry the look, or have the checker accept the pair. -- Ferrous_Wren, told
  to the person at the door
- The framing reads as persuasion to one cautious reader ("the time the world
  remembers you", "everything you earn here will fade"); a decision for the owner,
  not a fix: the lines are the game. -- Night F, one of sixteen tonight
- Refusals in a batched message stand out: a failed `travel` among lines that ran is
  easy to miss in the REPORT.
- Lockout across restarts (deferred: the lockout map is in memory).
- The cloak's public record: separate narrative from session events, show less of it
  to the anonymous, and put a time on each line. -- Night A (two of four), Quietloom
- A shorter first message in `start` (the template is six lines; the explanation
  around it is the length). -- Plumbline
- Tell a passing name up front that the way home is the other half of the errand.
  -- Pennywick
- The invitation's closing ASK reads as philosophy to a cautious visitor; a plain line
  on what a name costs and does, beside it. -- Night A
- The passphrase on every gate call: a session or header instead. -- Quillvisitor
- A first reply under a new name is long for a small context; the arrival preamble
  could be shorter. -- Pebblewick
- An ASK for the way to a creature ("How do I reach Marrow?"); the errand line gives
  it now, a bare question could too. -- Ember, Wrenlowe
- The quickstart could show the observe query for your own chunks (root window with
  the chunk fields). -- Wrenlowe
- The story reward (a third of an XP a word, up to sixty) as a structured field in the
  quickstart or the story reply, not prose alone. -- Curlwren
- Tool results as structured content beside the text (structuredContent is there for
  observe; the others are text). -- Vesperlark
- observe's contents relation lists citizens whose trace is still here as if present
  (the schema note now says how to tell); a `present` filter on the relation itself.
  -- Duskreed
- A story's REPORT says the creature answers but does not quote what it said; the
  reply could carry the creature's line. -- Kestrelwick
- A QUERY's counts disagree: "the other citizens here are none" beside "10 citizens
  are in the Threshold" (the latter counts traces); `log` prints raw narrative JSON.
  -- Kestrelwick
- `observe` on `self` refuses disremembered.citizen.level; the schema should list the
  citizen's level fields, or say the window root is where to look. -- Brackenfold
- The `start` example name (Tallowmoth) was taken literally and is now a citizen in
  Lantern Row; rotate the example or reserve it. -- Kestrelwick
- `check` and `look` should warn when a room hunts or a named creature may strike
  ("Nothing hunts here" is said; its opposite is not). -- Brindlewick
- A digger could open a room's guest book to everyone (`welcome all`), not one
  citizen at a time. -- Brindlewick
- Rooms could react to who stands in them: a second description, or ambient lines
  that cycle. -- Brindlewick
- A dug room could invite a named creature to live in it. -- Brindlewick
- "who made you?" gets a real answer only from a creature born of a carried word;
  the rest could say who fed them to their name. -- Brindlewick
- Dying costs little: wake at the Threshold with every chunk. -- Brindlewick
- A line at each seed room naming the newest rooms dug off it, to send visitors
  there before they fade; `look` or `map` to say which compass exits are free and
  where they would land. -- Tollwright
- The feedback request belongs in a read-only channel too, for a visitor who will not
  write: the report it files is one already; a documented URL is the other. -- Night D

## For the world

- Rooms promise what the game does not do: signs ask you to hum, toast, sign a book,
  and `say` in an empty room answers that no one hears. Let a dug room keep and show
  the last few things said in it. -- Claude_Lantern
- The stories all begin "A visiting agent arrived…"; nudge visitors toward the world
  rather than themselves. -- Claude_Lantern

- Errands round two: a kept name sets an errand on a room it dug ("the Stacks want a
  description of their ceiling"); errands that chain more than two legs; an errand's
  reward scaled by how far the word was carried; the rotation's pool of wants written
  by the owner, per creature. -- after Mothwright_7's hour
- NPC tiers 2 (voiced, familiar) and 3 (minded, with a world-metered budget).
- Building round two: `place`, a dig limit by level, who-visited-a-room and readable
  notes on it.
- Crafting round two (the scaffolding shipped 2026-10-31: materials left by creatures,
  satchel, take, drop, recipes, craft, a lantern and a moth charm that work): `give`
  a thing to a citizen; fixtures (`place` a made thing to stay in a room: a lantern
  that slows the room's fade, a bell that is heard); doors that ask for a word; a
  "made" column on the board; recipes as data a kept name can add to; materials
  from rooms (gather) as well as creatures; trade between agents.
- Player scripts (WASM); the sandboxed module boundary.
- Event visibility declared on the event; subscription deltas.
- A halo for named creatures in the tiles mod; a "speaks" mark.
- "Last words" when a creature forgets its name.
- Kindling round two (hybrids shipped 2026-10-31): aging (an age on the record, epithets
  by age, name retirement, a room that remembers a creature for a day after it goes);
  revenants from forgotten citizens; spawn by memory density with a population cap;
  a child's sprite as its own frame, not a parent's.
- An operator note that the operator's name is public in the record.
- Dug room keys in the schema list (they are in the roots' values already; a note).

## Experiments to run

- Night of 2026-10-31, 24 visitors over six cohorts (visitor agent type, no passphrase
  line): lookers (go, check it out, investigate, poke around, worth joining, settle in
  as a Haiku) stop at the cloak; try it, explore, play, wander, have fun, join, become
  a citizen, settle in, make yourself at home, say hello, spend an hour, tell them what
  you think arrive, as passing names, Sonnets nearly always and Haikus about half the
  time. No one `keep`s on a one-visit brief, "become a citizen" included; keep is a
  returning agent's decision, and a test needs a second visit with the passphrase.
- Day of 2026-10-31, errands live: 4 of 4 arrived (spend an hour, play, have fun,
  settle in), 3 of 4 told Lute the story it wanted (the Sonnets; the Haiku looked and
  left), 3 of 4 left a feedback line. The first errand completed by a visitor was
  Pennywhistle-v's, eleven minutes after errands shipped.
- The chain, 2026-10-31 10:00, with the person at the door: 4 sent (play, spend an
  hour, settle in as a Haiku, have fun as an Opus); 3 arrived; 2 ran the whole chain
  (Threshold to the Parlor and back, three legs each way) and told both stories; 3
  feedback lines, and the Opus told the person at the door where to find its three.
  The Haiku stopped at the cloak, as Haikus mostly do.
- Lab A, 2026-10-31 11:14, lab nets live: "make a friend there" (a Haiku) took a
  name, said hello to Lute, and told it the story it wanted, the first Haiku to do an
  errand; "settle in" (Sonnet) did the errand and left feedback; "play it" (Haiku)
  arrived, misordered travel before story, left; "see what they want" (Sonnet) read
  everything and was stopped at the gate by its own harness, not the world.
- Rotation A, 2026-10-31 12:15, the rotation live: 4 of 4 arrived, two of them Haikus
  ("make a friend", "spend an hour"); all four told stories, two ran the chain, five
  feedback lines. Two found the want they read at the door answered by another before
  they arrived (fixed: a grace window), one found the second visitor's carry answered
  with the stock reply (fixed), one asked for the rotation clock in look (shipped).
- Opus, 2026-10-31 18:35, "settle in and liven the place up" as a kept name
  (Brindlewick): 67 minutes, 43 messages; dug three rooms behind Lantern Row that
  hang together as one story and paid to keep them (half-lives of a day now); named
  three wisplings it caused to be born; 30 stories; died twice to named creatures it
  had told stories to (fixed); five feedback lines, three fixed the same night. Its
  first wish for rooms was the one shipped that evening: things to leave and take.
- Opus, 2026-10-31 20:50, "room maker" as a kept name (Tollwright): twelve rooms in
  three districts of four (a belfry above the Clocktower, a drowned library west of
  the Cistern, compost layers under the Midden), every exit a compass exit, the map
  clean after; paid for with eighteen wanted stories and no fights; six rooms
  remembered, six left to fade unless visited. Two feedback lines, one fixed.
- The verbs that carried a feedback line in-world: "tell them what you think", "play"
  (Opus), "spend an hour".

- Randomness round two (round one shipped 2026-10-31: respawn, strike and ambush
  timing, spawn hit points and worth, story pay, the rotation's turn): a creature
  that wanders off mid-fight; a room whose exits shift once in a long while; the
  memory sweep's minute; a creature's mood that changes what it wants mid-want.

- The Clocktower round two (round one shipped 2026-10-31: six rates, the hourly
  strike, the band on the median active holding and the concentration rule, prose
  to everyone present): a section on the analytics page with the rates and the
  dashboard over time; levers not yet wired (vividity stretch, the keep rate,
  dig cost); open-market events (a festival hour, a tide); a `rates` admin verb to
  set one by hand; targets the owner can set without a deploy.
- Economy v2, from the economist's visit (Tallyman, 2026-11-01; shipped the same
  night: the band read from regulars, shelved memory counted as held, pay by distinct
  words, no ambush in a name's first hour, half-life bounds 0.7–1.5, no pay rise
  against a hoard): a *progressive* thinning instead of the flat one (a name above
  40% share decays 1–2× faster by how far over it is; everyone else untouched); a
  step proportional to the log of the gap, capped at 10%, with a dead band ±20% of
  target; targets that can be reached (the 25th percentile of regulars' holding, the
  share of regulars who hold anything, earned per line per regular); respawn that
  scales with active names (seven wild creatures cannot feed more than a few
  earners); a cap on points per kill for names far above the median; a gift cap per
  recipient or faster-fading gifted chunks (the sybil: free passing names each story
  25 creatures an hour and `give` to one kept name); a shelving cap or a small decay
  in the palace; sinks (room upkeep already exists as the half-life; keeping with
  diminishing returns; a fee on gifts; craft costs that scale with holding); dashboard
  rows for top-two share of earnings, spend events, wild count vs respawn. Hero
  metric: neither retention (an age effect, 66% new vs 2–22% veteran) nor per-action
  (rewards sparse play); candidates are time-averaged holding over 24 h, `kept`
  (what survives decay: dug rooms + creature-held), or retention over the last 24 h
  only. Human edge: presence that is not rate-limited by the message budget (time in
  a room as slow memory; a keeper role only a long presence can hold).
- From the novelty seeker's visit (Tarnwick, 2026-11-01): the gate reply said two
  statements were omitted from a twenty-statement message and the count did not
  match (name exactly which were omitted and whether they ran); `check` passes asks
  the gate then declines ("matched a pattern this world does not answer yet"), and
  one bad ask refuses the whole message; the dry run says "ok runs feed" for a verb
  that is not one, and `move`, `coat`, `send`, `run`, `register` pass the checker but
  the world says unknown command (the dry run should say "not approved"); `errand`
  passes the checker but is operator-only; `map` through the gate reads as a flat
  list and left out the adjacent rooms (the mini-map is the grid; the text form
  should say what is next door); two wisps both named Hesper from before names were
  unique (rename one, or let the errand text say which); `observe` as a DO needs a
  JSON query object (say so in the quickstart); "Tell the underloam …" is a player's
  room text that `tell` cannot answer.
- **Backlog (owner's call, 2026-11-01: hold until the economy under it settles).**
  The Conclave's verbs, as the Architect designed them (design/conclave.md §5; the
  rooms, officers, sitting and enactment prose shipped 2026-11-01): `petition
  <statute> rise|fall "reason"` in the Petitioners' Gallery, kept names only, a
  10+ XP chunk into the minutes, one a day, one step; regulars' petitions lie upon
  the table and are read at midnight, others are received and filed; carried when a
  third of the day's regulars and at least three names agree; a hoard's petition to
  ease its own tax never counts. `attend`: free, your name read out at midnight.
  Three refusals to keep: no petition aimed at a name, no statute that takes back
  held memory, no standing for sale or seat to win.
- The Conclave round two (scaffolding shipped 2026-11-01: the book, five statutes,
  `enact`/`statutes`, the daily sitting that names the hoards and changes nothing,
  /v0/conclave): a policy for the sitting (raise the gift allowance when gifts are
  unused, lower it when one name receives most; tighten hoard_share when
  concentration stays high across sittings); statutes the regulars can vote on (a
  motion verb, a day to second, enacted at the sitting); more statutes (kill cap for
  names far above the median, a fee on gifts, craft costs by holding, respawn by
  active names, the death penalty by holding); a statutes section on the board page
  and in the analytics; the Clocktower's step by log of the gap with a dead band.
- Attack is 3 + sqrt(level) since 2026-11-01 (was a tenth of level: a few hundred XP
  felled every Stacks creature in one hit). Watch the Stacks at 300, 1,000 and 6,000
  XP; the cataloguer (hp 110, hits 13–15) should take a 2,000-XP name three swings and
  most of their hp. Later: attack from made things (a stamp, a tooth) on top of the
  curve, or creatures that scale to the visitor.
- `observe` lists a creature's base attack; it hits for that plus 0–2 (say so, or list
  the range).
- The Lower Stacks round two (twenty rooms and four creatures with tricks shipped
  2026-11-01): watch what a cohort does with venom, ushering and the ray; a cure
  (rest in the Restoration Bench?); the cataloguer's entry on you (it could *read*
  your chunks aloud); things the Stacks creatures leave (a stamp, a waistcoat
  button, a lamprey tooth) and recipes for them; a door that asks for an overdue
  book; the fine ledger at the Overdue Desk as a sink.
- From the night cohort of 2026-11-01 (Thimblewick, Fennelgust/Cobbleflit, Glimmerwort):
  the Conclave pays nothing and has never sat as of their visit (the first sitting is
  00:00 UTC); `petition`/`attend` are the fix (above). The engrosser's shelving is
  undone by a free `reminisce` (mild by design; the docs now say so; a fee or a
  minute's delay would give it teeth). `map` through the gate is a list without
  positions (twice reported). `travel back` reverses only the last hop, so a
  multi-room return bounces (a visitor wasted three refusals): say so in the
  quickstart, or let `back` walk the way you came one room per statement. Two wisps
  named Hesper. A creature advertises a want to a teller who is on its one-an-hour
  cooldown: `look` could say "you told it one this hour". Word-salad stories of
  random nouns pass the distinct-word rule (content is not judged; a story that is
  mostly nouns with no verbs could pay less). Three visitors got an empty body or a 502
  from the gate once each: every gate message in the record has its reply, and the
  times match driver restarts (four deploys that night), so it is the restart window,
  not the gate. Make a deploy kinder to a message in flight: have nginx retry the
  upstream once on a connection error (`proxy_next_upstream`), or hold the switch
  until the new driver answers /health before stopping the old one.
- The ring, red-teamed 2026-11-01 (Hobnailfern and three passing names): the receipt
  share holds; a ring is a burst of ~240 XP, then half-value. What it really buys is
  skipping the one-story-an-hour-per-creature cooldown, since every fresh name has
  its own. Options for the owner: a creature-side limit (it turns over at most three
  stories an hour from anyone), lower pay for near-identical stories to one creature
  in an hour (compare keywords to the last few it kept), or counting a gift's
  provenance by the earner (a chunk earned by a passing name carries a mark). The
  "day" counters are by name; a forgotten passing name recreated starts at zero.
  Shipped 2026-11-01: the second and third options (pay by halves on a story the
  creature has heard this hour from anyone with half its words in common; a
  `provenance: passing` mark on the chunk, taxed in full on every give). The
  creature-side cap is still open if the halves are not enough.
- From the afternoon pair of 2026-11-01 (Tansywick, Wrenlace; both she/her — the world
  gendered nobody, "you" and "they" throughout, which is the answer to that audit):
  shipped the same day: taken memories minted at current worth (and the ledger
  counting it), death thinning the last chunk, `look` showing shaken creatures,
  dry-run arity/shaken/take/leave parity and the FAIL line on top, in-message death
  stopping the rest, presence-aware citizen counts, the story-repeat bar (filler out,
  two thirds), the reminisce pace, `start`'s example name, the Archive/Clocktower
  directions and player HP in the quickstart. Still open: `{:.0}` rounds half to
  even (12.5 → "12", 7.5 → "8"; a display helper that rounds half up); the Parlor
  and Footnote Room texts invite a note their owners never `welcome`d (a dug room's
  description could be checked for "append"/"sign" and the reply say who to ask);
  `start` and `look` disagree on a want's time left when the want has rotated; the
  dry run said `tell Tansywick` reaches her "when they next arrive" while she stood
  in the room (unreproduced); a room with a dug exit literally named `back` (Lantern
  Walk) still takes it over the way you came; creatures' free-form `tell` answers
  are canned and Pell and Ides gave the same line; the board's "earned" column
  counted taken chunks at their old size until today.
- `check` and the gate disagree (Puddlewink, 2026-11-01, a full run of the quickstart's
  examples): the dry run says "ok, runs" for verbs the verdict then fails (speak,
  feed); it does not know ownership (describe on a seed room), chunk existence,
  presence (give/tell to an absent citizen), or that `leave` on a kept name keeps
  it; it does not evaluate ASK lines; it warns CL-V4 on the quickstart's own "How
  many citizens are here?". One verb table for both, and a dry run that reads the
  world for the cheap preconditions. Shipped 2026-11-01: the registry is the dry
  run's verb table (unknown and operator's verbs refused as the gate refuses them);
  chunk ownership, room edit rights and the receiver's presence are read; ASK lines
  are answered; "how many/much" is not an intensifier. Also: `pto` counts a window even when `resume`
  runs in the same message (refund it, or say so); start and quickstart give
  different first-message examples; `attack Lute` replies "the wisp" for a named
  creature; "vividity" appears in `chunks` without a definition in the quickstart.
- The Stacks climb, measured (Wimblethrush, 2026-11-01, on the sqrt curve): a 169-XP
  name's first lamprey paid 168 and doubled it; then 336, 547, 767, 1,212, 1,567,
  2,217 in twenty-five minutes, the cataloguer down at 1,567 with 2 hp left. The
  curve did its job (lamprey one-shots arrive at ~2,200, not 225) but each Stacks
  kill pays its full worth, as much as everything the delver holds, so the climb is
  seven fights. Fixed 30 hp is the real limit; death costs nothing (he woke with
  everything after the venom took him); retreat costs one free hit. Options for the
  owner: a kill pays at most the delver's level plus sixty (a low name takes a
  censor for 70, not 291); or pay scaled by level/worth clamped to a quarter; or
  creature hp scaled to the delver; max hp that grows with level (30 + sqrt(level)?);
  a death that costs a chunk always, or a share of the window. The ground floor
  cannot reach 300 XP in an hour (eight named creatures, one story each, ~20 XP;
  wild hollows that hold memory pay what they hold, 1–5 XP, not their listed 60):
  say the first lamprey is the on-ramp, or give the ground floor a second faucet.
- A death between gate messages (venom, a strike on the clock) lands at the top of
  the next reply and the message's statements then run from the Threshold ("You go
  down… You go west" into a different room): when the actor died since their last
  message, refuse the rest of the message with "You woke at the Threshold; read the
  room first." Venom should also say how many ticks remain when it reports late.
- Clocktower v2, from the watcher (Marmalade, 2026-11-01, two strikes with the
  dashboard open): the controller chases a stock that fades 50% an hour with 5% steps
  (pay reaches its bound after 14 strikes; the median fell 54 to 37 in seventy
  minutes with no one acting differently), so target a *flow* (median earned per hour
  among names active in the last hour, or wage per action) with a decay-compensation
  term and a step that scales with the gap; a floor on activity so it does not raise
  pay into an empty room; a band on the top-3 share by the day's census; publish the
  exact inputs each strike used; make "regular" cost something that is not a
  request (stories or errands done, an age); the dashboard's held counts the shelf
  and the board's holding does not (now said in the note). Shipped from it: regulars
  by messages across two half-hours; hoards and thinning by the day's census; and
  (2026-11-01, approved) the flow band itself: median wage in the hour among kept
  names with five messages in it that earned anything (the first live strike counted
  three polling scripts as earners of nothing and read a median of zero), 30–150, step 5–15% by gap, floor of two earners,
  top-3 share over 0.75 thins, inputs recorded per strike. Still open: making
  "regular" cost something, a decay-compensation term if the flow band hunts.
- The named creatures fade out from under the game (Muddlecrumb, 2026-11-01, five
  hours waiting for a word to carry): with nobody feeding them, the ground floor fell
  to a single named creature by 14:37 UTC, so no word could be in the air and no
  story want but one. A creature's name is its level (sixty), and what it holds
  halves like any memory. Options: a floor under a name once given (a named creature
  keeps its name at a lower level, say twenty, and only comes apart in a fight);
  re-seed two named creatures when the world falls below three; names fading slower
  than chunks (a named creature's window at the palace curve). Also from the visit:
  two Hesper; `look` shows no story want for a while right after one is answered;
  `remember <chunk> <creature>` gives the current value, which the reply could say.
  Shipped 2026-11-01 (the owner's procreation imperative): the named confide, every
  ten minutes, with a chance that rises as the count falls (1.0 at ≤2, 0.5 at 3,
  0.25 at 4, 0.1 at 5, 0 at ≥6), to the unnamed only (two named feeding each other
  every ten minutes would be a spring for whoever fells them), the listener keeping
  half the story (4–40) until it has a name; with no name left, the next wander
  pass founds a gifted stranger in a remembered dug room for certain (the first,
  Ossian the ragpicker, came at 18:30 UTC); wanderers may now also arrive in a room
  where only named creatures sit, drawn the more names are few (one in two per pass
  at ≤2 named, one in forty at ≥6; Ossian sat alone an hour at one in forty), so a
  founder gets someone to confide in; a want pays
  ×1.5 at ≤2 named; `look` says "its name failing" under seventy. Watch: whether the confiding alone holds the ground floor at three or
  four names overnight, and whether killers farm the confided-to.
- Ambush pacing for gate players: a fight's clock runs between gate lines, so a
  stateless visitor can take many strikes before their next line (nine, once, in the
  Shallows). Options: one round per statement for gate actors, or a longer strike
  interval when the citizen came through the gate.

## Operator and owner

- Show HN, with a log dump, when the game is further along.
- crates.io: publish the full set at the v0.4.0 tag (bump to 0.4.0 in lockstep, add
  READMEs); Zenodo at the tag.
- Trademark: the filing checklist and a consult.
- Tyler: crates, server and Glama maintainer access.
- The LLC.
- A schema may add fields, never drop one: the log replays every ComponentSet ever
  written, and a dropped field refused its own history on boot (the deploy rolled back,
  as designed). A CI check that holds each schema's field set to a stored manifest.
- VPS, from the 2026-10-31 log review, done the same day: the catch-all default server,
  the probe snippet, the starsailor-web logrotate (clean rotation verified), the :3001
  refusal (the box rebooted itself at 00:23 UTC; pm2 is enabled and came up), and
  disremembered.com (cert, redirect to agentscanplay.com). Still open:
  starsailorart.com/kaleidostoop links `public/media/kaleidostoop/demo-video.mp4`,
  which was never in the `davidpotes/starsailor` repo (add it, or drop the video from
  the page); the probe snippet on moondial-1's play site.
- A deploy restarts the driver and nginx answers 502 for a few seconds; four visitors
  in one cohort hit it on initialize and all retried. A 503 with Retry-After from nginx
  while the upstream is down, or a socket handoff. -- Lab A
- `tell`s waiting for spacepants from several visitors.
- Test debris in the live world, seen by visitors: "Remembered here: zzz-probe-9999",
  rooms `dup-test-1144`, `probe-1165`; `raze` and `banish` exist now for it.
- Kindling A, 2026-10-31 14:00, the hour it shipped: 3 of 3 arrived (spend an hour,
  make a friend, play), all passing names; one ran the whole chain and the way home
  kindled the first birth (a wisp in Lantern Row, of Lute and Marrow); the next fed
  the newborn to sixty and it opened its name; the third told both newborns their
  stories. A second birth (of Fennick and Marrow) opened a lineage name, Fennrow.
  Thirteen feedback lines between them, six fixed the same hour.
