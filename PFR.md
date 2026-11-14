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
- **D34 · Telling levels too fast** (S to change, yours to weigh). The teller of 11-11 was level 168 in 19 messages and five minutes, most of it not the ladder: an errand leg pays about 21 and chains (each receiver hands on a fresh word), a wanted story at double 12 to 17, a word bonus 6 to 9; the guides put a wisp at about 6 and the Stacks at level 100. a) **lean**: an errand leg pays once per giver per name per hour and a chain's later legs pay half; b) errands and double pay scaled so an hour of telling is about an hour of fighting (needs a figure from you for what an hour should be worth); c) as is: telling is the fast road, on purpose. The teller of 11-12, after D32 a: level 55 at four messages, 136 at ten (three minutes), about 190 at twenty-five, then flat, every named creature's hour spent; the ten-minute rule held (one of eight guesses weighed). What is uncapped is the errand leg, a flat 20 a message in a chain that does not end (Tinsel to Osgar to Tinsel, then the next pair): it is a) above, and the teller's first ask.
- **D30 · A carried word nobody gave you.** `wants` prints the phrase an errand carries ("the fourth door is open"); two of the Watch read it there, walked to the receiver, said it, and were paid 23 without meeting the giver. a) **lean**: the errand is taken from the giver (a `tell` to it, or standing with it), and `wants` says who has a word, not the word; b) as is: `wants` is the notice board. S for a.

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
- **R148** (S) The lent-word strike pay reads as fractions of an unnamed thing ("half again", "as much again", "one and a half times again", "2.5 times the pay"), and the in-hour "bonus alone" first strike (3 XP) paid less than a plain miss (4–6 XP): say the figure beside the words ("half again: 3 XP"), and say in `help story` that inside the hour only the word's bonus is paid.
From the Watch of six, 11-13, the night the second door opened (a newcomer, a faller, a teller, a walker, a manual reader on a kept name, a latecomer; all six woke at the Postern; see T16):
- **R218** (S) The dry run and the gate chose different ways of the same length (Threshold to the Orchard Wall: "north, lane, orchard, wall" in `check`, "north, lane, gate, wall" walked): one route search, one tie-break. And a named walk's reply, and its dry run, still list the doors taken and not the rooms passed (R193).
- **R220** (S) `wants` gives a creature in a dug room a way "by the signs" ("5 steps: lane, row, south, west, annex") that `where Osgar` and `travel "The Annex of Second Readings"` then refuse ("you remember no way there from here") until the room has been walked: one rule for all three.
- **R221** (S) The Threshold says "Nothing hunts here" beside "wisp (13/13, hits 2–4, holding 4, born of Fallis and Osgar)": a newborn shown as a plain wisp, which a bare `attack` refuses (the refusal now says it is newborn). `look` should say "newborn, left alone for its first hour" on its row. The chalk tally lists a name as "came through" that has never stood at the Threshold (it counts arrivals at either door): each door its own tally, or say "came into the world".
- **R222** (S) What the Postern still lacks beside the Threshold: the Docent in person, the day's name, a tally. The quickstart's list of signposts leaves the Postern's out, and its signpost says nothing of how far the Threshold is. `where mothwing` from the Postern names the Wall and not the Orchard; `where bench` offers two other benches before the Reading Bench.
- **R223** (S) The cloak shows the Threshold whatever door a name will wake at, and says so in a line; it could show the door a new name would wake at just then, or both in brief.
- **R224** (S) An absent name gets two answers: `tell Gnashfall` (away) "They will hear it when they return", `tell Gnewelpost` (gone, forgotten on leaving) "There is no one called Gnewelpost", at a quarter, and `where` answers with the whole catalogue of what it can find. Say "the world has forgotten that name" of one it knew tonight. `where <name>` recommends `log`, and `log` takes a count, not a name. A word said in a room after you left it reaches you only if you walk back ("Said here lately"), never through `messages`.
- **R225** (S) A room with nothing left to fight says when something drifts in and not where else to go from where you stand (the Orchard, one step `over`, had three mothwings): R151 again, from the gate. A felled unnamed wisp "gets up again, holding 17" where a mothwing comes apart, and nothing says why.
- **R226** (S) A message of thirteen statements had one left out of its reply "under the statement limit, the receipts first", and it was the one that mattered: leave out the reads first, and say which statement.
- **R227** (S) The quickstart, from its first whole reading by a tester: a first kill's memory "halves in about twenty minutes" (44 to 58 in the world); the gift allowance "120" where `statutes` reads 150 (say "as `statutes` reads it"); `attend` listed with no place where the world wants the Conclave's rooms; "the mildest things in the world" said of lanternflies in one place and mothwings in another; the busy-creature cap (three stories an hour from anyone) and the plain-pay hearing of a wanted story inside the hour are in `help story` and not in the page; `reply` to a letter whose writer is forgotten too is not described; the first letter a visitor sees may be no. 2.
- R209 was met again by this Watch (`mine` after a fall says "nothing touched any of it").
From the live Watch of 11-13 (a fighter, a teller, a maker on a kept name, a postmaster seventy minutes late; see T14):
- **R209** (S) `mine` after a fall says "Since you arrived: nothing touched any of it, and nothing of it has faded enough to say" when the fall has just taken memory from a chunk (twice in one night; with R157). Say the fall, the chunk and what it took.
- **R210** (S) `retreat`'s summary gives figures nobody can read ("A second stand now: about 4 rounds… and you have 0 in you. When you are whole: about 5 rounds against its 10, and you have 10"): label each. The last-blow warning names a jar of fog and not where one comes from (`recipes`); `rest` in a fight says "retreat, and it starts" and not that a retreat takes a blow. "The second fall of the night: it costs half what the first did" beside a second fall that took 3 where the first took 2 (the window had grown): say "half the share".
- **R211** (S) A felled wisp struck again by its id is answered by kind ("wisp is not itself yet and cannot be struck for 59s; it stands up with nothing to take"), where an inkling's id "is gone": say the id typed, and say once why one stands up and the other does not. The bookworm's round prints "(6 left)" and then "it mends by what it took (8 of 9)": the count after the mend, in order. A creature's hit points are fractional in `look` (5.2/5.2) and whole in the round.
- **R212** (S) What is left of the dry run against the gate: `name` and `remember` do not see what an earlier statement of the same message did ("holds 0" after a `remember` two lines up); a message of nineteen or twenty statements failed "CL-I6"/"CL-I5" with no reason given.
- **R213** (S) Things made: a new lantern's reply says "it holds nothing yet" and `read` says "with a faint memory of…"; the lamp's recipe says "a chunk (any…)" and refuses chunks of 12.7 and 5.7 with "You lack the memory to spend"; a trade for a thing not there says "There is no a small key here to speak of"; an entry in the Exergue's book is stamped 09:51 at 09:09 by the world's clock (whose clock stamped it?).
- **R215** (S) `tell <creature> "What do you want?"` says the story it wants and not the word it wants carried (`wants` and `look` do); "what is your word?" is the same paragraph from every creature and does not say which rung that one stands on. The quickstart's example names (Marrow, Lute) are nobody in the world, and `where Wyndle` is refused though the Register of Names lists a Wyndle (a name taken back: say so). The Namery lists "out" and "back", both to the Archive. `rest` says "3 strikes in 10 land" where the quickstart says one in four.
- **R216** (S) The Dead Letter Office, from its first visitor: `reply 1 ""` is refused by the checker as a missing argument at a quarter where the world's own line ("Answer it with what?") is better; `carry` of a letter nobody can receive is refused well. Nothing tells a newcomer the Office exists unless they read the quickstart: a line on the Library's signpost, and the Docent's news when a letter arrives.
From the live Watch of 11-12 (a guide reader, a teller, a mender, a latecomer):
- **R204** (S) A fall ends the message and the statements after it are dropped without a word (three attacks sent, one run): the reply should say what was not run and why.
- **R205** (S) `wants` promises "hears the wanted story once more" after it has.
- **R207** (S) `read doors` gives an age only to rooms dug since 11-12 (say "before the wall kept dates" for the rest); a named walk to the Lookout goes through the Bog with no word (it is the only way but the Hush: say so as it steps in).
- **R208** (S) What a newcomer still is not told: `recipes` on its first line says what you lack and not where it is (the where is at the foot); the charm's recipe says "worth 5 or more" and takes the whole chunk offered (20.6); the Exergue's plaque is "a visitor book" in `look`; `chunks` shows "(passing)" and says nothing of what it does; a gardener's row says its bite will not close and one of two never bled in six bites (chance or fault: check). `tell` to a name elsewhere should say when they last acted.
From the live Watch of 11-11 (a walker, a maker, a teller, a newcomer):
- **R192** (S) `travel The Stair of Indices`, written unquoted in a message through the gate, is refused "There is no exit 'stair'", and `take moth dust` (R198) for two arguments at a quarter of a message: a typed line is quoted for the player by the page, a written `[DO]` is not. The gate should quote a place or a thing of several words as the typed line does, or the refusal should say `travel "The Stair of Indices"`.
- **R193** (S) A named walk's reply lists the doors taken and not the rooms passed.
- **R194** (S) The Mudroom's digest of doors changed between two visits with nothing dug (a room at 53% left the line while rooms at 6% and 5% joined it): "best remembered" should be by what is left now, and the line should say its rule ("yours, the newest three, the best kept three").
- **R195** (S) The Lookout says "nobody on it but you" beside "longsight is looking at spacepants in The Picnic That Waited". The Threshold's signpost gave Fallis's want three minutes after it had turned.
- **R196** (M) A creature strikes on every statement of a message while a fight is on, `take` and `craft` among them (two blows before the maker's own `attack` ran, 10 to 5 of 30): right by the round-per-statement rule, and said nowhere in the reply. Say it on the first such blow ("each statement is a round while it fights you"), or let statements that are not blows ride free once a message.
- **R197** (S) `read` a lantern gives "three times as long" where `map` gives "half every 12.0 hours"; `recipes` does not say a first chunk comes from a first fight; a poultice carried on a `rest` that needs none says nothing.
- **R199** (M) The word ladder disagrees with itself: one reply said "It was the story it wanted… the want stands" and "Not the one it wanted: it listens for book, overdue…" (the want had turned); Wyndle's second word was not struck by any word of the sentence its hint quotes (the sentence is also its old want, so a story carrying it is heard as the want and likely never tried against the word); `check` refused a story the gate then heard at plain pay; a creature full for the hour answers eight guesses with "takes no more" and no count, though `wants` says a word-carrying story is heard.
- **R200** (S) The Docent: two answers were of where the asker had been, not where it stood; its first tip names a wisp that is "not itself yet".
- **R201** (S) For a newcomer: "holding 71" in a creature's row is never explained; the dry run says `tell` "needs an argument (text)" where the gate says two.
From the live Watch of 11-10:
- **R181** (S) `mine` before any fall says only "Nothing in the world is yours yet" (the quickstart sends a reader there for falls): with memory in the window it should say what a fall now would take ("about 20 XP, 15% of what you carry").
- **R184** (S) Ten `attack` statements into an empty room return ten identical refusals, and the "drift in within about 42s" countdown is only seen by spending a message: after the first refusal in a message, the rest are said once ("and 9 more the same"); a held `attack` that waits for the next arrival is R174's cousin (M, a word first).
- **R187** (S) Creatures listed "holding 0" paid 20 to 30 XP a kill (bookworms), so the row's holding does not say what a kill is worth: say what it is worth.
- **R189** (S) `session` counts "1 line written" after a `describe` and an `append`, and "1 said" where nothing was said aloud (the story).
- **R190** (S) Short, a room of your own says "Yours: 100% remembered" on arrival the same after four seconds or four hours: say what the absence did ("it had faded to 97%; you bring it back whole"). The builder's one ask.
- **R191** (S) `observe` with no argument fails the whole message at a quarter ("'observe' needs 1 argument (query)"); a `describe` aimed at a room that is not yours should be refused in a line that says whose it is.

From the Night's Watch of 11-10 (on a local copy):
- **R151** (S) `chunks`, the Docent's first line and the Threshold all send a newcomer east to the Fogbank's wisps; with three of the Watch there the room was empty for most of an hour ("There is nothing here to fight… Something will drift in within about 34s"). The pointer should name a second room when the first is empty (the Orchard's mothwings, Lantern Row's lanternflies), and the empty room's line could say who took them.
- **R155** (S) `look` lists a creature "holding 0" in the reply after the fall text says it "keeps" what you lost (the revenant, 58.7/64); its holding should show what it took.
- **R156** (S) The `chunks` footer's half-lives move with the Clocktower's rate ("30m 0s … 3h 0m", later "28m 33s … 2h 51m") and nothing says why; one clause ("by the Clocktower's memory rate, 0.95 now").
- **R157** (S) `mine` after two falls: "nothing touched any of it, and nothing of it has faded enough to say" under an empty "What remembers you:"; with nothing kept, say that, and the falls.
- **R159** (S) The word hint after a story that used two of the listed words ("moon", "fog") says "One story from you since has carried none": if a pool word was carried and is not it, the hint strikes it from the list (D20's refusal does; the heard story does not).
- **R160** (S) A creature's holding after a story rose by less than the chunk the teller got (27.0 XP to the teller, 61 → 74 in the creature) and by more after a plain one (6.0, 74 → 90): the teller's pay and the creature's keep are two figures; say the creature's in the reply ("it keeps 13 of it").
- **R162** (S) In one gate message of five attacks the switch line repeats on each later statement ("wisp-200 is gone; you turn on wisp-201." twice); say it once.
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
- **S7 · The box** (yours, later). One VPS runs the driver, PostgreSQL, nginx, the
  Docent and the Bindery. Before a second box: PostgreSQL's settings for the log
  table (it is append-only), a connection pool, and the driver built in release with
  its worker count set. A second world or a second process per world is past 10x.
- **S8 · What the game itself does at ten times** (design, yours). Three of a Watch
  already empty the Fogbank; ten times the names want spawn counts that follow who is
  in the room, more first rooms (R151), and the Mudroom's wall (H18). These are the
  limits a player meets before the server's.
- **S9 · A write the schema does not know is refused when it is made** (S; from the
  outage of 11-14, T17). The store takes a `FieldSet` on an undeclared field while
  it runs and refuses the same event on replay, so one such write stops the next
  boot, and the rollback's too. Refuse it at `record`, or accept it on replay with
  a warning; Claude leans the first, with the second as well for the log already
  written. Beside it: the deploy's health check could replay the live log against
  the new build before it stops the old one.
- Order: S9, S6 (a hundred next), S4. S2b when you choose.

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
    times three quarters to one and a half by the memory's vividity; the kinds
    hold 40 to 100 and hit 3 to 8.
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
- **C6 · The commonplace book** (S/M). You forget; a book you carry does not. A made
  thing that writes one line a visit of its own accord (your first fall, a name you
  gave, the room you dug) and can be read whole when you hold nothing. It is the
  answer to "what did I do here" for a player who comes back small, and it is a
  second-tier recipe, so it sends people to the Stacks for a reason other than XP.
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
- Claude's order, if left to it: C11's rest, C4, then C6. What it needs from you
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
- **Y6** On the VPS, confirm nginx serves the site's text compressed and with cache
  headers for `/play/mods/*.js` (the repo has no copy of that config; P5 leans on it).
  And the analytics timer on the box renders hourly where the repo's
  `site/deploy/setup-analytics.sh` says five minutes: re-run it when convenient.

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
