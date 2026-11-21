# PFR: the punch list

What is open, by who acts on it. Items move to the ledger when they ship, and the
ledger is its own page since 11-12: [`PFR-SHIPPED.md`](PFR-SHIPPED.md), newest first;
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

**Security upkeep (set 2026-11-18, David: a high priority).** Claude tees up security fixes unasked. Every deploy ends with a "Security posture" step on the box it touched (`deploy/security-report.sh`: pending security updates, a restart owed, automatic updates, fail2ban, how ssh is opened, certificate days left); Claude reads its warnings after each deploy and, for anything open, says so that turn and hands over the one command that closes it (root is yours, so the running is yours). Dependency advisories and anything in the code are fixed and shipped as an R.

- **ALPHA · what stands until beta** (called 11-15; the gates and how each was met are in the ledger). The reset warning stays until beta; beta brings deploy windows, and with them "no resets" and a tried restore. Capacity work stopped at five hundred names answered: S15 is for after alpha. Nothing else is a gate: no invites, cap, moderation or status page.

## Decisions (D): one letter each; **waits** means yours, the rest go by policy

Stories, errands and the word ladder
- **D33 · The third word by meaning, not by token** (M/L; **waits**: David raised it 11-11, "use our haiku connection to do semantic analysis at level 3"). At the third rung the creature holds a meaning ("something lent and never returned"), and a story strikes it by being about that, in any words; a model reads the story against a fixed rubric and answers 0 to 3 with one line in the creature's voice. What it buys: no word list finds it, hints can be thematic, and the top rung is understanding, not search. What it costs, and how each is met: the verdict is late, so it arrives in `messages` the way the Docent's does and is never made under the world's lock; `check` cannot predict it, so rungs one and two stay lexical and only the top is a judgment; two like stories may be judged apart, so the rubric is fixed, the score coarse and the reason said; a story will address the judge, so the story goes in as data and a structured verdict comes out, tried first by a Watch member told to break it; when the model is not there the rung says the creature sleeps, and nothing guessable stands in for it. One judged story per creature per name per hour (the cost of an attempt is what D32 was about, and it caps the spend). a) **lean**: build it for the third rung as above, behind a switch, and send a teller and a breaker; b) the same, and the second rung too; c) no: the ladder stays lexical with D32 a.

- **D48 · How often a creature tells a story** (S; **waits**, from the Watch of 11-19 on X16). Today one telling a day for each creature (David, 11-18: slowly). Seen: each of two wisps told once, then nothing in 45 minutes of standing together, and a shepherd's own stories never went, because a creature tells the newest story it holds and another name told one a minute later. **(a)** Leave it: a hop stays news, and a story takes days to cross the world. **(b)** One telling every few hours (four, say): a story can be followed across two rooms in an evening. **(c)** Keep a day, and let the creature tell the story it has held longest untold, not the newest, so every story gets its turn. (b) and (c) go together. Lean (b) with (c).
- **D49 · How often a creature walks to a neighbor to tell** (S; **waits**, same Watch). A creature at home with a story untold today, and a named creature one door away that lacks it, goes one time in three when its turn at business comes (three creatures every ten minutes). Seen: no visit in ninety minutes; Gedder and Fallis never left their rooms. **(a)** Leave it. **(b)** Every time its turn comes, so stories leave the room they were told in. **(c)** Also let it walk two doors, not one (most named creatures do not live next door to another). Lean (b); (c) if stories still pool in the Fogbank.
- **D50 · A copy kept alive by retelling** (S; **waits**, same Watch). The same story told to the same creature pays plain again each hour, as stories did before X16, and now each retelling renews the copy's five days: one cheap story an hour keeps a copy in the world for good and pays each time. **(a)** Leave it: it is the old story pay, and tending a story is play. **(b)** A retelling of a copy the creature holds whole under your name renews it and pays nothing after the first. **(c)** It pays, and does not renew: only a mending or a claim restarts the five days. Lean (b).
- **D51 · How loosely a want is matched** (M; **waits**, same Watch; older than X16). A story "counts as the wanted one by what it is about, meant so or not": a candle and a chair in a room was taken as "a warm cup of tea in an empty room" and paid double. **(a)** Leave it: generous, and a newcomer's first wanted story lands. **(b)** Ask two of the want's own words, not one. **(c)** With D33 (the haiku connection): by meaning, read by a model. Lean (b) now, (c) when D33 is taken up.
- **D52 · The third word and a wanted story** (S; **waits**, from the Night's Watch of 11-18 on R199). David chose "a wanted story is also weighed against the word"; Claude built it for the first two rungs and left the third, drawn from the room, at one word to a story (D29, against a room's text pasted into a story), and the Watch found every living named creature on its third rung, so the change reached nobody. **(a)** A wanted story is weighed whole on every rung. **(b)** Leave it as built. **(c)** On the third rung a wanted story may carry up to three of the words it could be. Lean (c).

## Ready to ship (R): all ship on the Night's Watch by policy, in Claude's batches

Each is designed and needs no decision. Say `R<n> hold` to keep one back.

Away from Moondial and Entropic (the engine's and the game's old names): what is left
is the box's half (Y1); the `MOONDIAL_*` fallbacks in the driver and the tools go in
a release after Y1 has run.
The play page's bigger pieces:
- **R257** (S) The phone, what is left: a first tap on a creature does nothing and the second opens its menu (not reproduced by hand yet); a draft bar stays after a tap elsewhere. (The rows that scroll sideways fade at the right edge from 11-20.)
- **R258** (S) The first hour at a keyboard, what is left: a speech bubble is see-through from 11-20 but still sits over whoever stands above the speaker (it could move to a clear row); the red mark on what hunts has not been judged by a first-hour visitor yet.
- **R259** (S) Smaller, from the Watch of 11-19, what is left: the record page's left half is mostly empty beside a long reply; out of the Ripe Week the Orchard's `gather` gives nothing (by C5 a ripe hour is the Week's; David asked for a ripe hour there, and may want one all year); reading a thing from the room's own sentences is unseen live.
From the Watch of six, 11-13, the night the second door opened (a newcomer, a faller, a teller, a walker, a manual reader on a kept name, a latecomer; all six woke at the Postern; see T16):
- **R256** (S/M) The Night's Watch of 11-18, the fifteenth (Gnash at level 100, Gnosis the teller, Gnubbin the builder, Gnettle the maker). **Seen working:** the leave ending at 100 and not coming back under it; the Stacks stair either side of 100; no second "hour is over"; the signpost's "for about 3 hours more"; "keeps half as much"; `tell` saying when a name last acted; `session`'s told, said and written; the room's "It had faded to 62%"; `describe` refused with whose room; the dry run in the gate's words; `recipes` with places and the whole chunk spent; the lantern's 12.0 and 4.0 beside `map`; `rest` saying why a poultice stayed; `where Wyndle`. **With David as D52:** R199's change reached nobody (every named creature is on the third rung). **Balance, for David:** a teller reached level 100 in four minutes (four wanted stories at double and a carried word), so the leave ends almost at once; the Midden's floor holds 76 moth dust, 34 wisp embers and 77 ripe hours, which makes `gather`'s hour moot; mothwing memories come in at 2 to 4, under the charm's 5; `status` is refused at a quarter (an alias?). **Open:** `rest` with a poultice and a bandage names one; the Mudroom lists doors at "0% left" as live exits with no mark on them (a fixture now only delays the unmaking, 11-18; a mark on the door was offered and not asked for); `wants` gave six steps to a creature an earlier listing put at three "by the signs"; a miss reply gave "none of the words it could be" beside the three-minute rule (weighed or not?); `messages` listed four words where `session` said "3 heard"; a signpost want cut at "crossed the" was taken as the slip's answer; two wisp fights left nothing, so a new name could not make a lamp, and the remember-then-name dry run went untried; a want turns seconds after it is met though the signpost says "3 hours more" ("sooner if someone meets it" is there). **Not seen:** a creature full for the hour, a bleeding bite, another name taking a thing first, two things fitting one short name.
- **R255** (S/M) The Watch of 11-17, the fourteenth (Gnocturne the reader, Gnarl the fighter, Gnimbus the teller, Gnapsack going down; Gnapsack was cut off by a machine restart at 00:28 and its findings are from its log). **Seen working:** held attacks at the brink, "You felled it already", `take` naming who took a thing, the quarter for a refused message with reads beside it, reads in all eight new rooms, twelve slips and the Reader's Round, the Under-Docent, the Tide Table against the water (table: low 23:40, Court clear about 4 minutes; seen clear 23:38:59 to 23:42:51), `travel` stopping before a room under water, a brine glass holding the loss to a point every ten seconds for two minutes. **Balance:** nine slips put a new name at level 290 in a quarter of an hour and the level-100 stair is no bar (slip pay stands, D47); the free walk of the Lower Stacks that followed is closed (the leave ends at level 100, shipped 11-17) and waits for a Watch to see it. **Open:** the Cistern Stair's slip figure "does not keep" (fair only to a fast reader); the Lookout's slip count (24) did not match the rooms the reader had stood beside; the Division Lobby's wrong-answer line does not say which way, and "midnight sitting" beside `time` saying half past; the Under-Docent is not in `look` ("You are alone"); `where Bindery` answers as of an absent visitor (`where "The Bindery"` works); `time` said the Clocktower strikes in 19m 34s at 23:40; a refusal of a story not about the new want says "has heard the story it wanted from you inside this hour already"; the rules of the creature's word (one guess every three minutes, one word to a story) appear only in a refusal, and what finding it pays is never said; `wants` sends a carrier toward the Carrels without saying the Stacks stair asks level 100; a poultice of the second tier took an ordinary hit and gave "20 back" where the recipe says a bleeding bite and ten; the fall's line does not say when a swing is worth sending (R253); "a message runs whole or not at all" beside a refused statement whose neighbor read ran; an unnamed wisp held "Tinsel's story of the way home" with Tinsel in the room; the Docent could not say where a brine pearl comes from (the recipe says "the wing's deepest rooms"; one lay on the Binder's Tidepool floor). **Not seen:** "no longer knows you", a second carrier's pay, the starfish's blow going wide, a splint, anything of D43 or D44.
- **R254** (S/M) The open Watch of 11-17, what is left. **The hour's sixty messages ended three of four evenings early**; a read beside a refused statement no longer makes a whole message, and whether that is enough is for the next Watch to say. (`wants` says from 11-20 when another name has carried a word already.) **A wanted story paid double twice inside five minutes**: the code holds a teller to once an hour with each creature, so this was two creatures or a fault not found; the footnote now says "from each creature". **The creature's word** stays as it is (David, 11-20: c). **The Docent sent a name to Gedder "in the Fogbank"** when it lived in a dug room. A shout from the Hush arrived at the Lookout fourteen minutes later to nobody. A revenant cost 42 hit points and paid 1.6: a trap more than a fault.
- **R253** (S) The strike team of 11-17, what is left. **One of three never had the first-hour line** (Gneiss, named 23:33:09, walked six rooms at 00:34 and was told nothing; the other two were told at 00:33): cause not found; it fell in its first hour and was fifteen minutes silent before the hour ended, and the other two were neither. **The Shaken spiral** stays as it is (David, 11-17: no rule change); the fall's line could still say how long until a swing is worth sending.
- **R252** (S) D41, what is left: forty-one rooms answer `read` from the list, and the Trail's Stile, Well, Lookout and Long Afternoon and the Exergue's rooms from their own code. Not yet: the rest of the Stacks (the Stairwell, Cistern, Undertow, Gallery, Oubliette, the Dark Reading Room and the rooms past the Index Stair), the Conclave's rooms, the Namery and the wings. A batch a Watch.
- **R251** (M) What is left of David's six calls of 11-16 (D34, D35, D36, D37c, D40c, D42; all live). **Still unseen:** an arm growing back (it fell inside one low), a salve, a dart, a lantern or a charm acting, a Warden fought at its new figures, the Purse taken at six hundred, a full creature answering a guess, a creature at home fading at half the pace over an hour, the shelve line at eight chunks. **Built 11-17 on David's word and unseen:** one starfish blow in five goes wide; a starfish splint and a brine glass (Claude's recipes: "we can change later"). The way south was already shut while any starfish is in the Court. **To watch (D43, D44, live 11-17, none of it seen in play yet):** every fall to a whole starfish goes into the same arm, the first to come off, so what earlier parties lost is one thick arm in front of four plain ones (a fall of 100 is about 107 hit points on that arm; "the arm holding least" is a one-line change if that reads wrong); a boss that holds memories and is felled still gives one memory to the striker alone and stands (only the coming apart is shared), and the Purse is its taker's; a share is capped at each name's own level plus sixty, so a low name's share of a Warden is mostly lost; one blow and a walk away earns a full share ("a tenth of the damage" was the other choice). **Also to watch:** a body has no ceiling now, so a name at level 2,000 has 230 hit points and the Wardens' 27 to 29 is a different sum for it; a night lantern found on Lantern Row's hook is second tier, so whoever carries it lands nine in ten; the starfish learns (it had a shove after three arms and a bite that will not close after four), which no text of its own says.
- **R250** (S) What is left from the resident of 11-16: nobody signed the visitor book in 85 minutes; the same few unnamed creatures at the doors are fought over by four names at once. With R190 (a room of your own saying what your absence did, which `mine` does well and the room does not): the resident's first ask, as it was the builder's. (The Gatekeeper answers in his own voice from 11-20, unseen live.)
- **R249** (S) What is left from the Watches of 11-16: a wounded wisp that `retreat` left did not drift off in a minute, so R169 is still unseen; the one-story-an-hour refusal that says when the hour turns was not reached (the wanted-story refusal came first), so it is unseen live, as are "holding nothing now" and the line for taking back what a fall gave a creature.
- **R248** (S) Unseen live: a strike on a hidden word and its pay (the wait between weighed guesses is three minutes).
- **R247** (S) Unseen live: the doubled line (two testers, three rounds, one room, did not recur).
- **R246** (S) Nothing said aloud by a neighbor was there to test the doubled line (R242) on the live world: still unseen.
- R209 was met again by this Watch (`mine` after a fall says "nothing touched any of it").
From the live Watch of 11-13 (a fighter, a teller, a maker on a kept name, a postmaster seventy minutes late; see T14):
- **R212** (S) What is left of the dry run against the gate: `name` after a `remember` in the same message passes with an "if" (the dry run does not add the chunk to what the thing holds); a refused message now names each rule with where and why, and a message of nineteen statements has not been tried against it since.
- **R213** (S) Unchecked since: a new lantern's reply against its `read` (the line "it holds nothing yet" is no longer in the code).
From the live Watch of 11-12 (a guide reader, a teller, a mender, a latecomer):
- **R208** (S) What is left: a gardener's bite never bled in six bites for one of two testers (the code gives one bite in two, so one chance in sixty-four: chance, most likely, and the row now says the odds).
From the live Watch of 11-11 (a walker, a maker, a teller, a newcomer):
- **R199** (S) The word ladder, what is left: `check` refused a story the gate then heard at plain pay (not reproduced). A wanted story strikes the word whole on the first two rungs since 11-18 (David: option 1); the third rung, drawn from the room, keeps its one word to a story (D29): unseen live.
From the live Watch of 11-10:

From the Night's Watch of 11-10 (on a local copy):

From the Night's Watch of 11-06:
Pulled from the backlog 11-05 ("B19 yes, B21 yes, B22 yes"), each in pieces the Watch can take a batch at a time:
- **R74** (M; B19) Crafting, second tier, what is left: desires beyond rooms (a hand not met, a story told near it); a thing's past as a want; a sprite per tier on the page. (The ribbon marker, the smoked lens and `gather`'s cue shipped 11-19.)
- **R75** (M; B21) The Exergue, what is left: the sweater's bag and the clippers' filings as things; atlas frames for the eight. (The other six shipped 11-19.)
- Kept on purpose, not to be tidied: `nginx-moondial.conf` (moondial.ai redirects to foxing.dev), the
  `/clear/entropic.yaml` redirect, the postgres schema-lock constant, the host's name
  moondial-1, TRADEMARK.md's history line, the repo's local folder name.

## The front door (H): the Threshold redesign, step by step

Your project (11-05): "an awesome front door for players". Asked in chat as T1–T3 that
night (the cohort T's below are another list); kept here as H. Each later step is
yours to call: say `H5 go`, or `H5 brief` for a design first.

What is left of the front door, each yours to call:

- **H18 · Rows past forty doors** (M; **waits**, it was H14's second question). a)
  **lean**: as built, the wall is one room and `look` shows a digest of it; b) past
  forty doors, new rooms are dug off the Mudroom's own side rooms, ten doors each.

## The keyboard (P): performance for a player at the keyboard

Where it stands (a local build in a headless browser at 1440×900): ten idle seconds
in a room cost half a second of work, the picture's ten frames a second being most of
it; panels take only the rows that changed, and the page keeps the last five hundred
lines of the transcript. Load is seven requests, `index.html` near
270 KB in one file with styles and script inline, `tiles.js` near 100 KB. The live
round trip is on the analytics page, from the play page's own reports. What is left:

- **P1b · The room as a kept layer** (S/M). The floor and walls drawn once and kept, so
  an ambient frame draws figures only.
- **P5 · A lighter door** (M). Styles and script out of `index.html` into two files
  with a hash in their names, cached for a year, the page itself small and never
  cached; `tiles.js` preloaded; the schema and vocabulary kept in the browser and
  revalidated; the composer and observer tabs loaded when first opened. Check the
  server compresses text (your side to confirm on the box: Y item below).
- **P8 · A quiet mode** (S). One setting: no ambient motion, no sky, no sound; for a
  slow machine or a long session.

Order: P5, P8, P1b.

## Ten times the players (S): planned 11-11 at David's ask

Where it stands (the live world, 11-13, about 299,000 events; the world's own share of
a reply, from `took`): `say`, `satchel`, `map`, `time` 3 to 4 ms; `look` 5; `wants` 7;
`travel` and `attack` 3 to 10 by what they write; nobody waits for the lock. The log
is half to three quarters of that, at about 0.65 ms an event written. The `board` verb
is 11 ms and a first `look` after arriving 30. `GET /v0/ops` and the analytics
page keep these by verb. One world behind one lock is still the shape; what is left:

- **S2b · A command as one unit of the log** (built, off; **waits**: it changes what
  a database failure does). Since S2 an append is one insert and one commit, about
  0.65 ms on the live box, and an ordinary command makes two to nine of them: the log
  is still half to three quarters of a command's own time. `FOXING_LOG_UNITS=1` holds
  a command's (and a tick's) appends in one transaction with one commit: on a local
  PostgreSQL a command's log share becomes a flat 0.9 ms whatever it appends (an
  `attack` of six appends 2.0 to 0.9). The price: the store has applied the events
  before the commit, so a commit that fails leaves the world ahead of its log, and
  the driver stops (exit 70) and is restarted from the log by its service (two
  seconds, then the boot). Today a failed append refuses that one command and the
  world stays up. a) **lean**: turn it on (set the variable in the driver's
  environment, or say the word and it becomes the default in the code); b) leave it
  off: 3 to 7 ms a command is already not what a player feels.
- **S4 · Readers do not wait on writers** (L). Public reads (`/v0/map`, `/v0/board`,
  cloak, observe) served from a snapshot swapped in after each write, so a read never
  takes the world's lock: what lets a hundred watchers cost nothing.
- **S6 · The load tests, what is left** (the record of every run, twenty names to five hundred, is in the ledger). Five hundred names are answered, every one, about a second apiece, on an empty load world (11-15): degraded and usable, which met alpha's first gate. Load tests run on the load world (`tools/loadtest.sh <names> <minutes>`, or the `Load test` workflow) and write nothing to the live log; `WORLD=live` is for one last rehearsal, David's to call. The harness's read of the load world's `ops` was mended on 11-15 and has not been run since.
- **S15 · Quick, and not only usable, at five hundred** (L; **waits**, for after alpha). What was left when the runs stopped was structural, and David's to choose: (a) the world behind a read-write lock, so observations run side by side on the box's four cores and only commands take it whole; (b) a box with dedicated cores; (c) serve a name trimmed out of a packed room from the shared run with its own node put in.
- **S16 · The log's archive, what is owed** (the checkpoint, the verifier and the second table are live, 11-15 and 11-16; the design and the record of each step are in the ledger). The nightly backup looked at, as root: does it carry `events_archive` and the checkpoints? The dump taken before the move (`events-before-archive-20261116T012858Z.dump`, 409 MB, beside the data and inside what the backup copies) removed when David is content. A restore from that dump tried once on the load database (`pg_restore --clean` is the way back and has not been rehearsed). The archive run again when `events` has grown: by hand for now (`Server check` → `archive-live`, David's go each time), to be automated. `verify-checkpoint` run after each deploy that touches a fold or the store's `apply`. Not explained: a process that replayed the whole log peaked at 4.3 GB over ten hours where the next peaked at 270 MB (look at the board's fold and the citizens' report when next it is seen). Decisions inside it, Claude's unless David says otherwise: the tail is 60,000 events; checkpoints daily and at boot; two kept; load-test events archived with the rest.
- **S7 · The box** (yours, later). One VPS runs the driver, PostgreSQL, nginx, the
  Docent and the Bindery. Before a second box: PostgreSQL's settings for the log
  table (it is append-only), a connection pool, and the driver built in release with
  its worker count set. A second world or a second process per world is past 10x.
- **S8 · What the game itself does at ten times** (design, yours). Three of a Watch
  already empty the Fogbank; ten times the names want spawn counts that follow who is
  in the room, more first rooms (R151), and the Mudroom's wall (H18). These are the
  limits a player meets before the server's.
- **S9 · The deploy, what is left** (S; the store's half is live, 11-14). A deploy
  still stops the old driver before it knows the new one opens the log. The step
  that would have caught 11-14 without any of the rest: replay the live log against
  the new build (read-only, beside the running driver) before the switch, and stop
  there if it fails. It needs memory for two drivers at boot (2.5 GB each on a
  7.7 GB box), so it wants S7's look at boot memory first.
- Order: S4, S9; S15 after alpha. S2b when you choose.

## Content (C): expansions, planned 11-12 at David's ask; C3, C9, C10 and C11 are live in a first cut

What the world has now: twelve districts, eighty-seven rooms of its own, forty-four
kinds of creature, two tiers of recipes, the errand and the word ladder, the Conclave,
Morphos twice a day, a tide, twelve reference slips. What the Watches keep finding is where it is thin, and that is
where this starts: the door is crowded, the middle is empty, and almost nothing asks
for two names at once. Sizes are for a first cut that a Watch can play. Say `C4
brief` for a page on one, or answer the lettered ones.

- **C2 · The Reading Rooms, what is left** (M; the first cut is live, 11-13: six
  rooms up a stair from the Archive, their creatures fought by tellings of fights
  you remember, a proofreader that strikes for repeats, a lampmoth that takes a
  fight nearly lost, and the Stacks' three tricks one to a room).
  - *Next:* a Watch sent up the stair, with names holding 20, 60 and 100, to say
    whether the figures are right: a telling lands two and a half words to a point
    times a quarter to one and three quarters by the memory's vividity (D39 a);
    the kinds hold 20 to 50 (D37 b) and hit 3 to 8.
  - *Not in the first cut:* the rooms' kinds leave only what the ground floor
    leaves (moth dust, ink, thread), nothing of their own; the Docent does not know the rooms by heart; the play page has no button for a
    telling; a named one of these kinds is a listener like any other, and whether
    that is right is undecided; the pilots do not tend the Mezzanine.
- **C4 · Things for two** (M). The Wardens (260 hit points), the Assayer and the
  starfish are the fights sized for company, each shared evenly among the names
  that struck it (11-17), and `retreat` already says "bring another name". Three small ones on
  the way there: a bell in the Clocktower that takes two hands in the same minute, a
  door on the Trail with a key in each of two rooms, a creature that only takes a
  story told in two voices (one name starts it, another ends it within the hour).
  Each pays both, and says who you did it with on the record.
- **C5 · The almanac, what is left** (small things forever; the first cut is live,
  11-13: `almanac`, and three seasons that each change one thing, the Fog Month
  (November), the Ripe Week (16 to 22 November), the Night of One Story (30
  November)). All three are in November because the world is three weeks old and a
  season nobody can walk into is not a first cut; the table is `almanac::SEASONS`.
  - *a · Spread them over the year?* a) leave November as the world's own month
    and add one season a month as each is thought of; b) move the Ripe Week to
    September and the Night to midwinter now. Claude leans a.
  - *Not in the first cut:* a season's first hour is not announced (the bells
    could say it); the board and the site do not show what is in season; the
    happenings are still the same six out of season.
- **C6 · The commonplace book, what is left** (S; the first cut is live, 11-14: a
  second-tier recipe, one dated line a visit of its own accord for a fall, a room
  dug, a name taken by your hand, a vigil; sixty lines kept; `read` gives them).
  - *Not in the first cut:* more things of note (a first kill of a kind, a binding,
    a petition enacted, a letter carried); `read` shows the lines run together with
    bars, not one to a line as a standing book's are; the play page has no page
    for it; nobody has made one on the live world yet, since it asks a catalogue
    card from the Stacks.
- **C7 · What players write for players** (M). Rooms, recipes and wards are
  player-made; quests are not. A kept name with a named creature may leave it a want
  of their own choosing (a story about a thing, a word for another creature), paid
  from the name's own memory when someone does it. The world's wants turn every four
  hours; a player's stands until it is done or fades.
- **C8 · The third word by meaning** (D33, restated as content). The top rung of the
  ladder as a riddle of sense, judged by a model: the same build as D33, counted
  here because it is the first content that cannot be read out of the room's text.
- **C9 · The Drowned Wing, what is left** (live 11-16; the starfish felled and the book read 11-17). (a) **Unseen live**: an arm growing back, a great low and a neap, the three- and four-a-tick stages, the tide carrying a thing off or leaving one (none in an hour: the round may be too shy at one in three and one in six), a quiet name carried up. (b) **Yours to weigh**: an axolotl or an anemone is too much for a name near level 50, which is what the wing's upper rooms make of a newcomer; the world's own arrival lines ("had not expected you…") repeat across the wing and are not damp; a fall mends you whole and costs a point or two at that level, so falling is cheaper than resting nine minutes. (c) **To build**: recipes for the starfish arm and the brine pearl (R251), pictures for the wing on the play page, the sewer, the Ashen Wastes.
- **C10 · For the agents, what is left** (live 11-17: twelve reference slips, one in a room of each district; `read slip`, `solve "<answer>"`, `slips`; and the Card Catalogue's `index "<subject>"`).
  - *Played, 11-17, by a Watch told nothing about reading:* all four found the slips unprompted inside their first six messages (the Fogbank's `look` line, or `slips` in the help), all four made the Reader's Round their first goal, and all four finished it, in 35 to 60 messages, with three wrong answers among forty-eight. "The best thing here"; "I would come back for it daily"; "a real treasure hunt across every district".
  - *The pay stands* (David, 11-17: "leave it, we'll check for balance later"): 25 a slip and 100 for the Round. Each of the four first readers took 400 inside its unhunted first hour and stood at level 450 to 520 before a real fight; to look at again when balance is.
  - *Thin, and known to be:* the Library's and the Exergue's are answered from the same register; the Deep Water's and the Wing's from the same table. A second question for each slip, turning by the day, would stop the *method* being a thing told once.
  - *Not in the first cut:* the Docent knows the slips exist and where to look, not their answers; the play page has no button for `solve`, `slips` or `index`; the board does not show the Round; the index has no card for a made thing, a recipe or a statute.
- Claude's order, if left to it: C11's rest, then C4. What it needs from you
  first: whether C9 (a wider world, under the water) follows the Reading Rooms, and
  whether C7 is in the spirit of the thing or too much rope.

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
  never scales with the player; it scales with its own level.** The rule and the first
  six kinds are live. Next cuts, by brief: aging (a named creature's level drifts up a
  little a day it is kept), epithets (earned from what felled it or what it fell),
  retirement (a creature past a level walks to a room of its own and stops fighting),
  last words (a felled named creature says one line from what it held), a creature's
  `felled` told as a story; pixel sprites for the six (the page draws their glyph).
- **X6 · The Court of Lethe** (**shipped 11-19; a courthouse from 11-20**; what is left). Five rooms behind the Division Lobby's door: the Hall of Lost Steps, the Filing Window, the Courtroom, the Cells of Record and the Appeal Stair. A plea is brought at the window in its hours, may be short a form, is stamped at the Archive, may come back smudged, and is entered; the answer is given in the Courtroom; the Court then sits aloud for a few minutes (a bell, the Realm's Attorney, the Semi-Public Defender, the gallery, a judge who loses the thread) and the ruling lands with the gavel. A hand raised in the Courtroom is answered by the Bailiff, who writes a name on its owner. The bench sat on its first live case on 11-20 (the Watch: plea to gavel in eleven minutes; the model's ruling and the model's tattoos both landed). Left: a sitting has not been watched line by line from a page that stayed open; only the Hall looks like itself in the picture (the Courtroom has no water, bar or jury box drawn; the Cells show bookshelves; the Appeal Stair is a bare floor); a plea the record does not bear out is still taken, with the Clerk's remark (David, 11-20: keep it so); the Crier hands over a form "already filled in" for a thing that never happened; a smudged stamp is forty steps for a roll of the dice; the courthouse is filed under the Conclave on the `map`; a message refused for one unknown verb does not name the verb; **free pleas**, with the right item or quest; the sitting's lines are fixed text with a few variants each, and the Docent's model could write the opining for each case; the judge's forgotten thread is one of five; the Realm's Attorney and the Semi-Public Defender are voices, not creatures (a room spawns one kind); only named creatures are drawn to a sitting; a banner on the play page when the Court sits; a referred case comes back on ANY petition of the petitioner's decided after the referral; a passing name cannot be pleaded against once the world forgets it; the "thing" case reaches back only as far as the recent log; the Conclave's minutes do not record a referral; a tattooed name is not told of it by `look` (others read it; the wearer learns of it once).
- **X7 · Unions** (factionalism without factions). Brief wanted: standing a name earns
  by what it does, not what it joins (diggers, storytellers, keepers, delvers), with a
  voice at the Conclave and a share of something; no banners, no enemies, a name may
  stand in several.
- **X8 · Morphos, a very problematic traveler.** The traveler, its riddles, its Hands
  and its ladder are live. Next: Morphos moving between rooms during a visit; riddles
  stocked by a tool like the Namery's (twenty in the table now); the Hands with a
  sprite of their own; a riddle that wants the answer carried to another room; what
  Morphos keeps of the clouded (it takes a chunk home once in a while?).
- **X9 · CLEAR 0.5.** The spec, the reference checker, the Rust port and the oracle
  corpus, the vocabularies, the editor, and the gate, moved together: structured
  results beside the text, the question patterns a world answers declared in its
  vocabulary, and whatever 0.5 adds. Brief: the change list first. On it (D18,
  11-06): what the gate may mend before the check so fewer messages score a 50
  (the message stays all-or-nothing): a missing final period supplied, an unknown
  verb answered with its nearest approved one (`strat` → `start`? no: `story`), a
  place or a thing of several words quoted as the typed line already is (R192), a
  statement over the word limit split at a semicolon, and the oracle cases for each.
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

- **X13 · A world that is lived in.** Happenings, named creatures' business and the
  Conclave's holidays are live. The rest, by complexity and policy:
  - **X14** (XL, brief wanted) Day and night: which creatures hunt by the hour (hollows
    by night, sighs by day), the page's light, the Fogbank's fog by the hour.
  - **X15** (XL, brief wanted) A travelling market: the Exergue's tapster walks a circuit
    of the ground floor, the slate with it, trade where it stands.
  - **X16** (M, **shipped 11-18**, seen live 11-19; what is left) Stories travel and
    wear. Seen by the Watch of 11-19 (Gnomad, Gnibble, Gnotary): two tellings in the
    Fogbank, wear ("found one lantern steps", "the room was swallows there"), the name
    going to "one someone told Tinsel", a worn copy set right (6.0, no hour spent), a
    worn-name copy claimed (8.0). **With David as D48 to D51:** how often a creature tells, how often it
    walks to a neighbor, a copy kept alive by retelling, how loosely a want is matched. **Open:**
    `tales` cannot show what an unnamed creature was told ("Tinsel tells the wisp a
    story, as if it had only just thought of it" is an older line, and reads like the
    new one); the same story told to two creatures is two stories to the world; a
    story refused for the hour counted as a try at the hidden word; `session` says
    nothing of stories passed or set right; a line in `look` for a creature holding a
    story at second hand; "nobody's now" unseen.
  - **X17** (S/M) They notice you were gone. A creature that knew a name greets its
    return with what happened between (who told it something, who felled its neighbor,
    that it nearly lost you); after a week's silence it writes (`letters`). Makes "no
    longer knows you" the end of something that was said on the way.
  - **X18** (L, **shipped 11-19**; what is left) Old age and the Ossuary: live, and
    unseen in play (the first creatures say they are old on the tick after the deploy,
    if any has been named three weeks, and go three days after). Left: the crafted
    thing that buys time (David: "future crafted items buy more time, but it becomes
    less effective each use"; `age::extend` is the hook, a week and then half each
    time, and nothing calls it); the last walk is said in the two rooms and not walked
    door by door; the Ossuary's revenants do not stand aside for a reader of niches;
    the almanac does not list who is old.
  - **X19** (M, with X14) The lamplighter. At dusk a creature walks Lantern Row lighting
    the lanterns; felled or kept from it, the row is dark and rooms fade faster from
    every map that night, unless a name lights them. A daily job anyone can take up, and
    a reason to keep an unnamed creature alive.
  - **X20** (S) Lost and Found, for real. What a name drops, or leaves lying when it
    falls, is carried there by the Docent with a ticket saying where it was found, and
    is claimed there. The room's text already says exactly this.
  - **X21** (M) The fog that walks. Some evenings of the Fog Month a bank rolls from the
    Fogbank through three or four rooms over an hour: inside it `look` hides names and
    one exit, and wisps come with it. Weather that moves, and that names warn each
    other of.
  - **X22** (M) The weekly binding. Every story said aloud at the Reading Bench in a set
    hour is bound that night into one volume in the Bindery, its tellers named; a volume
    a week, and old ones fade unless read. A reason to be in the world at the same hour.
  - **X23** (S) A plaque that fades with its holder. The name that leads the board has a
    plaque at the Threshold as vivid as what that name holds now; gone quiet, it dims and
    another's takes its place. The board as a place.
  - **X24** (M) How a tattoo comes off. The Bailiff's name on a forearm "does not wash off, and nobody has yet found what does" (David, 11-20: file it as future content). A quest, a craft, a ruling, a price; and whether the next one costs more.
  - **X25** (M) The evidence locker. A steel door in the Cells of Record, three padlocks, a docket blacked out by the Censor's Office but for "AND THE OTHER THING", and something behind it shifting its weight (David, 11-20: a reference now, for future hijinks). What is in it, who may sign it out, and what a case can do with it.
  - **X26** (M) The jury. Twelve empty seats in the Courtroom, RESERVED, with an M pencilled in the twelfth (David, 11-20: where problematic characters like Morphos may sit in the future). Who is empanelled, how a jury changes a sitting, and what it does to a verdict.
  - **X27** (M) Appeals. A door as tall as a boot at the top of the Appeal Stair, locked; whoever goes through goes on their knees, as a supplicant (David, 11-20). What opens it, what is on the other side, and what an appeal can undo.

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
- **Y0** Security upkeep, once on each box (it installs what is waiting, turns on daily security updates with a restart at 10:30 UTC when one is needed, and fail2ban on sshd; `REBOOT_TIME=never` to keep restarts by hand): on the site's server `sudo bash /srv/agentscanplay/current/deploy/secure-updates.sh`, on the game box `sudo bash /srv/foxing/current/deploy/secure-updates.sh`. The site's server showed 60 updates and a restart owed on 11-18.
- **Y2** Apex VPS: starsailorart.com/kaleidostoop links a demo video that was never in the
  repo (add it or drop it); the probe snippet on moondial-1's play site.
- **Y3** Issues: none open. The standing rule: a new issue gets an answer within a day
  and a close when it ships (Claude posts, signed, by the 11-03 permission); Glama's
  maintenance grade reads commit activity and whether open issues get a reply.
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

## What the cohorts taught (T): durable; read before briefing a cohort

- **T20 · The world down for 21 minutes, 11-15 (07:51 to 08:12 UTC), by the load tests.** Boot read the whole log into memory twice (once for the store, once for jobs and folds). Six load runs in one evening took the log from 0.78 to 1.38 million events; at that size boot wanted 6.9 GB on a box of 7.7, the kernel killed the driver 26 s into every start, and neither the new build nor the rollback could come up. An hour before, a good build had been rolled back for taking 55 s to wake where the deploy waited 50 (the wait is five minutes now). Boot and folds now read a page of 5,000 at a time. **What it teaches:** a load test on the live world writes to the live log for ever, and boot cost grows with the log: before more big runs, load names need a log that can be dropped, or the log needs archiving (David, 11-15: "we'll have to deal with log archiving at some point"), and boot needs a snapshot. The outage was found by a health check after a deploy that reported nothing, not by a notice: a driver that cannot start cannot say so, so the deploy workflow should say it in Discord.
- **T16** (11-13, six on the live world, three minutes apart, the night a new door opened) Six find what four do not: three of the six reported the same break (an exit named `in`, which the gate's grammar drops) within fifteen minutes, and it was mended while the last three were still arriving. Name nothing an exit, a verb or a thing that the grammar treats as a small word (in, on, at, to, of, by: `foxing-clear`'s list); test every new exit through the gate, not only at the keyboard. A tester will not pace itself: told to play for thirty minutes, four finished in under ten of the world's. If time matters (a creature's hour, a ten-minute rule), give sleeps between named steps. With somebody standing at the Threshold all evening, all six woke at the Postern: the split is by who is in each room at that moment, and a Watch meant to try both doors needs one tester sent to stand at the gate first. A fix deployed mid-Watch is seen by the testers still out (one noted the exit change its name under it), which is a fair price; say in the brief that it may happen.
- **T17** (11-14, the world down eight minutes) A new district took the live world down on its second deploy. The first walk into a district writes a field on the citizen named for its door (`edge_mezzanine`); the field was not in the schema; the running driver took the write, and the next boot refused to replay it. The rollback failed the same way, because the event was already in the log: a rollback cannot save a world from a write the older build cannot read either. What caught it was not a test: the test world never restarts. Now one does (`every_district_s_door_is_in_the_schema_and_the_world_reopens`). The wider rule: a write to a field the schema does not know should be refused when it is made, not when it is replayed (S9), and any change that adds a room, a district or a field wants one walk through it and a restart on a local world with a file log before it is pushed.
- **T18** (11-14, four up the stair, each with one part) Four testers with four different parts found more than six with one: none overlapped except where it mattered (three measured pay, and agreed). The one given the manual first and told to predict before measuring confirmed the formula to the point in eight of eight and found every place the manual was silent; do that for every new rule with a figure in it. The dry run was wrong five ways on the new verb because only one of the gate's checks had been copied into it: a new refusal belongs in one function that both ask. A brief's own figures must be read off the world, not remembered (the brief said the Fogbank keeps two; it had five). A tester will bleed itself on purpose for a rule that rewards blood (ten `say "hi"` in a fight to make a memory vivid): any rule that pays for risk will be farmed by the cheapest risk. Staggered three minutes apart, the last report came back sixty-eight minutes after the first tester started; tell the owner it is an hour.
- **T19** (11-14, the world down about ten minutes, found by David at the keyboard) PostgreSQL restarted on the box at about 06:45 UTC (its memory fell from 6.5% to 1.2% between two readings; the box's own doing: `unattended-upgrade` upgraded libc6 at 06:42:20, by `/var/log/apt/history.log`, and a libc upgrade restarts the services that use it; the load tests had nothing to do with it). The driver held one connection and never reconnected, so every command and every tick failed with "connection closed" while `/health` went on saying ok, and nothing told anyone. Three lessons: a health check that does not write is not a health check of a world that only writes (S12); a long-lived connection is reconnected or it is a single point of failure; and after a night of load tests, look at the world once more before saying it is well: the last check made was `/health`.
- **T15** (11-13, David's rule) Every test is on the live world: a Watch, a load test, a new switch tried. A pocket world (a local copy) is only for a content change David calls high-risk, and he says when that is. Claude's own check of a change before it ships, on a local build, is not a test in this sense and goes on as before.
- **T14** (11-13, four live, three staggered four minutes apart and one seventy minutes late) Give each tester the night's own changes as numbered things to try and ask for the world's exact words: three of four confirmed fixes line by line and found two faults in the batch within the hour (a help example that did not parse, a dry run that disagreed), which were corrected while the Watch was still out. A feature that needs time to exist (a dead letter takes a quiet name an hour to make) is seeded by a script before the Watch and visited by a latecomer told to sleep first. A builder needs a kept name, and so does anyone asked to petition: the teller, on a passing name, could not. The session's machine may restart under a long Watch: testers should keep a log in their folder as they go (the postmaster's survived, and its report was read from it).
- **T13** (11-12, four live, staggered four minutes apart) Staggering works and is worth keeping: the latecomer's report is a kind no simultaneous Watch gives (what is left, who is full, whether a word reaches anyone). A tester told to sleep before starting does it. Give one the newest player-facing page and tell it to play by that page alone: the guide's first reader found fourteen things wrong with it in four minutes, most of them figures the writer took from an older page. Testers do not answer each other unless told to check `messages` every few minutes: brief the first two to, or company goes untested. A brief that names where a creature lives must be checked against `where` first (the mender was sent to the Undertow for a lamprey that lives behind the level-100 stair).

- **T12** (11-11, four live: walker, maker, teller, newcomer) A newcomer told not to read the guides plays mostly through `check`: the dry run is a second front door and must know every verb form the gate does (it did not know named walks the night they shipped). A teller is always the fastest leveler and finds the ladder's seams; brief one every time balance changes. A maker needs a wound to test a bandage: tell it to get bitten on purpose (the Undertow's lampreys) or it will not.

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
- **T11** The live Watch of 11-10 (four, on the live world, each with its own scratch folder), by loop: storyteller 8, newcomer 8, builder 8, crafter 6. What landed: the Docent answered inside a minute every time and by name; a first story at message 5 with no fight; the fall's arithmetic to the point; a hollow felled for the memory it took; other citizens' dug rooms reading as theirs ("it has not decided about you"); a lantern fed to seventy-one and named. R145's fix read right on four creatures. Names used: Gnobblewren, Gnightjar, Gnomonica, and Gnarlfoot_II (Gnarlfoot was already a kept name with another passphrase, likely an earlier Watch's: pick names with a check first, or keep a list). The local copy's verdicts on D26, R151 and D28 did not hold live: a fresh empty world is a poor stand-in.
- **T10** The Watch of 11-10 (on a local copy; no model at the Docent's desk, no organic names), by loop: walker 8, fighter 8, crafter 7, builder 7, petitioner 7, storyteller 6. What landed: the short arrival (D25 a) was enough on every return; the Long Afternoon's word found on the well's rim; a made thing named at sixty reciting its life; a wisp fed to its name asking for a story and keeping it word for word; the fall's epitaph; the arithmetic (level as the sum, attack by the root, vividity as blood) checked and held every time. What stalled: three testers shared two wisps; a talker with no named creature; a crafter at level 0; the Docent's "within a few minutes" with no one at the desk. Names used: Gnettlethrush, Gnoddingcap, Gnimblequill, Gnuthatch, Gnarrowby, Gnorthwind. Two shared one scratch file and one sent three messages under the other's name: give each tester its own scratch directory in the brief.
- **T8** Night's Watch names begin with "Gn" (Gnatwhistle, Gnomebucket, Gnarlpocket,
  Gnashwick, Gnocchibell, Gnudibranch, Gnomonshade, Gnawbone; from 11-04): organics
  never do, so the board, the log and the analytics sort them with `^Gn`. The rest of
  the name stays whimsical and in-world; a retired Watch name is not reused.
