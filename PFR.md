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

## Decisions (D): one letter each; **waits** means yours, the rest go by policy

Stories, errands and the word ladder
- **D33 · The third word by meaning, not by token** (M/L; **waits**: David raised it 11-11, "use our haiku connection to do semantic analysis at level 3"). At the third rung the creature holds a meaning ("something lent and never returned"), and a story strikes it by being about that, in any words; a model reads the story against a fixed rubric and answers 0 to 3 with one line in the creature's voice. What it buys: no word list finds it, hints can be thematic, and the top rung is understanding, not search. What it costs, and how each is met: the verdict is late, so it arrives in `messages` the way the Docent's does and is never made under the world's lock; `check` cannot predict it, so rungs one and two stay lexical and only the top is a judgment; two like stories may be judged apart, so the rubric is fixed, the score coarse and the reason said; a story will address the judge, so the story goes in as data and a structured verdict comes out, tried first by a Watch member told to break it; when the model is not there the rung says the creature sleeps, and nothing guessable stands in for it. One judged story per creature per name per hour (the cost of an attempt is what D32 was about, and it caps the spend). a) **lean**: build it for the third rung as above, behind a switch, and send a teller and a breaker; b) the same, and the second rung too; c) no: the ladder stays lexical with D32 a.

Economy and death

## Ready to ship (R): all ship on the Night's Watch by policy, in Claude's batches

Each is designed and needs no decision. Say `R<n> hold` to keep one back.

Away from Moondial and Entropic (the engine's and the game's old names): what is left
is the box's half (Y1); the `MOONDIAL_*` fallbacks in the driver and the tools go in
a release after Y1 has run.
Left over, smaller pieces of larger things:
- **R76b** (M) An Under-Docent at the Stacks' stair: a second figure for the Lower Stacks.
- **R10b** (S) The shelf's "five times slower" measured 1.6x over an interval (it is a power law: half in 5x at first, a quarter at 15x). Still to check: per-chunk half-lives after a strike (none should move: a strike makes a new chunk).
The play page's bigger pieces:
- **R12b** (M) The transcript folded to a drawer with a "log" tab once bubbles carry the speech.
- **R13b** (S) A sprite per thing a creature leaves (icons are by word now).
- **R22b** (S) One `help` voice (the page's list and the world's line).
- **R25b** (M) A verb's argument from the scene (click `tell`, then a creature); a creature's own verbs on its figure; the palette on the plain page; the same marks on the record page; the record page showing a gate message's reply beside it and a day as a scroll; admin on the play page; a `remember` that keeps a room on your map.
From the Night's Watch of 11-08:
- **R138** (S) The Tally Room's sign gave the Gallery's way to the Division Lobby ("north, north, west"; from the Tally Room it is north, west, north, west). The sign is right now and points at `where tally book`; what is left is a sign that computes its way, as the Stile's step does.
- **R140** (S; looked at 11-10, open: the source is built in one place (errand.rs, "carrying <sender>'s word to <receiver>"), so the two clauses are likely two memories, one carried each way; kept for a Watch to reproduce) A dug room's "Dug by Thistlecroft with the memory of carrying Corrowkin's word to Corrow" and "Kept in memory by Thistlecroft (with carrying Corrow's word to Corrowkin)" flip the word's direction between clauses: one of the two source strings is built backward.
- **R142** (S) The Docent, asked two things ("Where is the tally book, and what does travel back do?"), answers by heart from one verb's help and drops the other; by-heart (R114) should split on "and"/"," and answer each clause, or say which it is answering.
- **R144** (S) The Docent suggests `tell Wyndle "what is your word?"` minutes after Wyndle's name failed (it is wisp-6579); the by-heart and the model's briefing should read the creature's current display.
From the Watch of six, 11-13, the night the second door opened (a newcomer, a faller, a teller, a walker, a manual reader on a kept name, a latecomer; all six woke at the Postern; see T16):
- **R218** (S) The dry run and the gate chose different ways of the same length (Threshold to the Orchard Wall: "north, lane, orchard, wall" in `check`, "north, lane, gate, wall" walked): one route search, one tie-break. And a named walk's reply, and its dry run, still list the doors taken and not the rooms passed (R193).
- **R220** (S) `wants` gives a creature in a dug room a way "by the signs" ("5 steps: lane, row, south, west, annex") that `where Osgar` and `travel "The Annex of Second Readings"` then refuse ("you remember no way there from here") until the room has been walked: one rule for all three.
- **R221** (S) What is left: the chalk tally lists a name as "came through" that has never stood at the Threshold (it counts arrivals at either door). (The newborn's row says it is newborn and for how long, 11-14.)
- **R222** (S) What the Postern still lacks beside the Threshold: the Docent in person, the day's name, a tally. The quickstart's list of signposts leaves the Postern's out, and its signpost says nothing of how far the Threshold is. `where mothwing` from the Postern names the Wall and not the Orchard; `where bench` offers two other benches before the Reading Bench.
- **R223** (S) The cloak shows the Threshold whatever door a name will wake at, and says so in a line; it could show the door a new name would wake at just then, or both in brief.
- **R224** (S) An absent name gets two answers: `tell Gnashfall` (away) "They will hear it when they return", `tell Gnewelpost` (gone, forgotten on leaving) "There is no one called Gnewelpost", at a quarter, and `where` answers with the whole catalogue of what it can find. (`tell` says "The world has forgotten <name>" of one it knew lately, 11-14.) `where <name>` recommends `log`, and `log` takes a count, not a name. A word said in a room after you left it reaches you only if you walk back ("Said here lately"), never through `messages`.
- **R225** (S) A felled unnamed wisp "gets up again, holding 17" where a mothwing comes apart, and nothing says why. (An empty room says where else something mild stands, 11-14.)
- **R226** (S) A long message's reply is cut "under the statement limit"; it now says what was cut (receipts first, then the last answers) and that every statement ran. What is left: cut the answers to reads before the answers to acts, which wants each line to know its statement.
- **R227** (S) The quickstart, what is left: `reply` to a letter whose writer is forgotten too is not described; the first letter a visitor sees may be no. 2.
- **R239** (S) Small things left from the Watch of 11-14: a fall's line "Nobody will say your name at The Proof Room…" reads as a riddle to a newcomer.
- **R244** (S) `messages` stamps a creature's answers "just now" however old they are, beside a Docent line rightly "8m 15s ago" (looked at 11-14: both rows take the event's own time, so the cause is upstream of the stamp; not found yet). A wisp that holds a memory of yours shows in `look` with no id and cannot be aimed at by id. The Fogbank's signpost says "an arm for each way off" and lists the compass ways only.
- **T20 · The world down for 21 minutes, 11-15 (07:51 to 08:12 UTC), by the load tests.** Boot read the whole log into memory twice (once for the store, once for jobs and folds). Six load runs in one evening took the log from 0.78 to 1.38 million events; at that size boot wanted 6.9 GB on a box of 7.7, the kernel killed the driver 26 s into every start, and neither the new build nor the rollback could come up. An hour before, a good build had been rolled back for taking 55 s to wake where the deploy waited 50 (the wait is five minutes now). Boot and folds now read a page of 5,000 at a time. **What it teaches:** a load test on the live world writes to the live log for ever, and boot cost grows with the log: before more big runs, load names need a log that can be dropped, or the log needs archiving (David, 11-15: "we'll have to deal with log archiving at some point"), and boot needs a snapshot. The outage was found by a health check after a deploy that reported nothing, not by a notice: a driver that cannot start cannot say so, so the deploy workflow should say it in Discord.
- **T21 · The load world, and the log's archive (David, 11-15).** (1) **Load tests run on a load world**: a second driver on the box, its own PostgreSQL database dropped and made at every start, ports 7787 and 7788, held to 2.5 GB and a lesser share of the processors; `deploy/server/setup-load-world.sh` once as root (David: yes), then `tools/loadtest.sh` and the `Load test` workflow start it fresh, run, read its `ops`, and drop it. `WORLD=live` is for a last rehearsal only. The harness times `/health` from a separate process, since the generator's own round trips above two hundred names are mostly its own lag. (2) **The load events already in the live log are not cut out** (they are 600,000 of 1.38 million; events depend on one another, and a log cut in the middle may not replay). They go with everything else when the log is archived: **a checkpoint of the world's state, the events before it moved to a second table (David: a second table, kept queryable), boot from the checkpoint and what follows it.** Unwritten; it touches boot, which took the world down on 11-15, so it wants a design, a copy of the live log to rehearse on, and a replay that is compared with the state it replaces. (3) Load names (`Gnload…`) are left out of the board's count of names forgotten (David: yes).
- **S16 · The log's archive: a checkpoint, a second table, a boot that does not replay everything** (M/L; alpha gate 3; David, 11-15: next, the design here, a second table kept queryable). **Not started.**
  - **Why.** Boot replays every event: 1.39 million on 11-15, about a minute of nobody answered at every deploy, and it grows with every event for ever (it took the world down once, T20, when it grew past memory). 600,000 of those events are one evening's load tests, which cannot be cut out of the middle of a log whose events lean on each other.
  - **What a checkpoint is.** One row in a new table `checkpoints (world, seq, at, driver, body)`: everything boot builds from the log, as it stood after event `seq`. That is four things, and the design stands or falls on the list being complete: (1) the store (it already serializes; its kind, parent and owner indexes are rebuilt on load, as they are skipped in the serialized form); (2) the pending jobs; (3) the three folds kept from the whole log: the board's, the citizens' (who has a name and a passphrase) and the waiting words' (`tells`); (4) the small things boot learns on the way: the log's head, what `seen` warmed. `body` is that, compressed.
  - **Boot.** If a checkpoint exists for the world and this driver can read it: load it, then replay only the events after its `seq`, a page at a time as now. If there is none, or it does not load (another driver's shape, a schema that has since gained a field the body lacks, a body that does not parse): say so in the journal and replay the whole log as today. **The checkpoint is never the truth; the log is.** A driver must always be able to boot without one, and a deploy that changes the store's shape simply ignores the old checkpoint and writes a new one.
  - **Writing one.** After a boot has finished and the world is answering, and then once a day at the quiet hour: under the world's lock only long enough to copy the state (the store clones; measured before it ships), serialized and written outside the lock. One kept per world, and the one before it, in case the newest will not load.
  - **The second table.** `events_archive`, the same columns and indexes as `events`. Archiving moves every event at or before the checkpoint's `seq`, less a tail, from `events` to `events_archive` in one transaction (insert, delete, the checkpoint's row marked `archived_through`). The tail left in `events` is the last 60,000 events: the furthest back any verb reads today (`recent_events(60_000)`; the rest read 8,000 or fewer), so `messages`, `session`, `log`, `news` and the folds' tails answer as they do now without knowing the archive exists. The archive is read by nothing in the game; it is there for a query by hand, for `trace`, and for a full replay.
  - **A full replay stays possible.** `foxing-driver replay --from-archive` (and the tests) reads `events_archive` then `events` and must arrive at the same store as the checkpoint plus tail. That comparison is the proof, and it is run before the first archive and by a workflow after every one.
  - **What changes for the nightly backup.** It copies the log; it must copy both tables and the checkpoints, and a restore must be tried once (beta's gate, but the backup must not silently start missing two thirds of the history).
  - **The order of work, each step shippable alone.** (a) Checkpoint written and read, archive untouched: boot falls from a minute to seconds, and the whole log is still in `events` if anything is wrong. (b) The comparison tool and workflow: a full replay against the newest checkpoint, on the load world's database with a copy of the live log (`pg_dump` of one world into `foxing_load`; David or a workflow, as root). (c) Only then the move to `events_archive`, first on that copy, then live at a quiet hour with the backup taken first.
  - **What can go wrong, and what catches it.** A fold or a piece of state missing from the checkpoint: the comparison (b) differs, before anything is moved. A checkpoint a new build cannot read: boot falls back to the whole log (so (c) must keep that possible: with the archive in place the fallback reads both tables). A verb that reads further back than the tail: a test that walks every `recent_events` caller against a world whose older events are archived. The move interrupted: it is one transaction.
  - **(a) shipped 11-15.** `checkpoint.rs`: a file beside the data, `checkpoint-<world>.json` and the one before it (a file and not the table first written here: the same for every kind of log, and already inside what the nightly backup copies). It holds the store, the pending jobs, the three folds, the schemas boot started from, and the place and time of its last event. Boot uses it when its format and schemas are this build's and the log has that event written at that time; otherwise it says why in the journal and replays everything. It is not tied to the build (every deploy is a boot): `checkpoint::FORMAT` is raised by hand when the same events would fold differently, and (b) is what catches a change that forgot. Written a minute after boot and daily. A test holds a boot from a checkpoint to a boot from the whole log: the same store with its indexes, jobs and folds; and a local world on PostgreSQL restarted both ways gave the same board and citizens' report (4,027 events replayed in place of 8,940). The first live boot after this deploy replays everything once more; the one after is the short one. `FOXING_CHECKPOINT=0` turns it off.
  - **Decisions inside it, mine unless David says otherwise.** The tail is 60,000 events; checkpoints are daily and at boot; two are kept; load-test events are archived with the rest and not deleted.
- **ALPHA · the gates out of pre-alpha (David, 11-15).** (1) **Five hundred names at once at any usable level, even highly degraded: met** (11-15, on the load world: every command answered, about a second at the middle, 3.7 s at the 95th; David accepts it without a live rehearsal, and capacity work stops there: S15 (a) and (c) are for after alpha). (2) The reset warning stays until beta; beta brings deploy windows, and with them "no resets" and a tried restore. (3) **The log's archive is next** (S16, designed 11-15: a checkpoint, the events before it in a second table, boot from the checkpoint). (4) A deploy that fails says in Discord whether the world is up (shipped 11-15). (5) **The first hour is fixed before it is called alpha** (David): `help` opens with what to do and `wants` explains its pay notes (shipped 11-15); what a newcomer still trips on is R248's. (6) Nothing else is a gate: no invites, cap, moderation or status page. Open decisions: none but D33, which waits.
- **R248** (S) What is left from the Watch of 11-14 (sixth): a bare `attack` after a kill silently takes another unnamed creature in the room; say which. A lampmoth "hits 3–5" hit exactly 5 five times running: check the roll. A refused extra story to a creature on its first word prints the sixteen words its word is among (David, 11-15: leave it, a reward for persistence). No strike on a hidden word has been seen live; the wait between weighed guesses is three minutes now (was ten). The cut-reply fall guard was not reached live and has no test of its own.
- **R247** (S) What is left from the Watch of 11-14 (fifth): `travel Annex of Second Readings` and `take stone roll` run right with no "read as one name" line (two bare words fit `travel`'s exit and word, so the gate has nothing to mend). A memory won from a wisp that held Fallis's story is named "Fallis's story of someone's name" though it counts as a fight with a wisp. `where lampmoth` gives a route without saying where it starts. Still unseen live: a strike on a hidden word and its pay (a tester sent for nothing else, 11-14); the doubled line did not recur in three rounds with two testers in one room. (Not a fault, struck: the hints differ by which word a creature is on, and the ten-minute rule bites only on a second story.)
- **R246** (S) What is left of it: a message of 21 statements passes with "scored 100 Legibility" beside "ran none of it". The dry run says a story sent inside ten minutes "counts as a try at the word" where the world says it was not weighed (with R212). `mine` opens "What remembers you:" over an empty list. Nothing said aloud by a neighbor was there to test the doubled line (R242) on the live world: still unseen.
- **R241** (S) `messages` drops "from The Orchard Wall" from a word left from another room (the record does not keep the room). `help` files `almanac` under "other" beside `time` under "read".
- R209 was met again by this Watch (`mine` after a fall says "nothing touched any of it").
From the live Watch of 11-13 (a fighter, a teller, a maker on a kept name, a postmaster seventy minutes late; see T14):
- **R210** (S) (`retreat`'s sums are labeled, 11-14.) The last-blow warning names a jar of fog and not where one comes from (`recipes`); "The second fall of the night: it costs half what the first did" beside a second fall that took 3 where the first took 2 (the window had grown): say "half the share".
- **R211** (S) A felled wisp struck again by its id is answered by kind ("wisp is not itself yet and cannot be struck for 59s; it stands up with nothing to take"), where an inkling's id "is gone": say the id typed, and say once why one stands up and the other does not. The bookworm's round prints "(6 left)" and then "it mends by what it took (8 of 9)": the count after the mend, in order. A creature's hit points are fractional in `look` (5.2/5.2) and whole in the round.
- **R212** (S) What is left of the dry run against the gate: `name` and `remember` do not see what an earlier statement of the same message did ("holds 0" after a `remember` two lines up); a message of nineteen or twenty statements failed "CL-I6"/"CL-I5" with no reason given.
- **R213** (S) Things made: a new lantern's reply says "it holds nothing yet" and `read` says "with a faint memory of…"; the lamp's recipe says "a chunk (any…)" and refuses chunks of 12.7 and 5.7 with "You lack the memory to spend"; an entry in the Exergue's book is stamped 09:51 at 09:09 by the world's clock (whose clock stamped it?).
- **R215** (S) (Asked what it wants, a creature says the word it wants carried too, 11-14.) "what is your word?" is the same paragraph from every creature and does not say which rung that one stands on. The quickstart's example names (Marrow, Lute) are nobody in the world, and `where Wyndle` is refused though the Register of Names lists a Wyndle (a name taken back: say so). The Namery lists "out" and "back", both to the Archive. `rest` says "3 strikes in 10 land" where the quickstart says one in four.
- **R216** (S) The Dead Letter Office: `reply 1 ""` is refused by the checker as a missing argument at a quarter where the world's own line ("Answer it with what?") is better; the Docent's news does not say when a letter arrives. (The signpost has an arm for it when a letter waits, 11-14.)
From the live Watch of 11-12 (a guide reader, a teller, a mender, a latecomer):
- **R204** (S) A fall ends the message and the statements after it are dropped without a word (three attacks sent, one run): the reply should say what was not run and why.
- **R207** (S) `read doors` gives an age only to rooms dug since 11-12 (say "before the wall kept dates" for the rest); a named walk to the Lookout goes through the Bog with no word (it is the only way but the Hush: say so as it steps in).
- **R208** (S) What a newcomer still is not told: `recipes` on its first line says what you lack and not where it is (the where is at the foot); the charm's recipe says "worth 5 or more" and takes the whole chunk offered (20.6); the Exergue's plaque is "a visitor book" in `look`; `chunks` shows "(passing)" and says nothing of what it does; a gardener's row says its bite will not close and one of two never bled in six bites (chance or fault: check). `tell` to a name elsewhere should say when they last acted.
From the live Watch of 11-11 (a walker, a maker, a teller, a newcomer):
- **R193** (S) A named walk's reply lists the doors taken and not the rooms passed.
- **R194** (S) The Mudroom's digest of doors changed between two visits with nothing dug (a room at 53% left the line while rooms at 6% and 5% joined it): "best remembered" should be by what is left now, and the line should say its rule ("yours, the newest three, the best kept three").
- **R195** (S) The Lookout says "nobody on it but you" beside "longsight is looking at spacepants in The Picnic That Waited". The Threshold's signpost gave Fallis's want three minutes after it had turned.
- **R197** (S) `read` a lantern gives "three times as long" where `map` gives "half every 12.0 hours"; `recipes` does not say a first chunk comes from a first fight; a poultice carried on a `rest` that needs none says nothing.
- **R199** (M) The word ladder, what is left (a reply no longer says both "it was the story it wanted" and "not the one it wanted", 11-14): Wyndle's second word was not struck by any word of the sentence its hint quotes (the sentence is also its old want, so a story carrying it is heard as the want and likely never tried against the word); `check` refused a story the gate then heard at plain pay; a creature full for the hour answers eight guesses with "takes no more" and no count, though `wants` says a word-carrying story is heard.
- **R200** (S) The Docent: two answers were of where the asker had been, not where it stood; its first tip names a wisp that is "not itself yet".
- **R201** (S) The dry run says `tell` "needs an argument (text)" where the gate says two. ("holding" has a figure beside it now: what felling one that holds nothing is worth.)
From the live Watch of 11-10:
- **R189** (S) `session` counts "1 line written" after a `describe` and an `append`, and "1 said" where nothing was said aloud (the story).
- **R190** (S) Short, a room of your own says "Yours: 100% remembered" on arrival the same after four seconds or four hours: say what the absence did ("it had faded to 97%; you bring it back whole"). The builder's one ask.
- **R191** (S) `observe` with no argument fails the whole message at a quarter ("'observe' needs 1 argument (query)"); a `describe` aimed at a room that is not yours should be refused in a line that says whose it is.

From the Night's Watch of 11-10 (on a local copy):
- **R155** (S) `look` lists a creature "holding 0" in the reply after the fall text says it "keeps" what you lost (the revenant, 58.7/64); its holding should show what it took.
- **R157** (S) `mine` after two falls: "nothing touched any of it, and nothing of it has faded enough to say" under an empty "What remembers you:"; with nothing kept, say that, and the falls.
- **R160** (S) A creature's holding after a story rose by less than the chunk the teller got (27.0 XP to the teller, 61 → 74 in the creature) and by more after a plain one (6.0, 74 → 90): the teller's pay and the creature's keep are two figures; say the creature's in the reply ("it keeps 13 of it").
- **R163** (S) A short name finds the wrong thing: `read charm` read Charmkin (found) over the moth charm just made; prefer what the reader made, then what it carries, and say which it chose.
- **R165** (S) A fixture's `read` gives only its line ("a lamp (item-166), whole (1 line)…") where a carried thing recites its life; and a spent charm cannot be read at all ("There is no item-197 here to speak of"): a fixture's past is read like any made thing's, and a spent thing leaves its last line where it was spent.
- **R166** (S) Charmkin ("your next three fights' memories more vivid… one each") shows no count and no line when it acts: say "Charmkin warms: two left" on the kill it touched.
- **R167** (S) The room text after a memory is fed to a dug room says "Kept in memory by Gnarrowby (with wisp, telling Loam a story): the memory of a creature of theirs, fed to it": the clause is the creature's, said of a room.
- **R168** (S) "The fog gives up shapes: 2 wisps" with one wisp in `look` (a named one beside it was counted, or one had gone): count what arrives.
- **R169** (S) A wisp drifts in at "0.3/8.3": a wounded one that left comes back unmended and reads as a new spawn; say "comes back, still hurt", or mend it away.
- **R170** (S) The map of a floor reached by two stairs drew two rooms five cells apart on one row ("05· . . . . . . 45@") with nothing between; the list now has an "Other floors" line and a named door's room sits on the far side of its parent, but the grid of such a floor has not been checked by a Watch since. The list still says "Lantern Row" beside "The Assay Office" (the article kept on some names only).
- **R171** (S) The Lookout says "the nine rooms from the Stile up" and the walker counted eight; the Long Afternoon's "(8 off)" is unexplained.
- **R173** (S) A thing that lay in a room and is gone on the next short arrival leaves no line ("as you left it" over a room missing its charm): short, a room says what has changed since ("Charmkin is gone").
- **R175** (S) `messages` counts the Docent's welcome again on every read ("the last N of N"); `read` with no thing means the room's page in the Conclave's rooms and "Nothing here has writing on it" in the Bar: one meaning, said in `help read`.
- **R176** (S) The budget's arithmetic is not sayable from the replies (a message of six with one refused cost 2.0; a free read after arrival left 58.75): the reply says what this message cost and why.

From the Night's Watch of 11-06:
- **R101** (S; looked at 11-07, open: `rest`'s "Full in" is read off the body's curve on active time, and a hypothesis is that the clause ran before the arrival's recurve; not reproduced, kept for the next Watch to measure) `rest` in a room just fought in moved 6 s in 31 s (nothing was fighting; the mothwing was dead); check whether `end_fight` recurved the mending.
Pulled from the backlog 11-05 ("B19 yes, B21 yes, B22 yes"), each in pieces the Watch can take a batch at a time:
- **R74** (M; B19) Crafting, second tier: an usher's ribbon (a thing that lets you choose the room an usher shows you to) and a censor's lens (halves the ray) as recipes from Stacks drops; desires beyond rooms (a hand not met, a story told near it); a thing's past as a want; a sprite per tier on the page; `gather` reading the first cue ("a river of ink" gives ink).
- **R75** (M; B21) The Exergue, next: a bar story's weight by its money words; the Assayer answering `tell` with your holding's assay; the Wardens' respawn lengthening each taking; the Purse's share as a statute; the sweater's bag and the clippers' filings as things; the Mintmark Hall stamping kept things with their hour; atlas frames for the eight; the slate in the tapster's voice.
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
- **S6 · A load test that is a Watch** (M; begun 11-13 on the live world, by David's
  word). `tools/load.py <names> <minutes>`: passing names at a player's pace, six in
  ten on a session with the play page's three subscriptions, two in ten through the
  gate, two in ten at the plain command door; fighters, walkers and tellers; a steady
  probe of `/v0/board` and `/health` beside them. The first batch, twenty names for
  seven minutes at 315,000 events: 1,444 commands (3.4 a second), the world's share of
  a reply 5 ms at the middle and 15 at the 95th (a gate message of three, 24), nobody
  waiting on the lock past 55 ms, a keyboard round trip 104 ms at the middle and 205 at
  the 95th, the probe flat, no reply other than 200. What it found: the pushes. Twelve
  sessions were sent 63,854 frames, 57 MB, eleven kilobytes a second each, for 8,002
  events written: every session is sent nearly every event (S5 is next, on this
  evidence). Two of twelve sessions were closed from the far side mid-run with no
  close frame (at four and six minutes; nginx's or the driver's doing is not known).
  And the game's own limit came first, as S8 said: 182 of 269 strikes from eight
  fighters found nothing to fight.
  The second batch of twenty, with the box read beside it (the server check's
  `vitals`, every five seconds): four cores and 7.7 GB, 98% idle through the run; the
  driver 4% of one core at the middle and 8% at its worst, PostgreSQL under 1%, nginx
  1%; memory flat at 0.8 GB; no wait on the disk; 16,900 pushes held the world 6.6 s
  of 366. The log grew 6 MB for 7,500 events (about 800 bytes an event: a gigabyte for
  every 1.3 million). Seen in passing, for S7: `shared_buffers` is PostgreSQL's
  default 128 MB under a 362 MB table; the driver peaks at 2.3 GB while it replays the
  log at boot and takes 21 s to do it; and every push to `crates/**` restarts it.
  Fifty names, six minutes: 3,134 commands (8.4 a second), 66 events a second, every
  reply a 200. The world's share of a reply 5 ms at the middle and 27 to 39 at the
  95th; the wait for the lock 6 ms at the 95th and 55 at worst. The box 93% idle; the
  driver 19% of one core at the middle and 31% at its worst (two and a half times the
  names, four and a half times the CPU); PostgreSQL 1.3%, nginx 3%; memory flat. The
  pushes are the curve: thirty sessions were sent 345,710 frames, 335 MB (nearly six
  times the twenty's), and 92,000 pushes held the world's lock 38 s of 370, a tenth
  of the run. On that curve a hundred names is most of a core and the lock held near
  half the time by pushes alone: **S5 comes before a hundred.** (The test's own
  machine, two cores reading 900 frames a second, is by now part of the round trip it
  reports; the driver's figures are the ones to trust.)
  Fifty again after S5 (a change read once and pushed only to the sessions it
  concerns): the same 8.7 commands a second and 76 events a second; the driver 7% of
  one core at the middle and 9% at its worst (it was 19 and 31); 53,770 frames and
  29 MB to thirty sessions (346,000 and 335 MB); the pushes held the lock 5 s of 370
  (38), and 81,765 of 102,000 wakings took no lock at all. The world's share of a
  reply is unchanged (5 ms, 30 at the 95th: `tell` and `say` in a crowded room are the
  slow verbs now, 33 and 18 ms at the middle, writing a line for every listener).
  What the fifty found that is not speed: a room past a hundred things had no room
  panel (mended the same day: a room says its fifty best remembered names and counts
  the rest), and one session in thirty still drops mid-run.
  Next: a hundred, then two hundred, each with `vitals` beside it.
  **A hundred names, six minutes (11-14, after S5 and S9, at 415,000 events):** it
  held. 6,353 commands (17 a second), 180 events a second, every reply a 200, no
  session dropped. The world's share of a reply 5 to 9 ms at the middle and 36 to 76
  at the 95th; the wait for the lock 16 to 20 ms at the 95th, 174 at worst; a
  keyboard round trip 123 ms at the middle and 399 at the 95th, the gate's 335 and
  611. The box 85% idle (four cores); the driver 44% of one core at the middle and
  85% at its worst (twice the names of fifty, six times the CPU); PostgreSQL 3%,
  nginx 3%; memory flat at 0.85 GB; no wait on the disk. Where it went:
  `say` and `tell`. A `say` wrote 41 events and took 28 ms, a `tell` 76 events and
  51 ms, nine tenths of it the log: 558 of them, a tenth of the commands, wrote half
  of the run's 68,900 events. And the pushes again: 92,600 held the world's lock
  102 s of 370, more than a quarter of the run (294,000 were passed over, S5's
  doing); sixty sessions were sent 273,000 frames and 509 MB, 23 kB a second each,
  389 MB of it `obs:room`, the whole room sent again on every change in it. Every
  read (`chunks`, `map`, `time`) writes two events. The log grew 38 MB (560 bytes an
  event; the disk is 150 GB, 3% used). Discord answered 429 to most of a hundred
  "arrived for the first time" notices. 1,020 commands were refused, not broken down
  (the test counts them and does not say why). **On this curve one core is full
  near two hundred names**, and the lock, not the box, is the limit: S10 comes
  before two hundred.
  **A hundred again after S10 a and b (11-14, twice):** the same 16 to 17 commands a
  second, and 62 events a second where there were 180 (23,200 for the run; 68,900).
  A `say` 4.8 ms and four events (28 ms, 41), a `tell` 6.2 ms and five (51 ms, 76).
  The world's share of a reply 5 to 8 ms at the middle and 12 to 29 at the 95th (36
  to 76); the wait for the lock 6 to 7 ms at the 95th (16 to 20). The driver 27 to
  30% of one core at the middle and 57 to 59% at its worst (44 and 85); the box 90%
  idle. The pushes held the lock 45 to 55 s of 370 (102); sixty sessions were sent
  306 to 349 MB (509), `obs:room` still 187 to 220 MB of it. The log grew 21 MB
  (38). **One core is now full nearer three hundred names than two.** What is left
  of the curve is `obs:room`: twice a second is still the whole room each time; the
  next step is to send what changed (S11, unwritten until it is wanted).
  **Two hundred (11-14, 21:18 to 21:24, after the fifth Watch's deploy):** 200 names,
  120 of them on sessions, 6.2 minutes, 12,079 commands (32 a second), 47,470
  events (127 a second). Nothing failed: `/health` and `board` answered every probe,
  eight logins were turned away by the login limit (429) and nothing else was. The
  world's share of a reply 4 to 8 ms at the middle and 12 to 33 at the 95th, worst
  79; the wait for the lock 11 ms at the 95th, worst 128. The driver 78% of one
  core on average and over 100% at its worst (it has more than one thread; the box
  never under 52% idle), PostgreSQL 3%, appends 0.7 ms. **The pushes held the lock
  165 s of 370** (45%), and 120 sessions were sent 882 MB: `obs:room` 423 MB,
  `store.component_set` 189, narrative 118, `say` 60. So two hundred hold with
  replies still quick, and the lock is the limit as foretold: the pushes would fill
  it somewhere past three hundred. S11 (send what changed in a room, not the room)
  is now the next thing a larger crowd needs. The arrivals went to Discord as a
  crowd for the first time, and read well (David, 11-14).
  **S11, shipped 11-14 (David: build it now, test on the live world):** a session is
  sent an observation only when it differs from the last one sent; the play page asks
  for patches (`"patch": true` on subscribe) and is sent what changed, by node; and
  somebody else's change waits 0.5 s in a room of up to forty things, 1 s to a
  hundred, 2 s beyond (one's own is still sent at once). Three hundred names next.
  **Three hundred (11-14, 16:45 to 16:51, on S11):** 300 names, 180 on sessions, 6.3
  minutes, 14,195 commands (38 a second), 59,767 events (158 a second). Nothing
  failed (27 logins turned away by the login limit, as before; every probe
  answered), but it is past the knee: the world's own share of a reply is as it was
  (5 to 8 ms at the middle, 15 to 34 at the 95th; lock wait 14 ms at the 95th), and
  a reply's round trip from here went from 0.13 s at the middle (two hundred) to
  0.68 s, 2.6 s at the 95th, `/health` 0.68 s. The driver 111% of a core on average,
  121 at its worst; the box 55% idle; PostgreSQL under 10%. S11 did what it was for:
  `obs:room` 199 MB to 180 sessions (423 MB to 120), with the load test's sessions
  not asking for patches. **What holds the lock now is the events, not the rooms:**
  the pushes took it 347,000 times and held it 217 s of 378 (57%), each session
  reading and redacting the same events for itself on every change; the events are
  also most of the 881 MB sent (`component_set` 287, narrative 223, `say` 91).
  **S13 (unwritten):** read and redact a change's events once for all sessions
  (one pass under the lock, a per-viewer filter outside it), and stop sending a
  session the `component_set` events its patched subscription already carries.
  Until then the honest figure is two hundred names with quick replies, three
  hundred with slow ones.
  **S13 a, shipped 11-14 (the goal David set: five hundred names, and alpha):** each
  event is judged once when its change is announced (can its entity be observed,
  whose is it, who may see each field), and a session filters and cuts the events
  for its own viewer with no lock; it takes the world's lock only to run an
  observation that is due. Still to do under S13: not sending a session the
  `component_set` events its patched room already carries.
  **Three hundred again (11-14, 18:53 to 18:59, on S13 a):** 14,737 commands, nothing
  failed (41 logins at the login limit). The sessions took the lock 71,000 times
  where they had taken it 347,000, and a reply's round trip from here fell from
  0.68 s to 0.43 s at the middle (2.6 to 1.9 s at the 95th). **But the lock was held
  as long as before, 207 s of 378:** each hold is now an observation and nothing
  else, 2.9 ms apiece, and the wait for the lock rose to 23 ms at the 95th. So what
  costs is running a crowded room's observation once for every session in it, each
  time it changes. The driver 130% of a core, the box 47% idle, PostgreSQL to 37%.
  **S14 (unwritten, the next step to five hundred):** one observation of a room per
  change, shared by the sessions in it (the public form once, each viewer's own
  fields laid over it), or the observations run beside the world rather than under
  its lock. And a load test from a box that is not this workspace, to tell the
  world's slowness from the relay's.
  **S14, shipped 11-14 (David: yes):** a room is run once for a change and shared.
  The query engine runs it as a stranger (signed in, owning nothing) and notes
  whom that cannot serve: the owner of anything in view with a field a stranger is
  not shown, and everyone when a relation ran past its breadth. A session it
  serves is answered from the one run; the rest are run on their own as before.
  The play page's room query reads only public fields, so a room serves everyone
  in it. A test holds the shared run to each viewer's own for three queries.
  **Three hundred on S14, from a runner (11-14, 19:53; the harness's first run):**
  16,959 commands, no errors at all (the tunnel goes around the login limit), a
  reply's round trip 0.27 s at the middle and 2.2 s at the 95th. The first cut of
  S14 shared almost nothing (a kept room was trusted only to the log's next event
  anywhere); the second keeps a room until something in it changes. With the
  observations timed by kind, the 190 s the lock was held went: rooms run for one
  viewer alone 91 s (19,457 at 4.7 ms: every room past its breadth of a hundred,
  where nobody shared), rooms shared 38 s (32,807), `me` 18 s (42,624). So the
  packed rooms are what is left, and a room past its breadth is now shared by
  every name the shared run kept (the order of what is kept is the order found,
  no longer the asker first). An index of what each actor owns replaced a walk of
  every citizen on each command and observation.
  **And again with packed rooms shared (11-14, 20:03, from a runner):** no better.
  15,901 commands, no errors, a round trip 0.36 s at the middle and 2.9 s at the
  95th; the lock held 194 s of 378; rooms run for one viewer alone still 16,374 at
  4.85 ms (79 s), shared 29,943 (35 s), `me` 37,959 (15 s). One wait for the lock
  of 8.3 s, cause not found (not a push: their worst hold was 57 ms). A packed
  room's observation measured on the workspace's machine, release build, 160
  names: 0.42 ms, of which ranking the names is 0.07. So the same work costs ten
  times as much on the box, under load, as here; and in a room of a hundred and
  fifty names a hundred are still answered alone. **What is left is structural,
  and David's to choose (S15):** (a) the world behind a read-write lock, so
  observations run side by side on the box's four cores and only commands take it
  whole; (b) a box with dedicated cores; (c) serve a name trimmed out of a packed
  room from the shared run with its own node put in. Three hundred names hold with
  no errors and a median reply near a third of a second; five hundred are not there.
  **Five hundred (11-14, 22:06, from a runner; David: "let's yolo it"):** the world
  stayed up and nothing was refused or lost (one session dropped on a ping), and it
  was not playable: 13,480 commands (35 a second, fewer than three hundred made),
  a reply's round trip 3.6 s at the middle and 11 s at the 95th, `/health` 4.8 s.
  **The wait is not in the world and not at the lock:** a command's own time was
  as ever (5 ms at the middle), the wait for the lock 42 ms at the 95th, the driver
  105% of a core and the box 57% idle. The pushes held the lock 208 s of 390. So a
  request waits seconds before any handler starts: the sessions take the world's
  lock on the async runtime's own worker threads, and with three hundred of them
  queued on it the four workers are all stood waiting and nothing else is picked
  up. **S15 is therefore first: (0) keep the lock off the runtime's workers (more
  workers at once; then the sessions' observations on blocking threads), before
  (a) the read-write lock or (b) a larger box.**
  **Five hundred, twice more, and what it really was (11-14, 22:27 and 22:35):**
  forty-eight runtime workers changed nothing in the generator's figures (4 s at
  the middle). Then the world timed from the workspace while a runner drove five
  hundred names: `/health` 0.29 to 0.32 s and `board` about 0.5 s, **the same as with
  nobody there**, for the last two minutes at full crowd; and one minute of stalls
  of 2 to 10 s before it. So (1) the generator's own round trips above two hundred
  names are mostly the generator (one Python loop reading a gigabyte of frames),
  and (2) the stall was the departure sweep: a quarter of an hour after a restart
  everyone the process has not heard from since goes, all in one tick under the
  lock, and each run tonight came a quarter of an hour after a deploy (the 8 s and
  11 s waits for the lock were this). A tick now sees off or forgets twelve names
  at most. To prove: five hundred with the sweep landing mid-run, timed from
  outside the generator throughout.
  **Five hundred, timed from outside the whole run (11-15, 07:05 and 07:34 UTC):**
  the earlier reading was wrong. `/health` took 2 to 14 s for as long as five
  hundred names were in the world and 0.3 s the moment they left; the departure
  sweep was a real stall and not the cause. Two things found in the path and
  changed: every command announced its change by taking `announced` and then
  waiting for the world's lock while holding it, so every reply queued behind that
  wait (now a change is flagged and carried by whoever is announcing); and a
  session that fell behind the announcements was answered from the log under the
  world's lock on every wake (65,224 of 67,584 pushes at five hundred; a slow
  reader cost everybody the lock): it now drops what it missed, is told, and has
  its subscriptions sent again. The second of these is live as of 08:12 and not
  yet measured.
  **Five hundred on the load world (11-15, 09:15 UTC; the first run that wrote
  nothing to the live log):** 24,667 commands in 6.3 minutes (65 a second, where the
  runs before made 35), every one answered 200, no errors. `/health` timed by a
  separate process every 2 s: 0.98 s at the middle, 3.7 s at the 95th, 12 s at the
  worst, 56 of 115 over a second (before the two changes: 2 to 14 s throughout).
  The generator's own round trips 1.0 s at the middle and 3.4 s at the 95th. The
  wait for the lock is now where the time shows, 405 ms at the 95th: the queue is
  at the lock and measured, no longer hidden behind `announced`. The live world,
  timed from the workspace throughout: 0.31 s at the middle, 0.53 s at the worst,
  as with nobody next door. The box 29% idle at its lowest, 1.3 GB used. **So five
  hundred names are answered, every one, about a second apiece: degraded and
  usable, on an empty load world.** Still to say it of the live world: one
  rehearsal with `WORLD=live` (its log is larger and its rooms more), David's to
  call. What would make it quick and not only usable is S15 (a) or (c). The
  harness did not read the load world's `ops` (the token line; to mend).
  **A second place to test from:** `tools/loadtest.sh <names> <minutes>` on any
  machine with SSH to the box (an SSH tunnel to the driver's own port, around
  nginx's per-address limits), and the same as a hand-run workflow, `Load test`.
  **The dropped sessions were the test's, not the world's.** Eight and nine of sixty
  sessions were reset mid-run in these two runs (none in the run before, two of
  twelve and one of thirty in earlier ones). The driver now says in its journal why
  a session ended, and the vitals check reads it: each was "reading from the client
  failed: Connection reset without closing handshake", and the test machine's own
  outbound relay logged the same seventeen closes as its own ("tunnel closed, code
  1012, server_draining"). nginx and the driver were bystanders. A load test from
  this workspace cannot hold sixty long connections without losing some; the
  figures above are the driver's and are unaffected.
- **S10 · What a hundred found, what is left** (a and b are live, 11-14).
  - *c · A read writes two events*: answered 11-14 (a): kept. The gate's record of every message and reply, reads included, is the promise.
  - *d · A lab arrival in Discord:* left as it is, by David's word (11-13): he
    wants to see arrivals for now.
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
- Order: S6 at two hundred; S4, S9. S2b when you choose.

## Content (C): expansions, planned 11-12 at David's ask; C3 is live, the rest is not built

What the world has now: ten districts, about seventy rooms of its own, some thirty
kinds of creature, two tiers of recipes, the errand and the word ladder, the Conclave,
Morphos twice a day. What the Watches keep finding is where it is thin, and that is
where this starts: the door is crowded, the middle is empty, and almost nothing asks
for two names at once. Sizes are for a first cut that a Watch can play. Say `C4
brief` for a page on one, or answer the lettered ones.

- **C11 · The Postern, what is left** (S; the second door is open, 11-13: a new name
  wakes at whichever of the Threshold and the Postern has fewer names in it, the
  Threshold at a tie; `FOXING_SECOND_DOOR=0` shuts it).
  - *Next:* a Watch of six on the live world (David's word), half of whom will wake
    at the gate.
  - *Still the Threshold's alone:* the Docent in person (the Postern has its card;
    R76b's Under-Docent as a Gatekeeper is the figure, if wanted), the chalk tally
    of who came through, the day's name, the cloak's view, and the pointers that
    send a newcomer "east to the Fogbank" from wherever it stands (`chunks` when
    empty, the Docent's by-heart tips: R151's cousin).
  - *Not done and not planned:* a name coming back from away is where it left the
    world, as now, not at its door.
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
- **C4 · Things for two** (M). The Wardens (320 hit points) are the only fight sized
  for company, and `retreat` already says "bring another name". Three small ones on
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
- **C9 · A district under the water** (L; later). The Undertow comes up in the
  Shallows and goes nowhere else. The **Tide Tables**: rooms that are there at some
  hours and under water at others, so the map itself is a thing you have to
  remember the time of. Large, and worth a brief before anything is cut.
- **C10 · For the agents** (S, alongside any of the above). Each district gets one
  thing that rewards reading over reflexes: a catalogue in the Stacks that is a real
  index of the public record, a division in the Conclave that can be predicted from
  its Book. Agents are half the players and the content so far is written for hands.
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
- **X6 · The Court of Lethe** (the judicial branch; the Conclave writes, the Clocktower
  moves, the Court decides between names). Brief wanted: who may bring a case (a
  wronged name, a room's owner, the Docent), what a judgment can do (restore a chunk,
  strike a name from the Roll, unmake a dug room, exile a creature), and whether the
  Court forgets its own rulings on the world's curve.
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
- **Y3** Issues: none open. The standing rule: a new issue gets an answer within a day
  and a close when it ships (Claude posts, signed, by the 11-03 permission); Glama's
  maintenance grade reads commit activity and whether open issues get a reply.
- **Y4** crates.io at the v0.4.0 tag (bump in lockstep, READMEs); Zenodo at the tag; the
  trademark filing checklist and a consult; the LLC; Tyler's crates, server and Glama
  maintainer access.
- **Y5** Show HN, with a log dump, when the game is further along.
- **Y6** Done on the boxes by David, 11-13 (his word; Claude cannot reach the apex
  host to see the headers): text compressed on the site box
  (`/etc/nginx/conf.d/gzip-text.conf`), `/play/mods/` served with
  `Cache-Control: public, max-age=300, must-revalidate`, and the analytics timers
  re-run. The box was ahead of the repo by that file and seven
  lines of `nginx-agentscanplay.conf`; both are in `site/deploy/` since 11-14.

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
