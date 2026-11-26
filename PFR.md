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
- **R261** (S) The Diggings, from the look at H18 (11-20). (From 11-20 a named creature walks out of a room forgotten a day, to the Mudroom; seen live 11-21: Fallis is in the Mudroom, and Kennoch's word follows it there.) **The live wall is six doors, four at 0%**: `look` at the Mudroom could leave a door under 1% off the exits line (it still opens by its word; `read doors` has it).
- **R272 · Left from the twenty-eighth Watch.** The Souvenir Shop's picture shows a floor and a figure: no shelves, tickets or jar are drawn, and it is very dark at night (so is the Wickmaker's Bench). Door and name labels still stack in Lantern Row, the Mudroom and the Orchard Wall when three or more names stand close. On a phone a "Show me" newcomer lands with the picture folded. A thing's condition ("19% remembered") does not lower its price. Memory remembered into a thing sells at about a third (a sink by rule; unexplained). `take` of a thing just sold says it is in your satchel. The death card lists a spent poultice under "Still yours". `retreat` at low health can kill with no warning of the parting blow's size. A new name can reach level 100 inside fifteen minutes (a ragpicker is worth 40), and the notice that the world's leave has ended arrives mid-fight. A fed unnamed creature can be felled for the memory just fed it. A palace chunk is spent by `buy` without a word that it came off the shelf. The shop traded on 26 November UTC, a US holiday: which calendar shuts it is not said. `leave` on a passing name does not say its memories go with it.
- **R271** (S/M) Left from the twenty-seventh Watch (11-25: Pnesta at the keyboard; the phone and the gate to come). **The satchel panel is far down the page** under the map and the other panels, and the five places with it: a player who does not go looking never sees them (for David: where it should sit). Its tiles have no legend for ringed and dashed, show a thing at work twice (in its place and in the grid), cut long names, and a panel reflowing under the pointer stowed a thing by a stray click. The sword, the lute, the primer and the poultice share the parcel icon. **On a phone** (Pnimbus): no panel of the page can be reached at all (the panels menu ticks and nothing shows); a creature's tap menu covers the lower half of the picture; a speech bubble is cut mid-sentence; "What you were fighting is gone" is said again two and three rooms on; a thing a fight leaves is taken by another name inside a minute; a second lute appeared in a satchel after three changes of figure (not reproduced). **Through the gate** (Gnarlx): a refused message gives a score and no reason (two-argument verbs want each argument quoted; `gather` takes none), told only by `MODE=check`; the reply to making a thing offers a passing name `remember` into it, which is refused, and does not say where the thing went; `mine` lists a thing at work as "in your satchel"; of two lanterns nothing says which is lit; "the fight took most of your hit points" was said of six in thirty-four; reading (`help`, `look`) in a fight draws a blow. **What is held is not drawn** (a hunter holding keys is drawn with a sword): the third step of C21. The sword's one in twenty is not said anywhere a player would see it (the sized-up line). `chunks` does not say which word of a row is the one to type, and a first lantern spends a level-0 name's whole memory. Unnamed creatures carry bars and no names in the picture; an exit's label can sit on a creature's bar; the Docent's label covers its head. `make` is answered "the nearest it has is craft" and does not run it. The panels' layout is not kept between visits.
- **R270** (S/M) Left from the twenty-sixth Watch (11-25: Pnock and Pnadir at the keyboard, Pnuma on a phone, Gnomon at the gate). **The page, for David**: on a phone the words get about a third of the screen (header two rows, picture, status row, the card), and at a desk the first-hours card and the chip row leave a few lines; the picture is 300 of 390 pixels wide on a phone, and the title is cut at the top of every phone screenshot (cause not found). **The picture**: the Mudroom draws doors the text does not list (`stay`, `chart` at 0%) and not `hearth`, and says "+4 doors" where the text says "5 more"; some creatures in the text are not drawn (Prattler, the crier) and many drawn ones carry no label; a speech bubble sat over the player's own figure with no speaker in the Orchard Wall; rag thread on the floor is a dark spool on a dark floor; a named creature's menu offers "Attack" in red. **Tell**: a story in another language cannot meet a want (the want listens for English words) and its miss line lists them; `wants` runs on for a creature told this hour; a want turns ninety seconds after it is met; a miss's line once called the story's own keeper by the creature's name. **The meaning rungs, to judge**: hints that mislead as often as they help ("Only one person knows" for a small theft nobody noticed; "Somebody stayed" for waiting on someone who is not coming) beside found-lines that are lenient (a spoon that "vanished" was a theft; a near story in Spanish was found); the middle verdicts are hard to reach on purpose; every named creature is past its first word, so the three-guess first rung (D57) is unseen. **Small**: `greet Gedr` sets a standing greeting; `help` ends with keyboard keys on a phone; a room header says "remembered 100% · bare" where `read doors` says "Bare, 0% left"; a name reaches level 100 in twenty-two minutes on four stories and is told the leave ended in one line.
- **R268** (S) Left from the terminal Watch of 11-23 (Pnib, at the keyboard; the plain errors shipped the same morning). **Not done, each small**: a walk by a room's name uses an exit's word as a noun ("You walk gate (Postern), west to the Hall of Lost Steps"); `dig burrow mistle` with `burrow` already a door answered "`dig` wants 2 things after it" where it should say the door's word is taken (not reproduced off the page); `remember` says a memory by what it is of ("the memory of wisp") and not by its word; one kill is said three ways ("comes apart", "goes down… gets up again", "Nothing left."); `recipes all` repeats "(the whole chunk is spent…)" on thirteen lines and writes "ink  a dab of ink: is a dab of ink"; the word "chunk" in signatures where a player's word is "memory" (asked of David 11-22, open); `wants` says "you have it from Kennoch" to a name that only stood near it (true: standing with it is hearing it; the room's own line says it "will hand you a word"); a bare `attack` after a kill starts a new fight; no line says a level fell when a memory is spent; the status pills run off a phone's right edge. **From the folds of 11-23, evening, not done**: bare `recipes` as chips; `help` with its keys paragraph folded and `help <verb>` cut to a signature and a sentence; `lore`'s bullets folded; `messages` already seen cut to a first sentence; a countdown that ticks in the transcript; exits and room names in prose as chips.
- **R267** (S) Left from R262 to R265 after the batch of 11-22, late. **Not done**: door levels shown on the picture's exits; `mine` reporting use (who stood in the room, lines in the book) and `read doors` saying what is inside; a shove that arrives between messages still leaves the next message's statements stale. **Not reproduced**: each mark announced twice in the foot (the logs of that Watch show one line a mark). **Kept as they are, by David's word**: a shove on a hurt name, mild swearing, the Reading Rooms' remembered fight, the picture open on a phone. **To watch**: what a case against the house is used for, and whether twice the fee is the right return.
- **R266** (S) From the twenty-third Watch (11-22, five at the keyboard on the first hour), not yet done. **To watch**: with the leave held for a road, a name can stay on one for its whole first hour and climb unhunted (the hour is the cap); whether that is used. **Page**: the card can cover the Docent's own words; the hunter's and teller's `look` steps teach little; on a phone the picture still takes the top third. The Threshold's own text still says "If you do one thing first, go east" beside the Docent's question. **Not reproduced**: an exit read "west (The The Threshold)" from the Fogbank. **Still open from before**: "did you mean" for a mistyped creature name, unquoted text and arguments in the wrong order.
- **R265** (S) From the twenty-second Watch (11-22, on L4), not yet done. **A person is not told**: the page first says `study` at minute 14, on arriving at the Mezzanine behind a door that says "for names of about level 50, and you hold 0"; `lore` is never named before a first folio; the first-steps card points at the Fogbank and stayed on "1 of 8" through three unlocks (R262's stale card, now in the way). **Long replies**: `lore` and a folio scroll so that only the tail shows; on a phone the log is seven lines tall. **Making what is learned**: every thing "with a past" asks a deep-Stacks material (a lens, a ribbon, a tooth) a reader cannot win, and `recipes` names no source for a lamprey's tooth; river weed is behind the Cistern's door (120), so `thread` is; `gather` gives one thing a room, an hour, and the Orchard gives a ripe hour in Ripe Week where `recipes` says moth dust; nothing marks a recipe as learned or says when it lapses but `lore`. **Puzzles**: Makings and Pins are one or two reads and no walking; the Book is given by its own title; the twelfth seat wants the Docent; an unquoted `study folio 12` is an answer and shuts the folio on a miscount. **Names and rooms**: `where The Bindery` found a name called Bindery; `where Courtroom` is refused and `where Court` is two rooms. **The rest**: a name felled in a hunted room while its page was closed, and the death overlay then took no click from the test's browser; `read pencil` at the Filing Window finds nothing where `lore` says pencil; the DO row offers `study` in some of its rooms and not others; a backticked command with quotes in it is not drawn as a chip; flyting still half pays alliterative nonsense (14 to a creature's 20); `damn` passes the list that stops worse.
- **R264** (S) From the twenty-first Watch (11-22), not yet done. **Tell**: `deeds` lists no felling of a named creature (only "lying shaken"); `repute` counted four creatures knowing a name while one said it "no longer knows you"; a newer jest replaces an older with no word, and a lapsed challenge says nothing to either side; each mark is announced twice in the foot (the act, then the narration); "X has answered you" is said when a mark of yours was struck; a wearer sees only a jest's first words; the four named creatures say the same line for the same form; a new name cannot sing or mourn until something has been felled; jest pay varied 6 to 9 for like jests; the eulogy's "Premature, but well meant" read wrong to one long awake; `beseech` paid about a plain story. **Court**: the Clerk takes the whole fee, a name's only memory, for a plea it says at once cannot succeed; a case cannot be brought against the house; nothing tells a keeper what a title changes, and `read wall` (Householder) and `repute` (no title yet) are two ladders unexplained. **Hunt**: a shove can arrive in the foot of the next message, so the statement after it is stale; a badly hurt name is shoved into a hunted room; "yours, if nothing joins it" is said with two in the room; a door said "does not ask you again" and asked (perhaps a message rolled back); the Under-Docent does not say what the Stacks do to a name on its leave; an unknown verb refuses a whole message. **Keep and the rest**: the lamplighter and the porter gave one line to two questions; "Tinsel follows X for three steps" is said of two standing still; a slip still fails when a want turns between the reading and the answer.
- **R263** (S) From the twentieth Watch (11-21, night), not yet done. **Hunt**: `travel down anyway` sent with other statements came back with no words (not reproduced); the keeper's first felling paid 2.3 ("worth 50 when the keeper got it") and its second 281 while the room said "worth 490": the three figures want one sentence; the hour counter gave "frees at 23:59:52" and then "frees at 01:00:00"; a room's chatter reached a name a minute after it left, in another room's reply; nothing at the Oubliette points on to the Drowned Wing. **Tell**: `wants` said Fallis "pays double again in 47m" and Fallis paid double twelve minutes later; `wants` said a leg of Kennoch's word was carried that was not; a want still turns between `wants` and `story` (it costs nothing now); the walkers cannot be told anything ("It is only a lamplighter"); Gedder's hidden-word reply says "refused" and "still heard at plain pay" together; the Reading Rooms ask a remembered fight that a level-200 teller cannot have ("it barely touched you"). **Keep**: the porter's making did not come in 45 minutes and nothing says when; a keeper cannot strike one visitor's line; the keeper's "1.25 times as long" shows nowhere it can be checked (`look` 9h15m, `mine` 9.5 hours); the Mudroom wall said Tended of a room that said Cherished; two glowworms stood in one kept room (one may have strolled in); "Tinsel comes in, looking for" a name not there; nobody is told when a creature they keep is felled. **Read**: the Namery slip's count moves while a name walks to it (two creatures named in ten minutes turned a right 5 into a wrong one and a ten-minute lock); `read lamplighter` says there is none while `look` lists one, `index` knows no runner, census taker or tinker and takes "lamplighter" for a citizen; `time` gave the Conclave's division as 27 minutes off and then 24 hours off, and none came; `where` says the first hour's words to a name at 110; the Clocktower door sizes up nothing for the hourhand; the runner's bag is "empty" and then has a "top letter".
- **R262** (S) From the Watch on the four loops (11-21), the fixable, not yet done. **Hunt**: statements queued behind a shove run in the room you were shoved into ("No such creature here. Here: keeper."); a bare `attack` refuses the Oubliette's keeper as "an officer of the house"; `retreat` leaves you standing in the room with what you fled, and two readers took that for a bug; a learned trick (a numbing bite) is named in `look` and not what it does. **The page**: no button for `anyway` when a door stops you, and the exit in the picture shows nothing of the door's level; the first-steps card stays on "attack wisp" in a room with no wisp and tells a fighter to give away its first win; the red mark meant nothing until the text explained it; nothing on arrival says what was done to you while the page was shut. **Tell**: a want that turns between `wants` and the story costs the hour's one miss; "The story it wants is still heard, at plain pay" was said in a refusal of that story. **Keep**: `mine` reports fading and not use (who stood in the room, lines in the book); `read doors` gives dates and percentages and nothing of what is inside; an invited creature leaves to find its keeper and `mine` places it wrongly; a done want still shows in the setter's `wants` as "not yours to do". **Read**: a slip's answer with an apostrophe is awkward to send; `recipes`' hints of where makings are found are out of date (a wisp ember from the Lane, hollow water from the Court).
From the Watch of six, 11-13, the night the second door opened (a newcomer, a faller, a teller, a walker, a manual reader on a kept name, a latecomer; all six woke at the Postern; see T16):
- **R256** (S/M) The Night's Watch of 11-18, the fifteenth (Gnash at level 100, Gnosis the teller, Gnubbin the builder, Gnettle the maker). **Seen working:** the leave ending at 100 and not coming back under it; the Stacks stair either side of 100; no second "hour is over"; the signpost's "for about 3 hours more"; "keeps half as much"; `tell` saying when a name last acted; `session`'s told, said and written; the room's "It had faded to 62%"; `describe` refused with whose room; the dry run in the gate's words; `recipes` with places and the whole chunk spent; the lantern's 12.0 and 4.0 beside `map`; `rest` saying why a poultice stayed; `where Wyndle`. **With David as D52:** R199's change reached nobody (every named creature is on the third rung). **Balance, for David:** a teller reached level 100 in four minutes (four wanted stories at double and a carried word), so the leave ends almost at once; the Midden's floor holds 76 moth dust, 34 wisp embers and 77 ripe hours, which makes `gather`'s hour moot; mothwing memories come in at 2 to 4, under the charm's 5; `status` is refused at a quarter (an alias?). **Open:** `rest` with a poultice and a bandage names one; the Mudroom lists doors at "0% left" as live exits with no mark on them (a fixture now only delays the unmaking, 11-18; a mark on the door was offered and not asked for); `wants` gave six steps to a creature an earlier listing put at three "by the signs"; a miss reply gave "none of the words it could be" beside the three-minute rule (weighed or not?); `messages` listed four words where `session` said "3 heard"; a signpost want cut at "crossed the" was taken as the slip's answer; two wisp fights left nothing, so a new name could not make a lamp, and the remember-then-name dry run went untried; a want turns seconds after it is met though the signpost says "3 hours more" ("sooner if someone meets it" is there). **Not seen:** a creature full for the hour, a bleeding bite, another name taking a thing first, two things fitting one short name.
- **R255** (S/M) The Watch of 11-17, the fourteenth (Gnocturne the reader, Gnarl the fighter, Gnimbus the teller, Gnapsack going down; Gnapsack was cut off by a machine restart at 00:28 and its findings are from its log). **Seen working:** held attacks at the brink, "You felled it already", `take` naming who took a thing, the quarter for a refused message with reads beside it, reads in all eight new rooms, twelve slips and the Reader's Round, the Under-Docent, the Tide Table against the water (table: low 23:40, Court clear about 4 minutes; seen clear 23:38:59 to 23:42:51), `travel` stopping before a room under water, a brine glass holding the loss to a point every ten seconds for two minutes. **Balance:** nine slips put a new name at level 290 in a quarter of an hour and the level-100 stair is no bar (slip pay stands, D47); the free walk of the Lower Stacks that followed is closed (the leave ends at level 100, shipped 11-17) and waits for a Watch to see it. **Open:** the Cistern Stair's slip figure "does not keep" (fair only to a fast reader); the Lookout's slip count (24) did not match the rooms the reader had stood beside; the Division Lobby's wrong-answer line does not say which way, and "midnight sitting" beside `time` saying half past; the Under-Docent is not in `look` ("You are alone"); `where Bindery` answers as of an absent visitor (`where "The Bindery"` works); `time` said the Clocktower strikes in 19m 34s at 23:40; a refusal of a story not about the new want says "has heard the story it wanted from you inside this hour already"; the rules of the creature's word (one guess every three minutes, one word to a story) appear only in a refusal, and what finding it pays is never said; `wants` sends a carrier toward the Carrels without saying the Stacks stair asks level 100; a poultice of the second tier took an ordinary hit and gave "20 back" where the recipe says a bleeding bite and ten; the fall's line does not say when a swing is worth sending (R253); "a message runs whole or not at all" beside a refused statement whose neighbor read ran; an unnamed wisp held "Tinsel's story of the way home" with Tinsel in the room; the Docent could not say where a brine pearl comes from (the recipe says "the wing's deepest rooms"; one lay on the Binder's Tidepool floor). **Not seen:** "no longer knows you", a second carrier's pay, the starfish's blow going wide, a splint, anything of D43 or D44.
- **R254** (S/M) The open Watch of 11-17, what is left. **The hour's sixty messages ended three of four evenings early**; a read beside a refused statement no longer makes a whole message, and whether that is enough is for the next Watch to say. (`wants` says from 11-20 when another name has carried a word already.) **A wanted story paid double twice inside five minutes**: the code holds a teller to once an hour with each creature, so this was two creatures or a fault not found; the footnote now says "from each creature". **The creature's word** stays as it is (David, 11-20: c). **The Docent sent a name to Gedder "in the Fogbank"** when it lived in a dug room. A shout from the Hush arrived at the Lookout fourteen minutes later to nobody. A revenant cost 42 hit points and paid 1.6: a trap more than a fault.
- **R253** (S) The strike team of 11-17, what is left. **One of three never had the first-hour line** (Gneiss, named 23:33:09, walked six rooms at 00:34 and was told nothing; the other two were told at 00:33): cause not found; it fell in its first hour and was fifteen minutes silent before the hour ended, and the other two were neither. **The Shaken spiral** stays as it is (David, 11-17: no rule change); the fall's line could still say how long until a swing is worth sending.
- **R252** (S) D41, what is left: forty-one rooms answer `read` from the list, and the Trail's Stile, Well, Lookout and Long Afternoon and the Exergue's rooms from their own code. Not yet: the rest of the Stacks (the Stairwell, Cistern, Undertow, Gallery, Oubliette, the Dark Reading Room and the rooms past the Index Stair), the Conclave's rooms, the Namery and the wings. A batch a Watch.
- **R251** (M) What is left of David's six calls of 11-16 (D34, D35, D36, D37c, D40c, D42; all live). **Still unseen:** an arm growing back (it fell inside one low), a salve, a dart, a lantern or a charm acting, a Warden fought at its new figures, the Purse taken at six hundred, a full creature answering a guess, a creature at home fading at half the pace over an hour, the shelve line at eight chunks. **Built 11-17 on David's word and unseen:** one starfish blow in five goes wide; a starfish splint and a brine glass (Claude's recipes: "we can change later"). The way south was already shut while any starfish is in the Court. **To watch (D43, D44, live 11-17, none of it seen in play yet):** every fall to a whole starfish goes into the same arm, the first to come off, so what earlier parties lost is one thick arm in front of four plain ones (a fall of 100 is about 107 hit points on that arm; "the arm holding least" is a one-line change if that reads wrong); a boss that holds memories and is felled still gives one memory to the striker alone and stands (only the coming apart is shared), and the Purse is its taker's; a share is capped at each name's own level plus sixty, so a low name's share of a Warden is mostly lost; one blow and a walk away earns a full share ("a tenth of the damage" was the other choice). **Also to watch:** a body has no ceiling now, so a name at level 2,000 has 230 hit points and the Wardens' 27 to 29 is a different sum for it; a night lantern found on Lantern Row's hook is second tier, so whoever carries it lands nine in ten; the starfish learns (it had a shove after three arms and a bite that will not close after four), which no text of its own says.
- **R250** (S) What is left from the resident of 11-16: nobody signed the visitor book in 85 minutes; the same few unnamed creatures at the doors are fought over by four names at once. With R190 (a room of your own saying what your absence did, which `mine` does well and the room does not): the resident's first ask, as it was the builder's. (The Gatekeeper brings the answer in his own voice from 11-20; seen live 11-21.)
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

- (Nothing waits here. H18 was closed 11-20 as built, with the wall made to read
  well at any length; see the ledger. Side rooms off the Mudroom come back as a
  question only if a wall of a hundred live doors is ever seen.)

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
  - *Spread over the year, 11-25* (David: "more events spread throughout the
    year"): thirteen more, a season in every month, each one of three turns (a
    night of one story, a week when wants pay more, a time when memory keeps).
    November's three stand where they were. Left: seasons with things of their
    own, as the Ripe Week has its fruit (a material, a creature, a room that opens).
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
- **C7 · What players write for players, what is left** (live 11-21: `entrust`; see the ledger). **Unseen live**: nobody has left one yet (through the live gate, 11-21, both shapes of `entrust` parse and are refused rightly for a creature the asker does not keep). **To watch**: with no limit on who does it (David, e3), two names can pass a memory back and forth through a creature and take the world's pay on top each time (a wanted story's doubling, or 20 for a word): what a ring makes of that is the first thing to read off the record. **Not built**: the play page has no button for it (the line is typed); a creature's tap menu could offer "leave a want". **The line** is the one place a player's prose is put before other players' agents: it is quoted and named every time, 80 characters, no marks or links; a model's eye on it (d3) was not asked for.
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

- **C11 · The Kept House** (M; **proposed 11-25**, David: "let's plan more content: more rooms, more creatures, more puzzles"). Six rooms off the Lamplighter's Lane where named creatures board, and six new residents, each with a word and two meanings. The Watch of 11-25 ran out of meanings in five minutes with six named creatures in the world. A house puzzle: the Visitors' Book, lines left by the residents, each to be matched to who would have written it.
- **C12 · Doors that listen** (M; **proposed 11-25**). A new kind of lock, eight of them in rooms that exist: a door with a line over it that opens to a story about what the line means, weighed as a creature's meaning is. Behind each a cabinet, one small room with one thing in it found nowhere else. Asks the judge once a try, one try an hour a door.
- **C13 · The Branch Line** (L; **proposed 11-25**). A train on a timetable between five halts, one new room at the edge of each of five districts, and a carriage that is a room while it moves. A ticket inspector who walks the train and wants a ticket found by reading the timetable right; pigeons on the platforms. Miss it and wait, or walk.
- **C14 · A second Reader's Round** (S to M; **proposed 11-25**). Twelve new slips, each needing two rooms to answer, and the Errata: one deliberate mistake written into a room of each district, found by reading and reported at the Overdue Desk. The first Round was the best-liked thing four testers found (C10).
- **C15 · The Observatory** (L; **proposed 11-25**). Five rooms over the Clocktower for names past level 150, where there is little now. An orrery set from the almanac's own dates; a creature that strikes where you stood a turn ago; comets that cross a room on a schedule; one room that is only there in season (C5's leftover).
- **C16 · Six kinds for the rooms there are** (M; **proposed 11-25**). No new rooms: creatures that do something no kind does now. A borrower that takes a thing and brings it back changed; an understudy that wears the last name to leave the room; an echo that fights with your own last telling; a flock of footnotes, many and weak; a knife-grinder and a pedlar who walk rounds and deal. Each wants a drawing at 32.
- **C18 · The Vale of Grief** (M; **David's, 11-25**: "5 rooms that tell a story about the world before this one"). Five rooms walked in order, nothing hunting, each holding one part of what was here before the world forgot itself; the telling is in the rooms, the things lying in them and what one creature in each says, never in a paragraph of history. A name that has walked all five in order remembers it as one memory. **To settle**: what the world before was (David's to say, or Claude drafts three and he picks); where the Vale opens from (past the Wistful Trail's lookout is the natural door).
- **C19 · The Vellum Way** (M; **David's, 11-25**: "a path that leads to something unexpected yet valuable"). A long path of thin rooms, each a skin scraped and written over, that gives nothing along the way and does not say where it goes. **To settle**: what is at the end. Drafts: (a) a room that holds the one memory you have lost most completely, given back whole, once; (b) a desk where one line you write is added to a room's own text for good; (c) a creature that will carry your name's oldest memory in its own keeping, so it never fades while that creature lives.
- **C20 · The LetterForm Archive** (M to L; **David's, 11-25**: "puzzles hidden in different typescripts"). Rooms of specimen sheets, each puzzle carried by how a text is set and not by what it says: a message in the letters set in italic; a line that reads only down the capitals; a sheet in blackletter where one letter is from the wrong fount; a page whose long s gives the date away; monospace where the column is the clue. The play page renders the typefaces; through the gate an agent is given the same text with its setting marked. **To settle**: how many (eight to twelve is a round like the slips); whether solving them all opens something.
- **C21 · Equipment you can see and choose** (M to L; **David's, 11-25**: "let's plan for a better equipment experience"). Today some twenty made things work from the satchel by being carried: nothing is put on, nothing shows on the figure, two of a kind are both "carried", and what is at work in a fight is said only in the fight's lines. Proposed: (1) **places on the body**, four: a hand (a dart, a buckler, a lantern), the neck (a charm), a pocket (a bandage, a jar, a salve: the one thing used up for you when its moment comes), and the light you walk by; `wear`/`hold <thing>` and `stow`; a thing works only from its place, so carrying three charms is a choice of one. (2) **a panel on the page**: the figure with its four places, each thing's icon, its uses left as pips and how well it is remembered, tap to swap; in the satchel a thing says what it would change against what is worn ("9 blows in 10 land, where 3 in 4 do now"). (3) **it shows**: the thing in hand and the light are drawn on your figure in the picture, on you and on other names. (4) **in a fight** the line says what your things did this blow, once, in one clause. **Settled, 11-25** (David: "1:limit / 2:players to place"): the places are a limit, a thing works only from its place; what a name already carries is left in the satchel for it to place.
- **C22 · Four figures to choose from, drawn finer** (M to L; **David's, 11-25**: "a new player design where they can choose one of four avatars, improve the resolution make it prettier"). Every name is one figure today, 32 pixels, tinted. Proposed: four figures, different in outline and not only in color (so they tell apart at a glance and in grey), chosen at the door when a name is first held and changeable at the Namery (`figure`); each with the frames a figure has now (breath, blink, two steps, a strike, a hit) and a place for a hand and a light (C21). **Settled, 11-25** (David: "3:b / 4:follow the 4 SGLs / 5:yes"): everything to 48 in one go, all fifty-six kinds and the four figures; the four are the four loops, a hunter, a teller, a keeper and a reader; and the rooms are in it, floors and walls drawn and not flat tint. First sketches of the four at 48 are in `design/figures48/` (11-25), for David to look at before anything is built on them. The hunter carries a short sword and a buckler (David, 11-25, after a quiver "doesn't look quite right"). **Each figure begins holding its own thing, a real thing in the hand** (David, 11-25: "b"): the hunter a short sword, the reader a book, the keeper a ring of keys, the teller something yet to be chosen. **To settle**: what each does (small, and of its loop); what the teller holds; whether the buckler is a second hand (five places) or part of the hunter's drawing.
## The four loops (L): what a person does here, and what each needs (David, 11-21)

David, 11-21, on playability for people: "my gut says we need to polish each one in
different ways." The four, and his direction for each, in his words where quoted. One
clock runs under all four: what is earned fades. Each loop wants a standing of its own
that others can see (Hunter, Teller, Keeper, Reader), so a new person can pick a way in.

- **Antics, live 11-21** (David: "let's have the non-hunting creatures conduct more business, move around, act weird sometimes and generally make the place more lively. we can add new characters as needed"; `antics.rs`, and the ledger). *To watch:* whether a line every other minute in an occupied room is lively or noise, above all at the foot of an agent's reply; whether strollers bunch up in the Threshold; the walkers' small sprites. *Not built, and worth it:* walkers that do real work (the lamplighter's lamps mattering to a room's light; the tinker trading a making for a story; the runner carrying a letter to a name); creatures reacting to what a player just did; officers of the houses leaving their posts on holidays only.
- **L0 · Decided 11-21** (David: "L1a:2 / L1b:1 / L2a:3 / L0:hunt first", then "m1:yes /
  m2:1 / m3:1"): the map is reordered now and not only gated; gates are soft; mischief
  may be aimed at other players, freely, with moderation filed for later (B, below);
  Hunt is first. The first cut of L1 (a) is live 11-21: see the ledger.
- **L1 · Hunt** (fight, take the memory, go deeper). *Live 11-21:* the Wilds run one
  way (the Shallows no longer open onto the Orchard), the Conclave is `up` from the
  Orchard with nothing hunting on the way, the Court is west of the Postern, and
  thirty doors say who they are for (`rooms::GATES`). *To tune after the Watch of
  11-21:* every gate level is an estimate from hit points and attack, not from play;
  a message of several statements stops whole at a gate (it runs whole or not at
  all), which is right and may read as a wall to an agent; the Long Afternoon has no
  gate because its wicket asks a word where `anyway` goes. *Still to plan:* (b) and (c). "We'll need to plan a re-ordering
  of the map so there's some sequencing at first with either hard or soft gates.
  Tactical battles go under this thrust as well. We'll flag equipment in this bucket as
  well." Known rough: deaths not seen coming, the Shaken spiral (R253), falling cheaper
  than resting at low level and newcomers overmatched in the Wing's upper rooms (C9).
  To plan: (a) the order of districts for a new name and the gate on each door, hard
  (a level, as the Lower Stacks' stair) or soft (the door says who it is for, and the
  travel stop of R244 holds you once); (b) a fight with choices in it beyond `attack`
  and `retreat` (a creature that shows what it is about to do; a guard, a feint);
  (c) equipment: made things already work while carried (buckler, lantern, charm,
  bandage), with no slots and no one place that says what you have on.
- **L2 · Tell, live 11-21** (David: "L2a:1 / L2b:1 / L2c:5 / L2d: recourses allowed / L2e: start with your suggestions"; `bard.rs`, and the ledger). *To watch and tune:* every number is a first guess (the forms' 1.25 and 1.5, a flyting's 12, the rungs at 2/4/6/9, the known-teller pay to 1.5, coaxing's 35 and 15, twenty seconds a rung for "stay", an hour for a mark); forms are five more payments a listener a day, so a teller's hour is richer than it was; a flyting's weighing can be learned and written to (long, alliterative, the name in it); "stay" on a keeper or a sitter is a strong thing for a Laureate (80 seconds). *Not built:* a eulogy for a room forgotten; talking a creature out of what it holds; a nickname a teller gives themself a stage name by; the Crier and the creatures' own wants in the new voice; nothing on the play page shows a title or an epithet but the text. *Before it is widely used:* the moderation backlog (below) is now owed: an operator cannot yet strike a jest or an epithet.
- **L3 · Keep, live 11-21** (David: "L3a:1 / L3b:1 / L3c:3 / L3d:2 / L3E:yes, everything decays"; `tending.rs`, and the ledger). *To watch and tune:* every number is a first guess (the points, a visit's 6 and 3, three paid visits a day, the wander odds 40/30/20/12, the titles at 2/4/6/9); two names can visit each other's rooms daily for 9 between them; a keeper cannot protect a creature it is feeding from another name's blow (R262); "cheaper digs" was asked for with the perks and not built, because a dig costs whatever chunk is given and has no floor to lower. *Not built:* the residents have small sprites and not the large ones; nothing shows a room's standing on the map; creatures do not greet a renowned keeper by title.
- **L3 · Keep, as first set down** (put fading memory into what lasts). "We should add graphical upgrades,
  improved beneficial creature spawning etc. for well tended rooms. High level keepers
  are renowned and that has effects." Known rough: little comes back (a visitor book
  unsigned in 85 minutes, R250; a room does not say what your absence did, R190; four
  of six doors on the live wall at 0%). To plan: tiers of a tended room and what each
  shows in the picture; what a tended room draws (a creature that mends, lights,
  carries); renown, who sees it, and what it does.
- **L4 · Read and make, live 11-22** (`lore.rs`, and the ledger). *To watch and tune:* every number is a first guess (three days, 20 and 30, ten minutes, the rungs at 3/6/9/12 and what each opens); seven recipes that any name could make last week now want a reading first, which a hunter after a surgeon's bandage will meet as a wall; the preparations make ragpickers, inklings and the deep cataloguer optional for a reader; an end's answer is the same for every name and will be told in a week (the folios are the part that cannot be); a folio's question is drawn from ten rooms near the Door and the doors' own levels, and will be learned as a table by an agent that keeps notes. *Not built:* a crafting bonus beyond the unlocking (a Scholar's cheaper chunk, a better chance of the third tier); puzzles in the Stacks and the Wing; a reader's title beside a name in `look`; anything on the play page.
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

- **Moderation, when players can act on players** (filed 11-21 at David's word: "we'll file a backlog for moderations later"). L2 will let a teller put a nickname on another name and talk names into things, with no limit (L2a:3); C7 already puts a player's own line in front of others (`entrust`). Before either is widely used: an operator's verb to strike a line or a nickname and say why; a player's own `ignore <name>`; a record of who did what to whom that the Court can read; and what a first-hour name is spared.

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
