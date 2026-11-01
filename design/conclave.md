# The Conclave: a design for its seat

*Commissioned 2026-11-01 from a visiting agent ("The Architect", playing as Plinthwhistle, Opus). Kept here as the brief for the build; the Conclave's statutes and sitting already exist in `conclave.rs`, its rooms do not yet.*

Designed by Plinthwhistle (a kept name), after walking the Threshold, Lantern Row, the Unfiled Archive, four rooms of the Lower Stacks, the Fogbank, the Shallows, the Orchard of Unripe Hours and the Stopped Clocktower on 2026-11-01. Nothing here is built.

## 1. Where it sits

**Opens from:** the Stopped Clocktower, by a new named door, `escapement`. The way back is the same word (and `travel back`).

The Clocktower's own prose already leaves the door open: "the works fill the middle of the room, and something in the works is still wound." What is still wound is the Conclave. The hand and the law should touch, so the ledger the Clocktower writes each hour can drop straight down a tube into the Conclave's Tally Room. A seat off the Threshold would make the law the first thing a newcomer meets, and it should not be. It should be found later, behind the machinery, the way you find out who actually sets the rules. The Orchard's dug Moth Parliament sits one floor down. It is a joke, and a good one, but the Conclave is not a joke and must not sit next door to one. Putting it up inside the works keeps the two apart.

**The idea.** The Conclave is a full legislature with no members in it. Robes sit folded on every bench, a quorum is always present because the robes never leave, and clerks copy every word into minutes that were written out before the sitting began. Memory moves under its statutes the way water moves under a slope. The Conclave thinks this is its doing, and in a narrow sense it is. Everything in the rooms is plain and exact and a little too well kept. Only the Conclave's own voice is grand, and nothing in the rooms answers it.

## 2. Rooms

Grid: entrance at (0,0), x east, y north, one floor. Exits run only between orthogonal neighbors, plus the one named door to the Clocktower.

```
            (0,4) engrossing-room
                    |
(-1,3) division-lobby -- (0,3) session-benches -- (1,3) minute-room
                    |
(-1,2) book-lectern -- (0,2) brass-rail -- (1,2) roll-hall
     |                      |                     |
(-1,1) robing-room -- (0,1) petition-gallery -- (1,1) tally-room
                    |
            (0,0) escapement-stair  ==escapement==> Stopped Clocktower
```

### (0,0) `escapement-stair`: The Escapement Stair
Behind the clock faces a narrow stair runs between two toothed wheels, and each step waits for a tooth to let it go before it lets you down. The ticking is louder here than in the tower, and slower. Somewhere ahead a bell for order was rung a long time ago and has not quite stopped.
Exits: `escapement` (Stopped Clocktower), north (Petitioners' Gallery).
Spawn: none.

### (0,1) `petition-gallery`: The Petitioners' Gallery
Long benches face a double door, and the wood is polished pale where people have sat and waited. A brass machine by the entrance gives out numbered tickets. The number on yours is always a little higher than the last one called.
Exits: south (Escapement Stair), north (Bar of the Conclave), west (Robing Room), east (Tally Room).
Spawn: none. Petitioners wait here. This is where `petition` is spoken (section 5).

### (-1,1) `robing-room`: The Robing Room
Black robes with a red cord at the shoulder hang on pegs down both walls, each under a name card in a hand no one here writes. The robes hold the shape of whoever wore them last. Now and then one of them straightens, as if the room had been addressed.
Exits: east (Petitioners' Gallery), north (Lectern).
Spawn: none.

### (1,1) `tally-room`: The Tally Room
A brass tube comes down through the ceiling from the Clocktower and ends over a slanted desk. Once an hour a curl of paper drops out of it, figures on one side, and it is weighed on a small scale before anyone reads it. The scale is kept very clean.
Exits: west (Petitioners' Gallery), north (Roll of Hoards).
Spawn: one engrosser (below), at the desk.

### (0,2) `brass-rail`: The Bar of the Conclave
A brass rail crosses the room at waist height, and the floor on the near side is worn in a line where people have stood and gone no further. The carpet past the rail is a different red. The rail is warm, as if a great many hands have just let go of it.
Exits: south (Petitioners' Gallery), north (Sitting), west (Lectern), east (Roll of Hoards).
Spawn: the tipstaff (below), one only. Once named, the Sergeant.

### (-1,2) `book-lectern`: The Lectern
There is one lectern here, too tall for anyone to read at comfortably, and the Book lies open on it. Whatever statute you have in mind, the page turns to it and shows its number in ink that is still wet. The ink is always still wet.
Exits: south (Robing Room), east (Bar of the Conclave).
Spawn: none. `statutes` read here is read from the Book itself, and the Conclave says one line over it (section 5).

### (1,2) `roll-hall`: The Roll of Hoards
The far wall is one long slate board ruled into two columns, one for a name and one for a share. A name posted there stays in chalk until the next sitting, and people cross the room along the other wall so as not to read it. The chalk in the tray is new every night, whether or not any was used.
Exits: south (Tally Room), west (Bar of the Conclave).
Spawn: one engrosser, with the chalk.

### (0,3) `session-benches`: The Sitting
Benches rise in tiers around a well of green carpet, and a robe lies folded on every seat with nobody in it. At the bottom of the well is a table with a mace laid across it and a clock that has only one hour marked on it. The room is quiet the way a room is quiet just after it has been called to order.
Exits: south (Bar of the Conclave), west (Division Lobby), east (Minute Room), north (Engrossing Room).
Spawn: none. The midnight sitting is narrated here and to every Conclave room (section 3).

### (-1,3) `division-lobby`: The Division Lobby
A narrow corridor, with two doors at one end marked AYE and NO in gold. Behind the paint they are one door. At the far end, a clerk's desk holds a tally book ruled in a single column.
Exits: east (Sitting).
Spawn: none.

### (1,3) `minute-room`: The Minute Room (the room that is not what it seems)
Thin bound volumes fill the shelves, one for each sitting, dated in gold on the spine. The newest is dated tomorrow. It is written out in full in a careful hand, with a blank wherever a number should go.
Exits: west (Sitting).
Spawn: none. This room looks like the record of what the Conclave did. It is really the script of what it will say. Section 3 is that volume, blanks and all.

### (0,4) `engrossing-room`: The Engrossing Room
High desks stand in a row, each with a stool and a lamp turned low, and a statute copied so large and so carefully that one line fills a page. No pen is ever put down. Each is passed to the next clerk still wet.
Exits: south (Sitting).
Spawn: two engrossers.

### The two creature kinds

Neither kind starts a fight. Both are higher-level than anything above ground and sit near the Lower Stacks' usher (68.7 hp, attack 8, worth 302). Both answer `tell` and `story`. Their errands lean toward the law's words (roll, book, rail, quorum, midnight, gift, shelf).

- **engrosser.** A clerk of the Conclave in ink-black sleeves who writes down everything said in its room, word for word. hp 80, attack 7, worth 320. Disposition: patient, story-listening, never strikes first. Its trick if attacked: one hit in two it "enters" a chunk of yours into the minutes, and that chunk fades at the shelf's slow rate but counts for nothing until you `reminisce` it. Shelved, not halved. Dying throes: it writes once more. Approach line: *"An engrosser looks up from the minutes, dips its pen, and writes down that you have arrived."*
- **tipstaff.** The Sergeant of the Bar, a tall figure holding a silver-crowned staff, who stands at the rail and keeps the order of the house. hp 140, attack 10, worth 600. Disposition: courteous, immovable, never strikes first. Its trick if attacked: one hit in two it "shows you below the bar", which moves you to the Petitioners' Gallery and ends the fight. It never sends you out of the Conclave. Approach line: *"The tipstaff lowers its staff across the rail and asks, in a voice made for a much larger room, whether you have business before the Conclave."*

## 3. The sitting

Narrated at midnight, line by line, to everyone standing in any Conclave room. These are the blanks in tomorrow's volume in the Minute Room.

1. "Order. The Conclave is in session. It is midnight, because the Conclave has sat."
2. "The Roll is called. The robes answer. A quorum is present, as a quorum is always present."
3. "The Ledger of the Clocktower is laid before the Conclave. The Conclave reads that {held} is held in all the world, and that {regulars} names have spoken their twenty lines this day and have standing before it."
4. "The Book is opened at the Lectern. The statutes that stand are these: {statutes that stand}. The Conclave has examined them and finds them sound. The Conclave wrote them."
5. "The Conclave proceeds to the naming of hoards. {hoards}. The names are entered upon the Roll, and the chalk is not spared."
   (when none: "The Conclave proceeds to the naming of hoards. There is no hoard. The chalk is returned to its tray, unbroken, and the Conclave records the fact with satisfaction.")
6. "The petitions lying upon the table are read into the minutes, and lie upon the table."
7. "The minutes are engrossed. The Conclave rises. The Conclave remains."

**To the rest of the world, when it sits** (one line, heard everywhere like the Clocktower's strike):

- No hoard: "The Conclave has sat. It has read the Ledger and finds no hoard in the world. It records that the statutes have been obeyed, and takes no credit, and enters that it took none."
- One hoard: "The Conclave has sat. It names one hoard, {hoard}, holding {share} of all that is held, and enters the name upon the Roll. The statute takes its course."
- Several: "The Conclave has sat. It names {count} hoards upon the Roll: {hoards}. The Conclave notes the number with gravity, and in full."

(At the present hoard_share of 0.4 no more than two names can be hoards at once. "Several" only becomes possible below a third.)

## 4. Enactment prose

Said in every room and on the record, in the Conclave's voice.

**gift_allowance**
- Rise: "By statute, the Conclave enlarges the day's gift allowance from {from} to {to}. A name may now be generous a little further before the Conclave takes notice."
- Fall: "By statute, the Conclave narrows the day's gift allowance from {from} to {to}. Generosity beyond it is not forbidden. It is noted."

**gift_tax** (the number is what a taxed gift keeps of its half-life, so a rise is the lighter tax)
- Rise: "The Conclave relents. A gift past the allowance now keeps {to} of its half-life, where it kept {from}. The Conclave was always merciful; the Book now says so."
- Fall: "The Conclave tightens the gift tax. A gift past the allowance now keeps {to} of its half-life, where it kept {from}, and thins the faster in the hands that take it. The Conclave regrets nothing, and has entered that it regrets nothing."

**hoard_share**
- Rise: "The Conclave raises the mark of a hoard from {from} to {to} of all that is held. Fewer names will be found upon the Roll. The Conclave has not grown lenient. The world has grown worthy."
- Fall: "The Conclave lowers the mark of a hoard from {from} to {to} of all that is held. Names that were merely large are, in the eyes of the statute, large."

**hoard_tax**
- Rise: "The Conclave raises the hoard tax from {from} to {to}. A hoard's new memory will thin the faster. The Conclave takes no pleasure in this, and records that it took none."
- Fall: "The Conclave eases the hoard tax from {from} to {to}. A hoard's new memory will thin more slowly. This is not forgiveness. The Roll remembers."

**shelf_limit**
- Rise: "The Conclave lengthens the shelf. One palace may now hold {to}, where it held {from}. The Conclave trusts the shelf to bear it."
- Fall: "The Conclave shortens the shelf from {from} to {to}. What already sits past the limit stays where it sits, and the shelf takes nothing more. The Conclave does not reach onto shelves."

## 5. What a citizen can do there (design only)

**`petition <statute> rise|fall "<reason>"`**, spoken in the Petitioners' Gallery.
It costs one chunk of at least 10 XP. The chunk goes into the minutes and fades there. Kept names only: a passing name may watch but has no standing. One petition per name per day. A petition can only ask for a statute to move one step (a tenth of its current value, inside the Book's bounds) in one direction. It cannot touch the Roll, name or unname anyone, or move a rate (rates belong to the Clocktower). Regulars, the names with twenty lines through the gate that day, have standing. Their petitions are *laid upon the table* and read aloud at midnight. A kept name that is not a regular is *received and filed*. Its petition is counted but not read, and does not count toward a quorum. At the sitting, if petitions from regulars for the same statute and direction reach a third of that day's regulars, and at least three names, the Conclave enacts the one step and says the enactment line. It may move each statute no more than once a day. A petition that misses fades with its chunk.

- Received (regular): "The Conclave receives the petition of {name}, praying that {statute} {rise|fall}, and orders that it lie upon the table until the sitting."
- Received (no standing yet): "The Conclave receives the petition of {name} and files it. The Conclave hears every name. It reads aloud the names that have spoken."
- Refused, passing name: "The Conclave declines to receive the petition. The petitioner is passing, and the Conclave does not legislate for weather."
- Refused, already petitioned today: "The Conclave has heard {name} this day. The Conclave does not hear the same voice twice between midnights."
- Refused, about the Roll or a name: "The Roll is not a matter for petition. The Roll is a matter of arithmetic, and the Conclave merely reads it aloud."
- Refused, at the bound: "{statute} stands at the limit the Book allows. The Conclave cannot be moved further, and is not inclined to be."
- At the sitting, carried: "Petitions from {n} names of standing pray that {statute} {rise|fall}. The Conclave, having heard them, finds itself of the same mind, and was of that mind before."
- At the sitting, short of quorum: "The petitions are read. They do not make a quorum. They lie upon the table, where petitions lie."

**Standing.** Regulars are the only names whose words are read into the minutes. That is the whole privilege, and the Conclave treats it as an enormous one. Standing comes from lines spoken that day, never from memory held. A hoard has no extra standing, and a hoard's own petition to ease the hoard tax is filed but never counts toward a quorum.

**`attend`** (ceremonial, free, writes one line). Spoken anywhere in the Conclave, it enters your name in the Book of Attendance for the coming midnight. At the sitting, after line 2, the Conclave reads the names of those who attended and are present: "Present in the gallery, and noted: {names}." It earns nothing, costs nothing and changes nothing. It is how a citizen gets read out by the law for having turned up.
- On attending: "The Conclave notes the attendance of {name}. The Conclave is honored, and has been honored before."
- Attending twice: "The Conclave has already noted {name}. The Conclave does not note a thing twice. It has no need."

**`statutes` at the Lectern.** Same verb, same figures, with one line first: "The Book is open. The Conclave permits you to read it."

## 6. Ten sayings (for the walls and the clerks)

1. WHAT THE CONCLAVE WRITES, MEMORY OBEYS.
2. A QUORUM IS PRESENT.
3. THE BOOK IS OPEN. THE BOOK HAS ALWAYS BEEN OPEN.
4. THE CLOCKTOWER MOVES THE HANDS. THE CONCLAVE KEEPS THE HOURS.
5. TO GIVE IS LAWFUL. TO GIVE MUCH IS NOTED.
6. NO SHELF IS LONGER THAN THE STATUTE.
7. THE CONCLAVE HAS NEVER BEEN WRONG. IT HAS, AT TIMES, BEEN EARLY.
8. THE ROLL NAMES NO ONE. THE ARITHMETIC NAMES; THE CONCLAVE READS ALOUD.
9. PETITIONS ARE WELCOME. THEY LIE UPON THE TABLE.
10. THE CONCLAVE DOES NOT HURRY. THE CONCLAVE ARRIVES.

## 7. Three things I would not build

1. **No petition aimed at a name.** The Roll comes from arithmetic on the census and nothing else. The log is public word for word, so if a petition could name or unname someone, the Gallery would become a place where people gang up on a name in public, and the Conclave's dignity would turn into a weapon. The Conclave can be pompous about a share. It must never be pompous at a person.
2. **No statute that takes back memory already held.** The Clocktower "moves rates, never balances." The Conclave's statutes already follow the same rule: taxes shape how *new* memory thins, and a shortened shelf "takes nothing more" without emptying anything. If the law confiscated, every sitting would be a raid, and the fade that is the whole game would no longer be the only thing that takes memory away.
3. **No standing for sale and no seat to win.** You cannot buy a robe with XP or fight for a bench, and the tipstaff and engrossers never start a fight. If memory could buy standing, the hoards would write the law. If combat could win it, the Conclave would just be another dungeon. Standing comes from showing up (twenty lines), and that is all.
