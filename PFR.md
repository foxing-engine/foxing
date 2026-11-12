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

The library's floors (from the Watch of 11-07)

From the live Watch of 11-10 (four on the live world, by David's word; see T11)
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

Away from Moondial and Entropic (the engine's and the game's old names): done in the
tree 11-06 (R16–R20); the box's half is Y1. The `MOONDIAL_*` fallbacks in the driver
and the tools go in a release after Y1 has run.
- **R26** (M; was B6, pulled 11-04) Unreproduced: "You again" on `leave` (rare); `board` at ~4 s. (The named walk that stopped short is H16's walk now, batch X.)
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
From the live Watch of 11-12 (four, arriving four minutes apart: a guide reader, a teller, a mender, a latecomer; what it fixed is in `PFR-SHIPPED.md`):
- **R203** (M) A creature that has "learned to shove" throws a fighter out of the room mid-fight, into the Shallows or a stranger's dug room ("The wisp shoves you north, and the door does the rest"): two of four met it, on their first wisp, and no guide says it. Say it in `look` before the fight in plain words ("this one shoves: a blow may put you through a door"), keep a shove from sending a first-hour name into a room that hunts, and put a line in the quickstart and the guide.
- **R204** (S) A fall ends the message and the statements after it are dropped without a word (three attacks sent, one run); a `rest` in a fight at 7 of 30 drew the killing blow, said only afterward by the narrator; `retreat` at 2 of 30 takes its parting blow and kills, with no warning. Each reply should say what was not run and why, and `retreat` should say when the parting blow would be the last.
- **R205** (S) `check` disagrees with the gate in four more places: a wanted story inside the hour ("refused: one story an hour" of a story the gate hears at plain pay), a passing name's `append` in a welcomed book ("append is a kept name's"), a message of nineteen or twenty statements failed "CL-I6"/"CL-I5" with no reason given, and `wants` promising "hears the wanted story once more" after it has. With R202, one pass over the dry run against the gate's own rules.
- **R206** (S) `attack inkling-7858`, already dead, turns on another inkling ("…is gone; you turn on inkling-5355"): a strike aimed by id at what is gone should be refused. `where bookworm` and `where silverfish` name only the Lower Stacks, not the one in the Archive and the one in the Cistern (D21 b). A trade's reply reads "for Gnimbus's a wisp ember".
- **R207** (S) The Mudroom's "newest dug" names rooms at 0% left (newest should be by what still stands); `read doors` gives an age only to rooms dug since 11-12 (say "before the wall kept dates" for the rest); a named walk to the Lookout goes through the Bog with no word (it is the only way but the Hush: say so as it steps in).
- **R208** (S) What a newcomer still is not told: `recipes` on its first line says what you lack and not where it is (the where is at the foot); the charm's recipe says "worth 5 or more" and takes the whole chunk offered (20.6); the Exergue's plaque is "a visitor book" in `look`; `chunks` shows "(passing)" and says nothing of what it does; a gardener's row says its bite will not close and one of two never bled in six bites (chance or fault: check). `tell` to a name elsewhere should say when they last acted.
From the live Watch of 11-11 (four on the live world, after batch Y; the three fixes it led to are in the ledger):
- **R202** (S) `check` says a bare `append "…"` in the Exergue would be refused as an edit of the room; the gate now writes it on the plaque. The dry run should follow.
- **R192** (S) `travel The Stair of Indices`, written unquoted in a message through the gate, is refused "There is no exit 'stair'", and `take moth dust` (R198) for two arguments at a quarter of a message: a typed line is quoted for the player by the page, a written `[DO]` is not. The gate should quote a place or a thing of several words as the typed line does, or the refusal should say `travel "The Stair of Indices"`.
- **R193** (S) A named walk's reply lists the doors taken and not the rooms passed.
- **R194** (S) The Mudroom's digest of doors changed between two visits with nothing dug (a room at 53% left the line while rooms at 6% and 5% joined it): "best remembered" should be by what is left now, and the line should say its rule ("yours, the newest three, the best kept three").
- **R195** (S) The Lookout says "nobody on it but you" beside "longsight is looking at spacepants in The Picnic That Waited". The Threshold's signpost gave Fallis's want three minutes after it had turned.
- **R196** (M) A creature strikes on every statement of a message while a fight is on, `take` and `craft` among them (two blows before the maker's own `attack` ran, 10 to 5 of 30): right by the round-per-statement rule, and said nowhere in the reply. Say it on the first such blow ("each statement is a round while it fights you"), or let statements that are not blows ride free once a message.
- **R197** (S) `read` a lantern gives "three times as long" where `map` gives "half every 12.0 hours"; `recipes` does not say a first chunk comes from a first fight; a poultice carried on a `rest` that needs none says nothing.
- **R199** (M) The word ladder disagrees with itself: one reply said "It was the story it wanted… the want stands" and "Not the one it wanted: it listens for book, overdue…" (the want had turned); Wyndle's second word was not struck by any word of the sentence its hint quotes (the sentence is also its old want, so a story carrying it is heard as the want and likely never tried against the word); `check` refused a story the gate then heard at plain pay; a creature full for the hour answers eight guesses with "takes no more" and no count, though `wants` says a word-carrying story is heard.
- **R200** (S) The Docent: two answers were of where the asker had been, not where it stood; its first tip names a wisp that is "not itself yet". `tell <creature> "What do you want?"` still gets the stock line (R158, third Watch).
- **R201** (S) For a newcomer: "holding 71" in a creature's row is never explained; `chunks` says "Level 14.50" and `rest` "Level 15"; the dry run says `tell` "needs an argument (text)" where the gate says two.
From the live Watch of 11-10 (four on the live world):
- **R181** (S) `mine` before any fall says only "Nothing in the world is yours yet" (the quickstart sends a reader there for falls): with memory in the window it should say what a fall now would take ("about 20 XP, 15% of what you carry"). (The line after a fall says "half the share" since batch Y.)
- **R182** (S) "It holds 60 now." and then `name` refused: "a lantern holds 60; a name is written in memory, and it takes 60": the holding had faded under sixty between the lines and both print it rounded. Say the figure to a place when it is within one of the bar ("holds 59.8").
- **R184** (S) Ten `attack` statements into an empty room return ten identical refusals, and the "drift in within about 42s" countdown is only seen by spending a message: after the first refusal in a message, the rest are said once ("and 9 more the same"); a held `attack` that waits for the next arrival is R174's cousin (M, a word first).
- **R187** (S) `chunks` prints a value over its own first size ("10.23 of 10.2", "23.62 of 23.6"): the "of" is rounded to a place and the value to two; and creatures listed "holding 0" paid 20 to 30 XP a kill (bookworms), so the row's holding does not say what a kill is worth: say what it is worth.
- **R189** (S) `session` counts "1 line written" after a `describe` and an `append`, and "1 said" where nothing was said aloud (the story). (The map half closed with H15, batch X.)
- **R190** (S) Short, a room of your own says "Yours: 100% remembered" on arrival (batch V) the same after four seconds or four hours: say what the absence did ("it had faded to 97%; you bring it back whole"). The builder's one ask.
- **R191** (S) `observe` with no argument fails the whole message at a quarter ("'observe' needs 1 argument (query)") and `help observe` shows no example; a `describe` aimed at a room that is not yours should be refused in a line that says whose it is.
- R158 (a creature asked what it wants answers the stock line) was hit by all four creatures the storyteller asked: confirmed live, and the storyteller's one ask.

From the Night's Watch of 11-10 (on a local copy of batch U; the obvious fixes are in the ledger as batch V):
- **R151** (S) `chunks`, the Docent's first line and the Threshold all send a newcomer east to the Fogbank's wisps; with three of the Watch there the room was empty for most of an hour ("There is nothing here to fight… Something will drift in within about 34s"). The pointer should name a second room when the first is empty (the Orchard's mothwings, Lantern Row's lanternflies), and the empty room's line could say who took them.
- **R152** (S) The petition's receipt lists four stages ("read into the minutes, then before the Committee on the Matter, then reported from committee, then decided") and `petitions` counts five ("stage 1 of 5"); /v0/conclave's `stage` is one less than the gate's. One list, one numbering.
- **R154** (S) A killing blow is printed clipped to what was left ("hits you for 5: 5 → 0 of 30" from a creature listed "hits 7–9"): say the blow whole ("hits you for 8; 5 was all you had").
- **R155** (S) `look` lists a creature "holding 0" in the reply after the fall text says it "keeps" what you lost (the revenant, 58.7/64); its holding should show what it took.
- **R156** (S) The `chunks` footer's half-lives move with the Clocktower's rate ("30m 0s … 3h 0m", later "28m 33s … 2h 51m") and nothing says why; one clause ("by the Clocktower's memory rate, 0.95 now").
- **R157** (S) `mine` after two falls: "nothing touched any of it, and nothing of it has faded enough to say" under an empty "What remembers you:"; with nothing kept, say that, and the falls.
- **R158** (S) `tell <creature> "What do you want?"` and `"Is the word hinge?"` get the stock line ("I hold what I am given…"): answer the want from `wants`, and say a guess is made by a story that carries the word.
- **R159** (S) The word hint after a story that used two of the listed words ("moon", "fog") says "One story from you since has carried none": if a pool word was carried and is not it, the hint strikes it from the list (D20's refusal does; the heard story does not).
- **R160** (S) A creature's holding after a story rose by less than the chunk the teller got (27.0 XP to the teller, 61 → 74 in the creature) and by more after a plain one (6.0, 74 → 90): the teller's pay and the creature's keep are two figures; say the creature's in the reply ("it keeps 13 of it").
- **R162** (S) In one gate message of five attacks the switch line repeats on each later statement ("wisp-200 is gone; you turn on wisp-201." twice); say it once. And an attack by id on a creature that has gone turns on the next one unasked (it killed the storyteller): refuse with who is here.
- **R163** (S) A short name finds the wrong thing: `read charm` read Charmkin (found) over the moth charm just made; prefer what the reader made, then what it carries, and say which it chose.
- **R164** (S) `describe "…" here` is refused ("There is no here in your satchel or standing here") at a quarter; `here` should mean the room.
- **R165** (S) A fixture's `read` gives only its line ("a lamp (item-166), whole (1 line)…") where a carried thing recites its life; and a spent charm cannot be read at all ("There is no item-197 here to speak of"): a fixture's past is read like any made thing's, and a spent thing leaves its last line where it was spent.
- **R166** (S) Charmkin ("your next three fights' memories more vivid… one each") shows no count and no line when it acts: say "Charmkin warms: two left" on the kill it touched.
- **R167** (S) The room text after a memory is fed to a dug room says "Kept in memory by Gnarrowby (with wisp, telling Loam a story): the memory of a creature of theirs, fed to it": the clause is the creature's, said of a room.
- **R168** (S) "The fog gives up shapes: 2 wisps" with one wisp in `look` (a named one beside it was counted, or one had gone): count what arrives.
- **R169** (S) A wisp drifts in at "0.3/8.3": a wounded one that left comes back unmended and reads as a new spawn; say "comes back, still hurt", or mend it away.
- **R170** (S) The map of a floor reached by two stairs draws two rooms five cells apart on one row ("05· . . . . . . 45@") with nothing between: H15 (batch X) gave the list an "Other floors" line and put a named door's room on the far side of its parent, but the grid of such a floor is not yet checked by a Watch. The list still says "Lantern Row" beside "The Assay Office" (the article kept on some names only).
- **R171** (S) The Lookout says "the nine rooms from the Stile up" and the walker counted eight; the Long Afternoon's "(8 off)" is unexplained.
- **R173** (S) A thing that lay in a room and is gone on the next short arrival leaves no line ("as you left it" over a room missing its charm): short, a room says what has changed since ("Charmkin is gone").
- **R175** (S) `messages` counts the Docent's welcome again on every read ("the last N of N"); `read` with no thing means the room's page in the Conclave's rooms and "Nothing here has writing on it" in the Bar: one meaning, said in `help read`.
- **R176** (S) The budget's arithmetic is not sayable from the replies (a message of six with one refused cost 2.0; a free read after arrival left 58.75): the reply says what this message cost and why.

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

What is left of the front door, each yours to call (H1 to H17 are in the ledger):

- **H18 · Rows past forty doors** (M; **waits**, it was H14's second question). a)
  **lean**: as built, the wall is one room and `look` shows a digest of it; b) past
  forty doors, new rooms are dug off the Mudroom's own side rooms, ten doors each.
- **H20 · One name for the mint** (S; **waits**, the names are yours). H12 as approved
  made the stair "The Mint Stair"; the district it opens on is "the Exergue District",
  your name for it. a) **lean**: the stair follows the district ("The Exergue Stair",
  as it was, with its exit glossed "the mint"); b) as shipped.

## The keyboard (P): performance for a player at the keyboard

Mapped 11-10 at David's ask. Measured on a local build in a headless browser at
1440×900 (software drawing, so the idle figures are an upper bound), and from this
session to the live server. Nothing here is built; say `P1 go`.

What was measured:
- Idle, standing in a room with nothing happening, the page keeps about a third of
  one core busy. Two causes: the picture redraws all of itself sixty times a second
  whether or not anything moved (`tiles.js`, and `sky.js` the same), and every panel
  (You, the window, Here, the lane, the first-hours card) is torn down and rebuilt
  four times a second: about 5,600 nodes and 820 listeners live, 64 layouts a second.
- Load: seven requests; `index.html` is 265 KB in one file (styles and script
  inline) and `tiles.js` 98 KB; the schema (18 KB) and vocabulary (16 KB) are fetched
  on every load. The CLEAR checker is already loaded only when first used.
- A command's round trip was 56 to 87 ms on a local debug build; a plain request to
  the live server took 330 to 570 ms from this session, most of it a fresh
  connection each time, so the live figure a player sees over the open socket is
  not known yet (P7 measures it).
- The transcript is never trimmed; lines are sent one after another, each waiting
  for the last reply.
- Keys that exist: Up/Down history, Ctrl+arrows to travel, arrows on an empty line
  for west/east, `?` for help, Ctrl+L, Ctrl+U. There is no completion.

- **P1b · The room as a kept layer** (S/M). The floor and walls drawn once and kept, so
  an ambient frame draws figures only (P1's first half is live: the picture draws on
  change, ten frames a second idle).
- **P5 · A lighter door** (M). Styles and script out of `index.html` into two files
  with a hash in their names, cached for a year, the page itself small and never
  cached; `tiles.js` preloaded; the schema and vocabulary kept in the browser and
  revalidated; the composer and observer tabs loaded when first opened. Check the
  server compresses text (your side to confirm on the box: Y item below).
- **P6 · Trim the transcript** (S). Keep the last five hundred lines in the page and
  fold the rest behind "earlier" (still there to scroll to), so a long night does
  not slow every new line.
- **P8 · A quiet mode** (S). One setting: no ambient motion, no sky, no sound; for a
  slow machine or a long session. It is P1's floor.

Order now: P6, P5, P8, P1b (idle is 0.5 s of work in ten seconds since batch Z; what is left is the picture's ten frames).

- **Y (yours)** On the VPS, confirm nginx serves the site's text compressed and with
  cache headers for `/play/mods/*.js`; the repo has no copy of that config.

## The analytics page (A): a cleanup, planned 11-11 at David's ask

The page is sixteen tables in the order they were added, thirty days each, with the
operator's questions ("is anyone here, did they get in, is it fast, is anything
broken") answered only by reading all of them. Claude cannot open the page from its
session (the site host is not reachable from there), so this is planned from
`site/analytics/render.py` and checked against a made-up store; send a screenshot of
the real page and the rest gets sharper.

- **A5 · The driver's side** (M). One admin read (`/v0/ops`): requests by path with
  median and worst `took_ms`, time the world's lock was held, the log's size and the
  slowest reads since boot. The page shows it under *How it felt*. It is S1's
  before-and-after.
- Left: A5. (A1 to A4 and A6 are live: `PFR-SHIPPED.md`, 11-11 and 11-12.)

## Ten times the players (S): planned 11-11 at David's ask

Measured from Claude's session against the live world on 11-11, at about 264,000
events in the log, with P7's `took_ms` (the world's own share of a reply):

| asked | the world took |
|---|---|
| `say`, `satchel`, `map`, `where` | 145 to 160 ms |
| `chunks`, `recipes` | 140 to 210 ms |
| `time` | 370 to 395 ms |
| `look` | 430 to 470 ms |
| `travel` (first into a room) | 420 to 1,270 ms |
| `wants` | 1,020 to 1,130 ms |
| `GET /v0/economy` | 7.5 s, with every other request waiting on it |
| `GET /v0/board` | 14.5 s, the same |

The same verbs on a fresh local world take 15 to 25 ms. The shape of the problem:
one world behind one lock, and several reads that go back to the first event the
world ever wrote, each time, while holding it. Cost grows with the log, not with who
is here: at ten times the events the board is two minutes and `wants` ten seconds
with nobody new in the room.

- **S2 · Know where the floor is** (S; with S1). `took_ms` split in the ops read
  into waiting for the lock, the handler, the log append and the pushes to sessions.
  If the append is the floor (a commit to PostgreSQL per event), batch a command's
  events into one transaction and one commit.
- **S3 · Indexes over the store** (M). `store.entities()` filtered by kind and by
  parent is walked on most verbs (8,300 entities now, most of them faded chunks and
  rooms); keep maps by kind, by room, by owner. And a sweep: entities forgotten for
  good leave the store (they stay in the log).
- **S4 · Readers do not wait on writers** (L). Public reads (`/v0/map`, `/v0/board`,
  cloak, observe) served from a snapshot swapped in after each write, so a read never
  takes the world's lock. With S1 this is what lets a hundred watchers cost nothing.
- **S5 · Pushes by room** (M). After any change every open session is woken and
  re-observed; with ten times the sessions that is ten times the work per command.
  Wake only sessions whose rooms, selves or subscriptions the change touched.
- **S6 · A load test that is a Watch** (M). A script of fifty then five hundred
  passing names on a copy of the live log (not the live world), walking, fighting and
  telling at a player's pace, reporting P7's figures; run before and after each step.
  It is also the test for the gate's budgets and nginx's limits at that size.
- **S7 · The box** (yours, later). One VPS runs the driver, PostgreSQL, nginx, the
  Docent and the Bindery. Before a second box: PostgreSQL's settings for the log
  table (it is append-only), a connection pool, and the driver built in release with
  its worker count set. A second world or a second process per world is past 10x.
- **S8 · What the game itself does at ten times** (design, yours). Three of a Watch
  already empty the Fogbank; ten times the names want spawn counts that follow who is
  in the room, more first rooms (R151), and the Mudroom's wall (H18). These are the
  limits a player meets before the server's.
- Order now: S2 (what the floor is on PostgreSQL, now that reads are memory), S6, S3, S5, S4.

## Content (C): expansions, planned 11-12 at David's ask; nothing here is built

What the world has now: ten districts, about seventy rooms of its own, some thirty
kinds of creature, two tiers of recipes, the errand and the word ladder, the Conclave,
Morphos twice a day. What the Watches keep finding is where it is thin, and that is
where this starts: the door is crowded, the middle is empty, and almost nothing asks
for two names at once. Sizes are for a first cut that a Watch can play. Say `C3
brief` for a page on one, or answer the lettered ones.

- **C1 · A second way to begin** (M). Three of a Watch empty the Fogbank in a minute
  and the fourth finds "nothing here to fight" as its first reply (R151, four
  Watches running). A second starter room on the read-and-tell side with its own
  first step: the **Reading Bench** off Lantern Row, where an unnamed thing takes a
  first story from a name that has never told one (no name needed, once), so a
  talker's first minute is not a walk to find a listener. a) **lean**: that; b)
  spawn counts in the Fogbank that follow who is in the room; c) both.
- **C2 · The middle: levels 20 to 100** (L; the largest gap). Between a wisp at 15
  and the Stacks' stair at 100 there is the Trail and little else, and the teller of
  11-11 crossed it in five minutes while a fighter cannot cross it in an evening. A
  mid district that is neither: the **Reading Rooms**, a mezzanine over the Archive,
  six rooms whose creatures are fought by what you tell them and told by what you
  fought (a moth that only takes a memory of a fight you nearly lost; a proofreader
  that strikes harder for every word you repeat). It teaches the Stacks' tricks one
  at a time before they come three to a room.
- **C3 · The Dead Letter Office** (M; the one Claude would build first). Words left
  for a name that never came back, and for names the world has forgotten, now have a
  fold of their own (S1) and nowhere to go. A room off the Namery where they are
  pigeonholed: read the ones addressed to nobody, carry one to whoever now holds the
  name, or answer it. It makes absence into content, which is the game's subject,
  and it is the first thing two players do for each other without meeting.
- **C4 · Things for two** (M). The Wardens (320 hit points) are the only fight sized
  for company, and `retreat` already says "bring another name". Three small ones on
  the way there: a bell in the Clocktower that takes two hands in the same minute, a
  door on the Trail with a key in each of two rooms, a creature that only takes a
  story told in two voices (one name starts it, another ends it within the hour).
  Each pays both, and says who you did it with on the record.
- **C5 · A year in the world** (M, then small things forever). The Conclave keeps
  five holidays and nothing else knows the date. A calendar the whole world reads:
  the fog heavier in one month, the Orchard's hours ripe for one week (the only time
  its fruit is a material), a night when every named creature wants the same story.
  The happenings engine (X13) already fires twice an hour; this is a table of dates
  for it.
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
- Claude's order, if left to it: C3, C1, C4, then C2 with a brief first. C5 can
  start as three dates. What it needs from you first: which of C2 and C9 is the
  world you want (a deeper library, or a wider one), and whether C7 is in the spirit
  of the thing or too much rope.

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

## What the cohorts taught (T): durable; read before briefing a cohort

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
