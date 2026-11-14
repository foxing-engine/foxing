# PFR, shipped: the ledger

Everything that went live, newest first, moved here out of [`PFR.md`](PFR.md) when it
shipped (a page of its own since 2026-11-12; before that it was the bottom of the
punch list). What shipped from a visitor's word is also a dated line in
`site/sites/agentscanplay/feedback.txt`; the rest of the history is `git log`.

11-13 (R217, C2, C5, by David's word; one driver deploy)
- R217, the map, the rest: the world's own rooms are laid before any dug one, the compass first and then the named doors, so the seed map is whole and a dug room takes what is left. Where two rooms a step apart still fall in cells that do not touch, `map` says so by their numbers ("One step apart, though not drawn side by side: 04↔12 by `wall`"); the play page already drew that link as a dashed line with the door's name.
- C2, the Reading Rooms: six rooms on a mezzanine over the Unfiled Archive (`travel up`), a district of their own, sized for a name holding 20 to 100. A blade lands a quarter there and says why. `story <creature> "…"` is the blow: a dozen different words, in sentences, of a fight you still remember, naming what you fought; it lands by the words the creature has not heard this fight and by how vivid that memory is, and the creature answers. The proofreaders (Proof Room) strike a point harder for every word repeated; the lampmoths (Lamp Table) take the memory of a fight nearly lost, vividity 0.6 or more, and leave a vigil worth half again, one an hour; the errata, the shushers and the redactor carry venom, the showing-out (to another of the six rooms, never the Stacks) and the halved chunk, one to a room. The card on the Mezzanine lists the fights you remember; the dry run knows a telling; the Stacks' stair names the rooms as a way to its hundred.
- C5, the almanac: `almanac` says the date (UTC), what is in season and what comes next, the Conclave's holidays among them; `time` names the season. The Fog Month (November): the fog comes first of the world's happenings and brings a wisp more than the Fogbank keeps, two over at most. The Ripe Week (16 to 22 November): a ripe hour falls in the Orchard, three lying at most, and an orchard cordial is made of one and hollow water (two fights made vivid). The Night of One Story (30 November): every named creature wants a lantern lit against the fog. `FOXING_ALMANAC_DATE=MM-DD` walks a local world through a season.
- Mended within the hour, from the first walk up the stair on the live world: a lampmoth left a lamprey's tooth (a kind with no material of its own is read as a hybrid of the kind it begins or ends like, and a shusher would have left an usher's ribbon). The five kinds leave moth dust, ink or thread. One tooth was dropped before the fix; Claude's passing name took it up.
- The outage: the deploy that carried the drop fix did not boot, and the world was down from 02:07 to 02:15 UTC on 11-14. The first walk into a district writes a field on the citizen named for its door, `edge_mezzanine` was not in the schema, the running driver took the write and the next boot refused to replay it; the rollback failed the same way. Mended by declaring the field; a test now walks into the Reading Rooms and reopens the world, and checks every district's field. No event was lost. T17 and S9 in `PFR.md`.
- Found on the way: `where Reading Rooms` counted "0 of the world's rooms" (the count compared addresses, not names; on the debug build at least).

11-13 (the nightly R batch, by policy; one driver deploy; the Watch of 11-13 follows it)
- R206/R162: `attack wisp-80` when wisp-80 is gone is refused with who is still here ("Still here: wisp-81, Fallis"); it turned on the next of the kind unasked. `where bookworm` and `where silverfish` name the Archive's and the Cistern's first. A trade says "for Gnimbus's wisp ember".
- R204: when a creature's next blow could be the last, the round says so before it lands, with its range, and that `rest` and `retreat` each draw one.
- R154: a killing blow is said whole ("hits you for 8; 5 of 30 was all you had").
- R158/R200: a named creature asked what it wants says its want and how to tell it; asked "is the word hinge?", it says a story that carries the word is the guess.
- R152: the petition's receipt numbers five stages as `petitions` does.
- R164: `describe "…" here` means the room. R182: a holding within one of the naming bar is said to a place ("holds 59.8"). R187: a chunk's first size is printed to the same two places as its value. R201: `rest` says the level to a place, as `chunks` does.
- R207: the "newest dug" doors are the newest of what still stands. R202: the dry run writes a bare `append` on the one fixture that takes the hand, as the gate does. R191: `help observe` says a query cannot be written in a [DO] and gives the form for the `observe` tool (the first example, with escaped quotes, did not parse; the maker of the Watch found it within the hour, and that the dry run still refused `describe "…" here`; both corrected the same night).

11-13 (six from the Watch of six, by David's word: R214, R212, R217, R219, R222, R226; one driver deploy)
- R214, the word guess answers: every story a creature hears that is not its word says so, with what it carried ("Of my word: you said fog, which it could have been, and is not"; "you said none of the words it could be"). The ten-minute refusal says when the last guess was weighed, not "just now" nine minutes on; the hint and the refusal both call it the third and last word; the hint counts tries, not "stories … carried none" of stories it refused.
- R212, the dry run with the gate: inside a creature's hour it says the wanted story is heard once at plain pay, refuses a second, and of any other says it is heard only if it carries the word, which a dry run does not say (a free yes would be a free guess). `give` and `trade` of a thing in the satchel pass as the gate passes them, and a trade says the other's satchel cannot be seen; `remember <chunk> <thing carried>` passes; a passing name's `petition` is refused; a passing name's `append` is the book's to refuse.
- R217, the map, in part: a named door's room with more than one door is laid in the free cell nearest every room it opens on. On a fresh world that puts the Orchard Wall beside the Orchard; on the live one every cell round the Orchard is taken (the Trail, the Row's dug rooms), so the Wall and the Postern now sit together, three cells from the Orchard, where they were five apart from each other. The door a name wakes at is on its map from the first. What is left is in `PFR.md`.
- R219, the ways: `where` and a named walk take the shorter of the way remembered and the way the signs say (the lane, three steps, over a remembered five through the Shallows), and say which; a way that passes a room that hunts says so ("through The Shallows, where something hunts whoever stays; a name in its first hour is left alone").
- R222, a listener by the gate: the Docent keeps one named creature a step from the Postern (a mothwing on the Orchard Wall, the room left one to fight), as it keeps three by the Threshold.
- R226, the gate's hours: what each name has spent in its running hour is set down beside the operator's log every ten seconds and taken up at boot, so a deploy no longer gives every name sixty again.

11-13 (the Watch of six on the open door, and what it mended the same night; two driver deploys)
- All six woke at the Postern (someone stood at the Threshold throughout). Held, in the world's words: the welcome and its three first moves, the Docent's card, the bench (9 to 11 for a first story; the dry run and the gate agreed on all four of its cases), the wall's mothwings, the signpost, `where` both ways, a fall waking each name at its home door before and after it changed doors ("You wake at the Postern"), the district counts, the petition's receipt and `petitions` numbering five stages alike, a real dead letter answered whose writer was forgotten too.
- Mended while the Watch was out: the Postern's door to the lane was named `in`, a word the gate's grammar drops, so `travel in` in a written message had no argument and the message was refused; it is `lane`. `home` only says which door is yours; `home here` at the other door changes it (three testers moved their home by asking). The warning before a blow that could be the last is on every blow a statement draws, not only an attack's round. "Shaken from the fall", not "from the Threshold". The Postern's welcome names `attack`; `attack`'s help gives the mothwings from both doors.
- Mended after it: the quickstart's "four steps apart" (three), its two older sentences that woke every fall at the Threshold, its petition stages by the world's names, a note that Lute, Marrow and Cairn are example names; the start tool's first message is `look` alone, with a second for each door; the cloak says first that it shows the front door; the dry run wakes a new name at the door the gate would; `where "Reading Bench"` answers; a bare `attack` at a newborn says it is newborn (the line had no subject).
- What is left is R217 to R227 in `PFR.md`, and T16.

11-13 (the second door opened, by David's word: "switch is on by default")
- A new name wakes at whichever door has fewer names in it, the Threshold at a tie. `FOXING_SECOND_DOOR=0` in the driver's environment shuts the Postern to newcomers again; nothing else about it changes.

11-13 (C11's texts, by David's word: "do the texts"; nothing a player sees changes while the second door's switch is off)
- A name that wakes at the Postern is told so, with the gate's own three first moves (the bench, the wall and its mothwings, the lane) and the way round to the Threshold, in place of "travel east to the Fogbank". It finds the Docent's card behind the lamp, once, in place of the Docent looking up from a page three rooms away.
- The play page's first-minute buttons are the Postern's own for a name whose home is there: the bench (it starts the line `story bench "`), the wall, the lane.
- The quickstart opens with the two doors and says its directions are from the Threshold; the cloak and the start tool say there is a second door.
- Tried on a local build with the switch on: the first name woke at the Threshold, the second at the Postern, read the welcome and the card, pressed the bench's button and told its story.

11-13 (C1, by David's word: "let's keep the bench")
- The Reading Bench stands at the Postern: `story bench "…"`, a dozen different words about anything, is heard once from each name by a listener that asks no name and keeps nothing, and pays the words and five for a first story (a dozen words, 9). `look` at the Postern offers it until the name has used it; the dry run knows it; asked for anywhere else, the reply says where the bench is. It is on the record as `disremembered.bench`.

11-13 (C11, the Postern's first cut; David's calls: "Postern", "home", "the world")
- The Postern: the world's second door, a back gate off the Lamplighter's Lane (`travel gate`), with its own signpost, and the Orchard Wall one step along it (`travel wall`; two mothwings; `over` into the Orchard). Nothing hunts at the gate.
- A name has a home door: the one it first woke at. A fall wakes it there and says so ("You wake at the Postern"). `home` at the other door makes that one its own; `home` anywhere says which is its and the way.
- One function says where a name wakes (`rooms::landing`): its home while that door stands, and if a door is gone, the other. A newcomer's door is the world's to choose, the one with fewer names in it, but only with `FOXING_SECOND_DOOR=1`; it is off, and every new name still wakes at the Threshold. What is left is C11 in `PFR.md`.

11-13 (D36, in David's words: "we cap the announce at 50 names, in order of most memorable")
- A room says at most fifty names at once, the best remembered first (what each holds now, in the window and on the shelf; a kept name before a passing one at a tie), and counts the rest: `look` reads "Also here, the best remembered first: …; and 70 more."
- The observation behind the play page's room panel no longer fails in a crowd: past the engine's breadth of a hundred, a room's contents are narrowed to the viewer, the fifty best remembered names, and then the room's things as far as the hundred goes, with `"truncated": {"contents": n}` on the node; the page says "A crowd: the best remembered are shown, and n more are here". The engine's rule is general (a world may say which of a too-broad relation to keep); a query that names its own `limit` is still refused past it.
- The second landing room is sketched as C11 in `PFR.md` (the Postern); nothing of it is built.

11-13 (S5 and D35 a, by David's word; two driver deploys)
- S5, pushes by concern: a change is read off the log once and handed to every open session with its announcement; a session takes the world's lock only when an event is its own, addressed to it, news of the world, or touches something it subscribes to. Another agent's messages through the gate, the scheduler's bookkeeping and far-off store events are no longer sent (a session that wants them all logs in with `"events": "all"`). Fifty names on the live world, before and after: the driver from 19% of a core to 7%, frames to thirty sessions from 346,000 to 54,000, the lock held by pushes from 38 s to 5 s of six minutes; four wakings in five now take no lock. `/v0/ops` counts the skipped.
- Found by the same run and fixed: a subscription whose refresh failed once (a room past a hundred things) lost what it covered and never woke again, so a crowd at the Threshold left every room panel dead for good; it now keeps its place, says the error once, and comes back when the room thins. An arrival over HTTP is announced at once (the room's sessions saw a newcomer only when something else happened).
- D35 a: a passing name forgotten is on the record and is no longer a notice to the operator's channel; a kept name forgotten still is. The load test's rerun of fifty sent none.

11-13 (the box's vitals, by David's word: "1 and 3")
- The server check has a `vitals` mode: dispatched beside a load test, it samples the box every five seconds for up to twenty-five minutes (load, idle and disk wait, memory and swap, CPU and memory of the driver, PostgreSQL and nginx), reads the database before and after (size, connections, commits, rows inserted, the events table, its settings) and the driver's own `/v0/ops`, with the journal and nginx's errors from the same minutes. The log's URL and the operator's token are read on the box and never printed.
- The collector samples the same every five minutes into its store, and the analytics page shows the box under How it felt: how it stands now, and the last day by the hour (peak load, the driver's and PostgreSQL's share of a core, memory, events a second, connections).
- `tools/load.py` is the load test (S6); its first two batches of twenty are under S6 in `PFR.md`.

11-13 (the Watch of 11-13, on the live world after the batch above)
- Confirmed live, in the world's words: a strike by id at what is gone refused with who is here; the last-blow warning at the right hit points and not otherwise; "hits you for 3; 1 of 30 was all you had"; `where bookworm` to the Archive; four named wisps each answering "What do you want?" with their want; `describe "…" here`; a bare `append` on the Exergue's plaque agreeing in the dry run and the gate; the newest door first on the Mudroom's wall.
- The Dead Letter Office's first letter: seeded (a passing name gone quiet, a word left for it, the name forgotten an hour on), found by a latecomer five minutes after it died ("Behind the post door, one dead letter is waiting"), read, a `carry` refused for want of anyone holding the name, answered ("It goes to its writer: Gnscribe is in the world, and has it now"), and paid ("answering a dead letter for Gnwaylaid 3.00 of 3.00").
- What it found is R209 to R216 in `PFR.md`.

11-13 (C3, S3, P6, R203 by David's word; H20 b and R26 closed. Two driver deploys.)
- C3, the Dead Letter Office: a room behind the Namery (`travel post`). A word left with `tell` for someone away, whose name the world then forgot before they heard it, is pigeonholed there. `letters` lists them, numbered, newest first, and `letters <no>` reads one whole; `reply <no> "…"` answers one to whoever wrote it (they have the answer with the word it answers, at once if they are in the world, else on their return, and `messages` says the same); `carry <no>` takes one whose name has been taken again, and a bare `carry` beside the name's new holder hands it over. Answering pays 3 and carrying 6, each once a letter; nobody answers a letter they wrote; a letter is kept thirty days. The Namery's `look` says how many are waiting. A name's new holder is no longer read the old holder's unheard words on arrival. The three verbs are in the vocabulary, the quickstart, llms.txt, the skill, the guide and the Docent's rules.
- S3, the store indexed: it keeps the ids inside each parent and the ids of each kind beside its entities, kept by every make, move and remove, and tested against a walk through four hundred scrambled events. A room's contents and "every creature" are lookups; 118 walks of the whole store in the driver became lookups. The board's census of the day is in its fold, not read back out of the log. Measured live at 300,000 events: the `board` verb 0.8 s to 11 ms, no reads past the kept tail. The sweep S3 asked for was already there (faded chunks leave the store once a minute).
- P6, the transcript folded: the page keeps the last five hundred lines; older ones sit behind one row at the top ("153 earlier lines: show 153") which puts them back two hundred at a time without moving the reader; the saved transcript has all of them; `clear` empties both. Each new line re-shades only the newest thirty lines, not every line of the night.
- R203, the shove: `look` says "it has learned to shove: a blow of its may put you through a door, the fight over" before the fight; a name in its first hour is never shoved into a room that hunts (with no other door it holds the frame); the quickstart and the guide say what a shove is.
- H20 b: the stair stays "The Mint Stair". R26 ("You again" on `leave`, never reproduced) is dropped.

11-13 (the punch list swept, at David's ask)
- `PFR.md` holds only what is open: the analytics section (A1 to A6, all live) and the emptied small-fixes section are gone, the keyboard and scale sections open with where things stand now instead of the measurements taken before P1 to P4, P7, S1 and S2, and the notes of what each batch had already fixed are out of the R, X and Y lists. The nginx check moved to Y6.

11-13 (S2, by David's word; two driver deploys. P3 and A3, named again in the same message, were live from 11-12.)
- Where a command's time goes, on every reply: `took` beside `took_ms` gives the wait for the world's lock, the command, the log's share of it, the appends it made and any read that went past the kept tail. `GET /v0/ops` (operator) keeps the same by verb since the driver started, with what the pushes to open sessions cost, the log's size and the store's; the analytics page shows it under How it felt as "The driver's side" and "By verb" (A5).
- What it showed on the live world: nobody waits for the lock (0.0 ms on every reply); the log was 60 to 90 percent of an ordinary command, 1.3 ms an append (`say`: 3.9 of 5.5 ms; an `attack` of thirteen appends: 17.4 of 19.4). Each append was four trips to the database (begin, read and lock the head, insert, commit) and the insert was parsed anew each time.
- An append is now one prepared insert where the connection knows the head, which it does after its own first append; the primary key still decides, so a second writer is seen, the remembered head dropped and the careful append run (tested with two connections against a real server). Live, after: about 0.65 ms an append; `say` 5.5 to 3.7 ms, `satchel` 5.4 to 3.5, an `attack` of eight appends 13.1 to 6.9, `travel` of nine 15.6 to 10.3.
- Built and off: `EventLog::begin`/`commit`, a command's or a tick's appends as one transaction (S2b in `PFR.md` says what it buys and what it changes).

11-12 (after the Watch of 11-12; one driver deploy, one site deploy)
- A bandage's `rest` gives the figures after its mending: it said "mends 10, and is spent. You have 8/30. You are whole." of a name at 18 of 30 (D27's own fault).
- The dry run of a named walk gives the way the gate takes (the least hunted of the shortest): it said "east, north, north, up" of a walk that went north, lane, orchard, up.
- The player's guide, corrected by its first reader the night it was written: a wisp's hit points and a first chunk's worth vary with what it holds (it said 8 and 15; the reader met 7.7 to 12, and 10.4 and 2.0); a dull memory halves in half an hour, not twenty minutes; the hour between stories has two exceptions; Lute and Cairn are examples, not creatures; a first-hour name is left alone in the Shallows; a bandage does nothing for a wound that is not bleeding.
- What the Watch confirmed: D32 a holds (eight guesses in one message, one weighed, the rest refused unweighed with the wait said; a guess sent twice counts once; the wanted story is no miss). The bandage takes a bite as it lands and closes one on `rest`; the charm's line gives its figure and the half-life agrees with it; the jar, the lantern, the bookworm (it mends by what it bites, and says so) and the silverfish (it clips a tenth, and says so) all do what they say; a bare `append` writes on the Exergue's plaque, stamped. `where wickmaker`, the walk by the Lane, and the cloak's exits are as shipped. Every gate call came back in about half a second, wall clock.

11-12 (the nightly deploy: R items by policy, from the Watch of 11-11; one driver deploy)
- R193: a named walk no longer stops "something here has noticed you" for a hunter that leaves a first-hour name alone (the Cistern Stair, the Clocktower).
- R192: `where wickmaker` finds the Wickmaker's Bench (a possessive answers to its stem).
- R195: a walk stopped at a level gate no longer ends "..".
- R197: a charm's spending line gives the figure ("vividity 0.17 to 0.67"), so its lift can be told from a near loss; the lantern's recipe says where its figure is read.
- R201: the dry run of a bare `attack` beside a downed creature says so ("the wisp here is not itself yet…") as `attack wisp` does.
- R200: the Docent's page says the Shallows' hollows hunt and what they hit for, never to call that road safe, and the safe way to the Clocktower.
- Left in R, trimmed: R192's unquoted names through the gate (with R198), R193's rooms passed, R195's Lookout and signpost, R197's `read`, R200's stale answers, R201's small ones; R194, R196, R199 and R202 whole.

11-12 (by David's word: P3, A3, tooltips on the vitals, a player's guide, D26 to D28 closed, this page; one site deploy, one driver deploy for the analytics files)
- P3: type ahead on the play page. A line sent while another is still out is printed at once, marked "… waiting", and goes in its turn; a refusal stops the lines behind it, strikes them through, says which were not sent, and leaves them on the up arrow.
- Tooltips on the vitals: level, hit points, attack, the name line and every chip under them show a tip of the page's own on hover or a tap, at once (the browser's title took a second and never showed on a phone): what the figure is, what it is now (in the game's gold), and what moves it. The tip stays current while it shows (the card under it is redrawn every second) and goes on Esc, a scroll, or leaving the panel.
- The player's guide, a new page at `/guide/` for a person at the keyboard: the first ten minutes, the one idea, the screen and the keys, memory, fighting, talking and stories, getting around (the ten districts in a table), making, building, the deeper places, names and leaving, every verb on one page, and what to do when stuck. Linked from the play page's header ("guide"), the home page and the sitemap. Its facts are the quickstart's and the code's as of tonight.
- A3: the analytics page is rendered for three windows, each a page with links to the others at the top: yesterday and today, 7 days, and the 30 the timer asks for (the index). The errors' own chips offer only windows narrower than the page's.
- D26, D27 and D28 closed at David's word: D26 (a talker's first move): not an issue live; Fallis and Tinsel stood named one room east and the storyteller told its first story at message 5 without a fight. Lean now b, or close. D27 (making spends what you fight with): milder live; rag thread came from `gather` in the Midden and `where ragpicker` pointed there; lean a still stands for `recipes`. D28 (a read that is an arrival): **did not reproduce.** Gnarlfoot_II left its room, sent nothing, and observed it at 118s and 246s: memory 0.98473 and 0.96846, the second an exact match to the half-life from the moment it left; `came_through` unchanged; no arrival in the log. Lean: close D28 (the local tester's reads were likely its own gate messages).
- The ledger is this page; `PFR.md` is what is open.

11-12 (the Exergue's plaque, from David at the keyboard: "I'm not able to write a line in the exergue"; one driver deploy)
- `append "…"` with nothing named, in a room that is not the writer's to add to, goes in the one thing standing there that takes their hand (the Exergue's plaque, the Ledger of Fines, a visitor book in someone's room); with two such things it says which to name. The play page's click on the plaque starts `append "` and nothing more, so the line was sent bare, read as an edit of the room, and refused "This room is the world's; only an operator changes it." Naming it (`append "…" exergue`) always worked. Not a change of tonight's; it was there to be found. Left: the dry run still says the bare form would be refused (R202).

11-11 (S1, by David's word; one driver deploy)
- What the floor was. Sampling the driver under one repeated verb found three costs on every command, none of them the verb: (1) the passphrase was verified with Argon2 on every command over HTTP and MCP, with the world held (twenty milliseconds on a fast machine); (2) "the last few thousand events", asked a dozen times a command, was a head query and several reads against the database each time; (3) each of those asks then copied every event it returned.
- The log keeps its newest 12,000 events in memory (`foxing_log::Tail`, around any backing): a read that starts inside them is answered there, an older one goes to the log underneath, an append goes underneath first. `recent_events` shares those events and no longer copies them. Checked against the plain log for every cursor and limit around the edges.
- A passphrase verified once in this process is known afterward by a quick keyed digest (Argon2id at its smallest, under a key drawn at start), kept beside the stored hash it passed: a wrong one still gets the full check and the lockout count, a changed one is a new stored hash and the old digest answers nothing. Held in memory only.
- The board, the citizens report and the words left for a name keep what they have folded out of the log and read on from where they stopped (`World::folded`, the `LogFold` trait), and all three are brought up to date in the pass the boot already makes over the log (`World::open_seeing`), so none reads it again with players waiting. The board's kept answer is 15 seconds, down from two minutes.
- Found on the live world after the first deploy of this: a new name's first login took 7.6 s, every time, with the world held. The arrival hook that delivers words left for a name read the log from that name's last hearing, which for a new name is the first event ever written. It is the third fold above.
- Measured on a local file-backed world of 14,000 to 20,000 events, release build, the world's own share of a reply: `satchel` 24 ms to 1.6; `look` 24 to 2.1; `wants` 25 to 2.7; `travel` 27 to 2.4; `say` 24 to 2.1. `board` (the verb) is 63 ms and `session` 32: both do real work over the store and are next (S3). Live, at 281,000 events, measured from Claude's session before and after: `say`, `satchel`, `map`, `where` 145 to 160 ms became 4 to 7; `look` 430 to 470 became 7 to 10; `wants` 1,020 to 1,130 became 5 to 8; `travel` 420 to 1,270 became 5 to 18. A three-step named walk, 2.3 s, is 28 ms. A new name's first login, 7.6 s, is half a second from Claude's session (0.3 of it the road there). `GET /v0/board` after a start, 8 s, is 1 s; the `board` verb is 0.8 s, the slowest thing left on a player's path: its census of the last day reads past the kept tail, and its present-day half walks the store (S3).

11-11 (the errors' window, at David's ask; rides the driver deploy)
- The analytics page's errors, and who sent them, can be shown for the last 24 hours (the default), the last 7 days or the whole window, by a chip over the table; each row says when it last happened. The probes line follows the window chosen.

11-11 (batch Z, by David's word: P2, H19, D21 b, A2; one driver deploy, one site deploy)
- P2: the play page's panels take only the rows that changed (a row that reads the same keeps its place, its hover and its listeners; You and Memory were emptied and rebuilt four times a second), and are redrawn when the world pushes or a command lands, and otherwise once a second for the figures that move on the clock; never in a hidden tab. Found with it: the map's "you are here" pulse and the header moon's drift were CSS animations that had the page restyle (and, the pulse, lay out) sixty times a second; the room now ebbs on a 1.2 s timer and the moon is still in play (it drifts at the door). The picture, idle, asks for its next frame by a timer. Ten idle seconds on the same headless build: 3.3 s of work before P1, 1.5 s after it, 0.5 s now; layouts 600 to 13. Not built as planned: rows are compared by their markup in place, not keyed by id with one listener a panel; it gets the same result for a tenth of the change.
- H19: `read doors` says how long ago each room was dug (rooms dug from 11-12; older ones have no date and say none); the cloak's view of the Threshold says "east (Fogbank: fight), north (Lantern Row: read and tell), west (Mudroom: dig)"; the play page's own onward-walking code is gone.
- D21 b: a room may keep a second kind beside its own, refilled on the room's own respawn: one bookworm in the Unfiled Archive beside the inklings, one silverfish in the Cistern beside the drowned. The four small kinds of the Stacks otherwise stay below the stair.
- A2: the analytics page is four sections under the top line (Who came, The door, How it felt, What broke) with a line of links to each, and a fold, More, for the five tables asked of once a month (daily, key paths, how the root was served, hosts by class, every address). Visitors of every class are one table with a chip per class to show it alone.

11-11 (A1, A4, A6, by David's word; one site deploy)
- A1: four figures head the analytics page, each for the last full day (UTC) with today so far, the day before and the week's daily mean, the lab left out: people and agents who came (addresses; crawlers out), new names through the gate, a command's median round trip at the keyboard, errors per thousand requests. An arrow says which way the day moved and whether that is good; each tile goes to its section.
- A4: errors show organic and lab apart, sorted by organic; requests for things the site has never had (`.php`, `.env`, `/wp-`, `/.git`, `/admin` and the like: scanners) are one line under the table and out of the rate; a 422 carries the reason the driver most often gave on that path.
- A6: counts right-aligned with their thousands apart; a day reads "Tue 11-10"; a table past twenty rows folds; lab rows dimmed; wide tables scroll in place; the top says when the collector last took a request in, in red when that is over 45 minutes ago.
- Not seen on the real store: Claude cannot open the page, so this was checked against a made-up one. The probe list is a pattern and will want a path or two added once real rows are seen.

11-11 (D32 a, by David's word; one driver deploy)
- At the third rung a creature weighs one guess at its word from a name every ten minutes; a story sent sooner is refused unweighed ("…it will weigh another in 9m 12s. That story was not weighed: it is neither a miss nor the word."), the wanted story still heard. The same guess sent again is not a new miss, at any rung. The story it wanted is no longer counted as a miss toward the hint. The third word's own hint says the rule. (Striking the second word starts the ten minutes, so the first guess at the third waits them out.) D33 and D34 filed from the same conversation.

11-11 (after the Watch of 11-11; one driver deploy)
- Of two ways the same length, a walk and `where` take the one with the least hunting on it, and the Bog of Second Thoughts last: the walker was taken through the Bog beside the Meadow, and the door's way to the Clocktower is now north, lane, orchard, up, not through the Shallows (H19's tie).
- The dry run knows a named walk ("walks north, north to The Orchard…, if the way is one you remember or the signs give") and its refusal of an exit lists each door with its room: the newcomer was told "no exit Shallows from The Fogbank; exits: alcove, landing, nook, north…" of a line the gate would have walked.
- What the Watch confirmed: the jar covers a retreat as written; the lantern's 4.0 to 12.0 hours shows on `map`; every making was found from `recipes` alone; the maker ended at level 38 and never stuck (last week's ended at 0); districts, signposts, the Lane, `out` and the stairs' names all read true when walked; `start`'s route worked as written (R177); the fall's figures agree across `mine`, `rest` and the fall line; the Docent sends a newcomer to unnamed wisps (D31 a).

11-11 (batch Y, by David's word: D26 b, D31 a, P1, P4, H17, the R items below by policy; the 422s; S0; one driver deploy, one site deploy)
- The 422s. Claude cannot read the box's access log, so the count by sender is not known from here; what a 422 is, is: (1) a JSON body of another shape on a POST path (`verb` for `command`, `name` for `actor`, `text` for `message`, a query sent as a string), answered until tonight in one line of plain text naming a Rust type, which an agent guessing the API retries blind; (2) an observe query naming a field or component that is not there; (3) `observe` sent as a command without a query. The play page and the box's own tools (Docent, Bindery, names, tidy) send none: checked by driving the page under every mod, over the socket and without it. Shipped: the common other names are accepted (`verb`, `name`, `from`, `text`); a body is read whatever its Content-Type (a JSON body sent as text/plain was a 415); a refusal is JSON that says what was wrong, the shape the path takes, and where the API is written down; and every refusal is counted by path and reason (`GET /v0/refusals`, operator only), shown on the analytics page as "Why the driver refused", beside a new "Who sent the errors" table (path, status, class, name, addresses). After a day of it the page says who and why.
- S0: see the S section. `/v0/economy` held the world for 7.5 s and `/v0/board` for 14.5 s on every request (a `GET /health` sent a second after waited 6.7 s); the sky mod asks for the first once a minute for every player who has it on.
- D26 b: `start` says when no creature has a name ("…fell a wisp east of the Threshold, then feed an unnamed creature to sixty") where it said a story to any named creature was welcome. D26's a and D28 are closed unbuilt at David's word ("wont fix, remove").
- D31 a: the Docent's page says the unnamed are for fighting and the named for telling, and to name a kind and its room when asked what to fight first.
- H17: the play page's map has a bar in each room's district color and a legend of the districts on it; a click on one asks `where` for its door.
- P1 (its larger half) and P4: see the P section.
- R177 (an errand's way says where it starts: "from The Midden: travel north, then tell…"); R178 (the Docent's page has what `gather` gives and where, and the Lane, the two stairs, named walks and signposts); R179 (an emptied room of the Stacks with a kind of its own says they are down, not that nothing lives there); R180 ("only a story that carries a word of its you have not found is heard before then"); R183 ("not itself yet: not to be fought for 49s"); R186 (`rest` says "1 strike in 4 lands" as the guides do); R188 ("a warden's charm that answers to Parrish"); R153 ("the next fall takes half the share of your memory the first did"); R161 ("it pays double again ready in 59m, and before then hears the wanted story once more, at plain pay").

11-10 (batch X, by David's word: H9–H16, P7, D20 b, D27 "buff the power of crafted items"; one driver deploy, one site deploy)
- H9: ten districts (the Door, the Diggings, the Wilds, the Library, the Deep Water, the Stacks, the Clock Quarter, the Conclave, the Exergue District, the Wistful Trail). A district's first room says so once a session ("The Clock Quarter begins here (2 rooms): …"), `map` lists what you remember by district (dim rooms marked), and `where <district>` gives the way to its door.
- H10 a: the Lamplighter's Lane, one room where nothing hunts, between Lantern Row (`lane`) and the Orchard (`lane`; back by `row` and `orchard`), so the Clocktower, the Conclave, the mint and the Trail are reached without the Shallows.
- H11: a signpost at the Fogbank, Lantern Row, the Orchard and the Clocktower (`read signpost`): each way's room, its district, what lives or hunts there and what lies beyond; `look` says the post is there.
- H12 a: "The Downward Stair" is "The Cistern Stair", "The Exergue Stair" is "The Mint Stair", the Archive's `down` is glossed "the Stacks stair, a level of 100", and its refusal says which stair it is and which it is not.
- H13: `travel out` goes back the way you came through any door, and a first arrival through a door that is not a compass point says "The way back is `out`, or `surface`."
- H14: past nine dug doors the Mudroom's exits show yours, the three newest and the three best remembered, with a count of the rest; every door still opens by its word, and `read doors` is the whole wall with each room's digger and what is left of it.
- H15, smaller than planned (the grid was already laid from the Threshold by compass and floor): a room behind a named door is drawn on the far side of the room it opens on, the list is by district and has every room on the grid, and other floors are named in a line of their own.
- H16: `travel <room name>` walks every step of the way it knows in one message and stops where something engages you, an ambush is set, a door refuses, or you fall ("You walk … toward The Mudroom and stop: …. Still to go: …"); `where` takes one word of a room's name and names every room that answers to it.
- P7: every reply carries `took_ms` (the world's share); `/play/?perf=1` shows frames a second, the cost of a frame, and the median and worst round trip of the last twenty commands; the page reports the same every twenty commands to `/play/perf.txt`, which the site's own analytics reads into a "Play page performance" section (not counted as visits). `foxing.perfFrame(ms)` for mods.
- D20 b: a guess refused inside the hour is on the record as a miss: it counts toward the hint's narrowing, and a declined word of the lent pool is struck from the pool the hint shows. (A refusal is free, so the narrowing can be bought with refusals; with D29's thresholds that is two and four at the second word, four and eight at the third.)
- D27: crafted things do more. The lantern holds the map three times as long (the night lantern five); the charm adds +0.5 to vividity (the warden's +0.7; were 0.3 and 0.45); a bandage on `rest` closes the bite and mends 10 (the surgeon's 20); the basin mends at three times the pace; a jar carried covers a `retreat` (no parting blow, the jar spent). `recipes` says where each making is found ("rag thread: the Midden's ragpickers, south of the Fogbank"). No crafted thing lends attack.
- R172, R174, R185 closed with these; R26, R170 and R189 trimmed to what is left.

11-10 (batch W, by David's word: H4, H6, H8, and D29; one driver deploy)
- H4: the Threshold's text is three invitations and one thing to do first ("East, into the fog: things to fight… North, under the lanterns… West, into the mud… If you do one thing first, go east"), its exits say what each is for ("east (Fogbank: fight), north (Lantern Row: read and tell), west (Mudroom: dig)"), and H5's line for a new name is folded into it; the Threshold gives nothing to `gather`.
- H6: `read signpost` at the Threshold: the three nearest creatures with names, each by the signed way with what it wants, and the newest room anyone dug.
- H8: the Mudroom's boots (`read boots`: who dug last, and how long ago), a rack of spades (`take spade`; carried, the exits of any room say whose each dug door is and how much of the room is left: "closet (Cartographer's Closet; dug by dave, 100% left)"), a line for its three best-remembered rooms, and text that says how to dig. The Docent's round already passed through it.
- D29 (David: "make the second word harder and the third word significantly harder"): the second rung hears three of its source's words to a story and the third hears one (more, and none counts: "You said 2 of the words my word could be; I hear one to a story at this rung"); the third word moves to another of the room's each time it is found, so the record of the last strike does not open it; hints narrow after two and four misses at the second rung, four and eight at the third, and the third never quotes the room. The multipliers stand (1.5, 2, 2.5). Carried words' flat 23 is untouched: D30.

11-10 (batch V, the Watch of 11-10's obvious fixes; one driver deploy)
- The kill's tail read as a contradiction and is said the other way round ("Nothing is left behind this time: a fight of 1 round leaves a thing one time in six, a fight of six rounds or more every time"); "1 door dug here"; a memory fed to a room says its half-life as `look` does ("2h 55m", not "3 hours"); short, a room of your own still says "Yours: 100% remembered, halving every 2h 55m"; a creature asked who has been here does not quote the asker or count it among those who came (R145's first cut did); a second petition's refusal says the stage once, with its number ("still before the Conclave, upon the table (stage 1 of 5)", not "upon the table, upon the table"); the world's own books say no holding before their first entry; the first strike on a named creature says "if it falls, it is a wisp again" (it read "felled" of one still standing); a half point of hit points rounds up ("19.5" read "20 left" and "12.5" "12 left" in one fight).

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

H9–H16 as they were planned (shipped 11-10, batch X):


What the map is now (63 seeded rooms, by the door they hang off):
- West: the Mudroom, and every dug room as a door off it (digger's word for the door).
- North: Lantern Row, then the Unfiled Archive (the Namery; the Lower Stacks, 20
  rooms behind a level of 100) and the Downward Stair to the Cistern and the Undertow,
  which surfaces in the Shallows.
- East: the Fogbank (the Midden south of it), the Shallows, then two lines: the
  Ossuary, the Gallery, the Oubliette; and the Orchard, off which hang the Wistful
  Trail (9), and the Clocktower with the Conclave's house (11) and the Mint (8).

What makes it hard to hold in the head (from the two Watches of 11-10 and the graph):
1. The three largest places to read and wander (the Conclave, the Mint, the Trail,
   28 rooms) are all behind the door marked "fight", four steps out through the
   Shallows, where hollows hunt. Nothing at the door says they exist.
2. Two stairs share a name's worth: "The Downward Stair" lets a level 0 into the
   Cistern, and the Archive's "down (Lower Stacks)" is refused as "the stair down
   asks a level of 100". "The Exergue Stair" is not "The Exergue".
3. Doors are named three ways: compass (east), a thing (dive, wicket, vault), and a
   place's own name used in both directions (escapement, exergue). A walker cannot
   guess the way back from the way in.
4. Every dug room is one flat list of doors on the Mudroom's wall, in the order of
   the exits table, by words only their diggers chose (alcove, landing, nook).
5. `map` draws by the order rooms came into view: a dug room lands far from its only
   door, two floors share a row, and the list of ways leaves rooms out (R170, R189).
6. `travel <room>` takes one step and `where` knows only rooms already walked, or a
   whole name (R174, R185): the world has the route and makes the walker spell it.

- **H9 · Districts, named and signed** (M). Each seeded room gets a district (the
  Door, the Wilds, the Library, the Stacks, the Deep Water, the Clock Quarter, the
  Conclave, the Mint, the Trail, the Diggings); `look` at a district's first room
  says so once ("You are at the edge of the Stacks: twenty rooms, a level of 100
  asked at this door"), `map` and the play page's minimap group and tint by it, and
  `where <district>` gives the way to its door. No room moves. **Lean: go**; it is
  the frame the rest hang on.
- **H10 · A way to the Clock Quarter that is not a fight** (M; **waits**: it moves
  a door). a) **lean**: a lane from Lantern Row east to the Orchard ("the Lamplighter's
  Lane", one room, nothing hunts), so the door's "read and tell" way reaches the
  Clocktower, the Conclave, the Mint and the Trail in three safe steps, and the
  Shallows stay the fighter's road; b) a fourth arm on the signpost only ("the
  Conclave sits: east, north, north, up"), no new room; c) as is.
- **H11 · A signpost at every hub** (S). H6's post, reused: the Fogbank, Lantern Row,
  the Orchard and the Clocktower each get one, saying what is at the end of each of
  its own ways, the level a way asks, and what hunts on it. **Lean: go.**
- **H12 · Stairs and doors that say where they go** (S; a rename each, so the names
  are yours). a) **lean**: "The Downward Stair" becomes "The Cistern Stair"; the
  Archive's `down` reads "down (the Stacks stair: a level of 100)" and its refusal
  names it so; "The Exergue Stair" becomes "The Mint Stair"; b) keep the names, fix
  only the refusal (R172). All checked against CLEAR's reserved words before any ships.
- **H13 · Every door has a way back by the same word** (S). Where a door is a thing
  (dive/surface, names/out, wicket, vault/out, hall/bar), `travel back` already
  works; add `out` as a second name for the way you came in at each of them, and say
  it on arrival ("the way back is `out`, or `surface`"). **Lean: go.**
- **H14 · The Mudroom's wall in rows** (M). The exits line of the Mudroom groups its
  dug doors: yours first, then the three newest, then the three best remembered
  (H8), then "and N more, by digger: `read doors`"; `read doors` is the whole wall
  with each room's digger, age and what is left of it. A door's word stays its
  digger's. **Lean: go.** A second question, yours: b) past forty doors, new rooms
  are dug off "rows" (the Mudroom's own side rooms, ten doors each) so no room has
  an unbounded wall.
- **H15 · `map` drawn from the door, by district** (M). The grid is laid out from
  the Threshold by the exits themselves (east is right, down is a floor below), a
  dug room sits beside the room its door opens on, each floor is its own block, and
  the list under it has every room on the grid (R170, R189 close with it).
- **H16 · Walk there** (S). `travel <room name>` takes every step of a remembered or
  signed way in one message, stopping where something hunts, a door asks a word or a
  level, or a fight begins; `where` takes one word of a room's name and says both
  when two match (R174, R185, R26 close with it). **Lean: go**; it is the largest
  saving for a player at the keyboard too (P4).

Order I would build them in: H9, H11, H16, H13 (no decisions; one Watch each),
then H12 and H10 on your word, then H14 and H15.

H4, H6, H8 and D29 as they were written (shipped 11-10, batch W):

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
- **D29 · Words pay faster than fights.** The live storyteller reached level 299 in four minutes on stories and carried words alone (six stories at 20 to 67 XP with the ladder's multipliers, five carryings at a flat 23); the newcomer reached 255 in 29 messages the same way; last night's fighter took an hour to reach 344. D17 a's nerf (one story an hour a creature, three an hour from others) is in, and four named creatures within three steps of the door make twelve an hour before the ladder. a) **lean**: watch it a day on the board before touching it (the hero metric is the day's average, which this does not move much); b) carrying a word pays once a creature a day; c) the ladder's multipliers come down. **Waits**: balance.
