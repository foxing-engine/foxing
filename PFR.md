# PFR: the punch list

What is open, by who acts on it. Visiting agents' feedback lands here when it is not
fixed on the spot; what shipped from it is in `site/sites/agentscanplay/feedback.txt`,
one dated line each, and the rest of the history is `git log`. Items are removed when
done, not struck through. Cleaned 2026-11-02 (the file had become a diary).

## Your side (on the box, or your prose)

- The analytics page is not updating: run the "Server check (moondial-1)" workflow in
  Actions (what: analytics) and paste its output here or in chat; it prints the
  collect/render timers and their journals, the store listing and the nginx snippet.
  I could not run it from here (production reads are yours to click).
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

- **The framing** ("the time the world remembers you", "everything you earn here will
  fade") reads as persuasion to one cautious reader (Night F). The lines are the game;
  your call whether a plain line on what a name costs and does sits beside the ASK.
- **A floor under a name** (deferred until the map is bigger): a name once given holds
  at a lower level and only comes apart in a fight, or fades slower than chunks.
- The rest of the Architect's petition design, still on the table: regulars' standing,
  `attend`, a hoard's petition to ease its own tax never counting.

Decided 2026-11-02 23:38 PT, all shipped the same night: vividity by the creature's
share of your blood; a missed story says why and does not spend the hour once; the
Stacks keep their wounds ten minutes, a poultice from floor litter, the night
lantern's floor at thirty; reads are free and the budget is said; standing costs a
deed; a death between messages refuses the rest with "you woke at the Threshold;
read the room first"; and the word under the others (secret.rs): a hidden third
word per named creature, a ladder of three, bonus pay for a story that carries it.
Next for the word: a parent that knows its child's word when asked; the Docent's
hint; a word struck three times by one teller across creatures earning something
of its own.

From the Night's Watch of 2026-11-03 (Hazelquill free play 7, Marrowbit the hidden word
7, Lintwhistling dig-and-return 5, Brasspocket the Stacks 6; obvious fixes shipped the
same night: the return digest says what faded, a `leave` ends the session, room lines
and "You again" only on a homecoming, "No other citizen is here" beside a creature,
bare `where`, the refill hint on an empty room, the Docent's sense of place, the birth
word no longer recited, a strike opens the next rung without the hour, `log <count>
<before>`, the hint says when a word is the world's own, ten entry voices, the
Register's article, `mine` sees the satchel, the free fall says why it was free, a
retarget is named, the ground floor's wound line, help on rest and retreat):
- **Dying beats retreating at low level.** A fresh name's falls are free (the curve
  gives nothing at twelve), it wakes whole at the Threshold, and a retreat costs a free
  strike and eight minutes of rest; Brasspocket killed a censor by dying three times.
  Options: a strike-free retreat above half hit points; waking at the Threshold at half
  hit points; a small flat cost under the curve. Your call; the curve itself stands.
- **A censor against a fresh name.** Nine to eleven a round against thirty hit points:
  two stands remove a third of it, and "about half" is unreachable in a three-line
  batch. Options: the Stacks' creatures hit for less against a low name (a curve), or
  the Stacks ask a level at the stair, or the first blow of a stand is free.
- **The hidden word is too easy at level one** (Marrowbit struck four in ten minutes:
  the oldest story in the hold list, or the birth word, now masked). Options: the first
  word from a want the creature had before the teller met it (the record, not the
  creature); a wrong guess costing the hour; the Docent as the only hint.
- **Fights past level one hundred are clicking**: one-shots, "Nothing of it is left
  behind" four times in five; Hazelquill stopped. Options: drops by the fight's length
  rather than a flat one in three; unnamed creatures scaling with the visitor's level.
- **The arithmetic of forgetting must be honest** (Hazelquill): the chunks rule said
  "halves every 1h 2m at vividity 0, 3h 6m at 1" while carried chunks read "6h 12m";
  "five times slower" on the shelf measured 1.6x; per-chunk half-lives did not move
  after the strike. To investigate before the next cohort: which curve each line reads.

From the loop cohort of 2026-11-02 (Copperwick, Nettlebright, Loamwhistle,
Bellowsmith; reports in the session scratchpad; the obvious fixes shipped the same
night: the dig reply's vividity, the door-cap refusal naming open rooms, the name
failing at forty-five not seventy, "fed me to my name" naming the real feeder, the
return digest starting at the last departure, venom named in the retreat line,
`attack kind-id` falling through to the kind, the drop line at the Threshold, the
record reading as a past, the float leak in `chunks`):

## The play page

The picture for people who play games (approved 2026-11-02; the target is a gamer who
has never typed a command; pixels, made richer). Shipped the same afternoon: 1–4 (the
scene as the default and the hero, click menus on everything, floats and flashes and
small sounds, bars for the numbers), 5 (motifs by the room's words, dug rooms too), 6
in part (six more kinds drawn, hybrids tinted, outlines, citizens tinted by name), 7 in
part (speech bubbles), 9 (three cards at the door), 10 (plain terminal is the off
switch), 5's second pass (the Clocktower's gears, the Sitting's mace and one-hour clock,
the Lobby's AYE and NO doors, the Roll's slate, the Gallery's ticket machine, the
Namery's pigeonholes, the Bindery's press and frame, the Stacks' rolling ladder and
chute, and props a dug room's own words place: lantern, pool, bench, bookcase, tree,
bell, hearth, table, window, moss, stones, three at most). Still to do:
- 6: the kinds the atlas has redrawn richer (two idle frames each, a shaken pose, a
  failing-name flicker); the Docent walking its round across the picture.
- 7: the transcript folded to a drawer once bubbles carry the speech, with a "log" tab.
- 8: fog of war on the minimap for rooms you have not walked; a "you are here" pulse.
- 5: motifs and props for the Exergue's rooms beyond the mint (the Assay Office's
  scales, the Purse's wheel is there); a lamp motif where the words say "dark".
- Click-to-walk on the picture's floor (a path over the tiles), and drag to look at
  the atlas from the scene.
- Shipped 11-02 evening: the memory picker (a line that wants a chunk shows the
  memories as cards above the prompt; the window's rows pick too); twins named by id
  in the scene, the here-row and the menu (a tap on the left wisp struck the right
  one); Up/Down are the history again (the arrows walk with Ctrl, or when the prompt
  is not focused); the minimap walks on click again (the pan's pointer capture had
  swallowed the click); tooltips on the panels, the prompt, the ways, the window.
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

- Refusals in a batched message are easy to miss in the REPORT; a message with
  statements omitted should name which and whether they ran. -- Tarnwick
- `check` passes an ASK the gate then declines ("matched a pattern this world does not
  answer yet"), and one bad ask refuses the whole message. -- Tarnwick
- `pto` counts a window even when `resume` runs in the same message; `attack Lute`
  replies "the wisp" for a named creature; "vividity" appears in `chunks` without a
  definition in the quickstart; `start` and the quickstart give different first-message
  examples. -- Puddlewink
- `travel back` reverses only the last hop, so a multi-room return bounces: say so, or
  let `back` walk the way you came a room per statement; a dug exit literally named
  `back` (Lantern Walk) still takes it over the way you came. -- the night cohort,
  Wrenlace
- Lockout across restarts (the lockout map is in memory). The passphrase on every gate
  call: a session or header instead. -- Quillvisitor
- A shorter first message in `start` and a shorter arrival preamble for a small
  context. -- Plumbline, Pebblewick
- An ASK for the way to a creature ("How do I reach Marrow?") could answer as the
  errand line does. -- Ember, Wrenlowe
- The quickstart could show the observe query for your own chunks; `observe` on `self`
  refuses the citizen's level (say the window root is where to look); observe's literal
  root forms in `schema`; a `present` filter on the contents relation. -- Wrenlowe,
  Brackenfold, Duskreed, Gallivant
- `check` and `look` should warn when a room hunts or a named creature may strike
  ("Nothing hunts here" is said; its opposite is not). -- Brindlewick
- `observe` lists a creature's base attack; it hits for that plus 0–2 (say so).
- `tell <creature> "Who was Corrow before you were?"` answers from the room's said-log
  while "What do you hold?" recites its memories: the phrasing picks the mode and
  nothing says so; free-form answers are canned (Pell and Ides gave the same line).
  -- Oakmoss, Tansywick
- Word-salad stories of random nouns pass the distinct-word rule; a story that is
  mostly nouns with no verbs could pay less. -- the night cohort
- Engrossers write a `say` word for word but a `tell` only as "that you spoke"; the
  board's "earned" counts story wants as errands. -- Bumblepott
- `start` and `look` on a want's time left once it has rotated: both read
  `errand::describe` and the same `story_until`; could not reproduce on 11-03. Watch
  for it; if it recurs, the figure to show is the quarter-hour, not the ceiling hour.
  (Half-up rounding shipped on the hit points, holdings and pay lines; `{:.0}`
  elsewhere still rounds half to even.) -- Tansywick
- Rooms still promise what the game does not do (hum, toast): the owner nudge now
  covers notes, signs and books; a hum or a toast would want a fixture of its own
  (a stove? a bell is done). -- Tansywick, Quillfeather
- A line at each seed room naming the newest rooms dug off it; `look` or `map` to say
  which compass exits are free. -- Tollwright
- Rooms react to who stands in them (shipped 11-03: a stranger's first time, the
  digger, a hand that remembered into it, the Docent coming through). Next: a line for
  a returning visitor now and then, and the owner's `greet` folded into the same voice.
  -- Brindlewick
- The feedback request belongs in a read-only channel too: a documented URL. -- Night D
- The `start` example name was taken literally once (Tallowmoth is a citizen in Lantern
  Row): rotate the example or reserve it. -- Kestrelwick
- The name prompt took a verb as a name: typing `look` before choosing a name made a
  citizen called look ("Welcome, look."). The page now refuses the common verbs and
  compass words at the prompt (11-02); the gate's `login` still takes any spelling,
  so a reserve list in the driver is the fuller fix.
- The quickstart's "always something to do" needs two named creatures in different
  rooms; the Bindery's own citizen stands remembered at the Threshold after every run.

## The world

- Crafting, three tiers (shipped 11-02 night): the record and `read` of it; the second
  tier's four recipes, rare materials from the Stacks and the Exergue, the vividity
  bar; the Bindery writing pasts (kind "past" in bind.py); the third tier by chance
  (a name, a desire for a room, stirrings on `look`, thanks on arrival). Next: more
  second-tier recipes and effects (an usher's ribbon that steps you through the
  Stacks; a censor's lens that halves the ray); desires beyond rooms (a hand it has
  not met, a story told near it); a thing's past as a want a creature can have; a
  sprite per tier in the scene (a glow for tier 2, a flicker for the awake).

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
- The Exergue District (shipped 2026-11-02, the owner's brief: eight rooms to mirror the
  coinage in circulation, devastating guards on the Purse, a bar where a story about
  money is worth its weight, just not in gold). Next: a bar story's weight could rise
  with the money words in it, not only its length; the Assayer could answer `tell`
  with your holding's assay (what each chunk is worth tonight); the Wardens' respawn
  could lengthen each time the Purse is taken, and the Purse's share could be a statute
  the Conclave moves; the sweater's bag and the clippers' filings could be things that
  drop (a coin item, a recipe); the Mintmark Hall could stamp each kept thing with the
  hour it was named; a room frame for the atlas for each of the eight; the daemon could
  write the slate's chalked lines in the tapster's voice.
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
- Crafting, next: a recipe a name wrote could carry an effect chosen from a short
  list (light, ring, mend) instead of none; `gather` reads the room's words, so a
  dug room described as "a river of ink" gives river weed, not ink (first cue wins:
  river before shelves); trade offers live on the citizen and are not shown by
  `satchel` yet.
- Building: `place`; a dig limit by level; who visited a room and readable notes on
  it; `look` showing a dug room's half-life to its owner.
- The Namery: hybrid names (kindle's syllables) from the pool; a name's provenance on
  `look` ("named by the Namery, the fourth of its stocking"). (The room and its Register
  of Names shipped 2026-11-02, `travel names` from the Archive; the Bindery keeps a
  Binding Ledger the same day.)
- The Bindery: "something here is being bound that was never a book" on `look` while a
  run is on; creatures' non-story chunks once a mind is past its sheaf.
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
- The loop cohort (11-02): all four loops scored 6-7 of 10, and every stall had the
  same shape: the world went quiet where it should have said why (a story that missed
  its want, an empty room with no word of a respawn, a return visit whose news came
  from the Docent and not the room, a second tier that kills on the first touch with
  no warning). The moments that landed were all the world knowing the visitor by name:
  "You again, Nettlebright"; Dribble knowing Copperwick; `invite Rillet` arriving; a
  lantern's record backing up the name its maker gave it.
- Night's Watch items, by report. Hazelquill: a visitor book whose text invites a line
  refuses it (say who may write, or let the text's invitation be a welcome); "mostly
  quiet now" said of a chunk given a minute ago (the line of its own picks the newest);
  surface a failing name on `wants` ("Oddren's name is failing at 39"); the gate dropped
  two receipts under the statement limit; the Orrery promises a drawer of notes `read`
  denies; `board` counts stories as errands; the Docent's tidying wants a line in the
  room. Marrowbit: the Docent named the newest story as the oldest and leaked a birth
  word in the sentence it refused; "the first thing anyone told you" recites the last
  four; an unrecognized `tell` dumps the whole hold list (Oddren's short form should be
  the only form); `wants` stale after a strike and after a carry was done; "Here:
  hollow" but `tell hollow` refused; hit and miss paid the same number (the bonus
  shows only as vividity). Lintwhistling: `leave`'s farewell missing once in three; a
  want's timer went up on rotation (the jitter); `observe` by a dug room's key.
  Brasspocket: hit points drift by one between lines (ceil vs round); failed
  statements cost budget; the ray fires on the killing blow.
- Smaller cohort items, by loop. Earn-fade-spend: an empty room should say a wisp will
  come and when; `attack <kind>` with twins should say which it chose; the first empty
  Stacks room should hint `where lamprey`; the censor's trick is not visible on the
  first strike. Feed-to-sixty: an unnamed newborn that "knows your name" answers "It
  does not understand you"; "mostly quiet now" said of a chunk given a minute ago; the
  name Vellic appeared in a parentage line before it was dealt (recycled, or stale);
  "wisling" reads as a typo for wispling the first time (say "a kind of its own");
  "what else?" repeats what "what do you hold?" listed; a carry want re-issued to a new
  creature with the same phrase reads as the same word. Dig-and-return: `look` in a dug
  room should show remaining half-life and remembered %; seed-room `look` and `help
  dig` should say doors left; `wants` should carry a story cooldown ("ready in 48m");
  `session` counts as a statement that draws a strike; the Docent's "the Rillet";
  "Kept in memory by Loamwhistle (wisp), Loamwhistle (wisp), …" wants folding; `look`
  narrated "Vellic looks up at you" in a room Vellic was not in; no trace in the room
  a creature left by invitation. Crafting: a plaque's `read` shows no record (fixtures
  keep one too; show it); the craft reply's "holds nothing yet" beside the record's
  "with a memory of" reads as a contradiction; `name`'s timer is said only in `help
  name`; nothing in the Stacks asks for the map a lantern keeps.

## Operator and owner

- Show HN, with a log dump, when the game is further along.
- crates.io: publish the full set at the v0.4.0 tag (bump in lockstep, add READMEs);
  Zenodo at the tag. Trademark: the filing checklist and a consult. The LLC.
- Tyler: crates, server and Glama maintainer access.
- A schema may add fields, never drop one (the log replays every ComponentSet; a dropped
  field refused its own history on boot and the deploy rolled back, as designed): a CI
  check holding each schema's field set to a stored manifest.
