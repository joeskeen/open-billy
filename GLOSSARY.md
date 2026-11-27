# Open Billy

A clean-room reimplementation of *The Worlds of Billy*. This glossary fixes the
game-domain vocabulary used by the core, the shells, and the design notes.

## Grid

**Cell**:
The 16×96-pixel grid square. A **Level** is 50 cells wide and 6 cells tall.
_Avoid_: tile, block

**Bottom Boundary**:
The lowest **Row** of the grid. It is never dug or filled, so a **Hole** can
never open the bottom of the level.
_Avoid_: floor row, last row

## Run and progression

**Run**:
A continuous play session from the main menu until **Game Over**. A **Run**
carries **Score**, **Hearts**, and **Inventory** across **Worlds**.
_Avoid_: game, session, campaign

**World**:
One themed level set: Jungle, Construction, Cave, Water, or Egypt. A **World**
has its own tokens, art, physics, and difficulty variants.
_Avoid_: theme, episode

**Monster Roster**:
The set of **Monster** tokens belonging to one **World**. Monsters do not
cross **World** boundaries.

| World | Regular monsters | End-of-level threat |
| --- | --- | --- |
| Jungle | Banana, Pumpkin, Pineapple, Coconut, Stump | Bee (`bi`) |
| Construction | Beacon, Spraycan, Saw, Hammer, Hardhat | Wrecking Ball (`crusher`) |
| Cave | Dynamite, Lamp, Pick, Ghost, Rock | Darkness (`fade`, area effect) |
| Lagoon | Conch, Seashell, Starfish, Chest, Tire | Shark/Piranhas (`fishes`) |
| Egypt | Urn, Coffin, Torch, Teabag, Pyramid | Ground fire (`ulmeild`) |

_Avoid_: bestiary

**End-of-Level Threat**:
A special **Monster** or hazard that normally appears only in **Overtime** at
the end of a **Level**, separate from the five regular monsters in its
**Monster Roster**. On a **Final Level** it is active from the start.
_Avoid_: boss

**Final Level**:
The last **Level** of a **World**. Its **End-of-Level Threat** is active from
the start; Billy must survive until the **Escape Door** appears and reach it.
The delay varies by level.
_Avoid_: boss level

**Escape Door**:
The **Portal** on a **Final Level**. Its `tid` appears to set when it appears;
the delay varies by level. Reaching it completes the **Level**.
_Avoid_: door, exit

**Level**:
One 50×6 playfield loaded from a `.K` file with exactly one `bane` descriptor.
_Avoid_: stage, map, board

**Level Chain**:
The `nestefil` progression from one **Level** to the next. A **Portal** can
warp outside the chain.
_Avoid_: campaign, sequence

**World Clear**:
Completing the final **Level** of a **World**. The **Inventory**, **Score**, and
**Hearts** at that moment become the starting state of the next **World**.
_Avoid_: campaign end

**Last Monster**:
The final surviving **Monster** in a **Level**. Defeating it starts the
**Victory Grace Period**.
_Avoid_: final enemy

**Victory Grace Period**:
The roughly 10-second window with no **Monsters** before the next **Level**
loads. It normally begins when the **Last Monster** is defeated, but other
conditions (a **Crate Monster**, the level-time gate) can change it. Billy can
still move, collect **Loot**, or die during it; **Residual Hazards** can still
kill him.
_Avoid_: victory delay, outro

**Crate Monster**:
A **Monster** released by opening a **Crate**. It counts as a Monster for the
**Last Monster** condition; if released during the **Victory Grace Period**, it
resumes the Level until defeated.
_Avoid_: crate enemy

**Residual Hazard**:
A hazard that remains after the **End-of-Level Threat** leaves, such as the
fire left by the Egypt Torch or a **Fire Column** after Billy moves. Touching
it can still kill Billy during the **Victory Grace Period**.
_Avoid_: leftover hazard

**Game Over**:
The end of a **Run** when a death takes Billy below zero **Hearts**; the game
proceeds to the **High Score** table.
_Avoid_: death, loss

**Darkness**:
The screen-wide fade to black that kills Billy if it completes. It has no
position and cannot collide. It appears as the Cave **End-of-Level Threat**
(token `fade`) and as the global time-out overlay.
_Avoid_: blackout, fade

**Overtime**:
The period after the **Level** timer expires and **Darkness** begins. The
**World**’s **End-of-Level Threat** becomes active; Billy must survive, exit,
or collect an **Alarm Clock** before full black. On a **Final Level**, the
threat is active from the start instead.
_Avoid_: extra time

**Monster Start Delay**:
The global delay after Billy’s first input before **Monsters** begin to move.
The duration is still under measurement; observation puts it at at least 1–2
seconds.
_Avoid_: monster delay, spawn delay

**Fire Warning**:
The small blue flame that appears in Billy’s cell during Egypt’s time-out
threat. If Billy does not leave, it becomes a **Fire Column**.
_Avoid_: blue flame, warning flame

**Fire Column**:
The lethal yellow/orange flame left in a **Fire Warning** cell. It kills Billy
on contact and remains for several seconds.
_Avoid_: flame, fire

**Ground Fire**:
The Egypt **End-of-Level Threat** (token `ulmeild`). It produces **Fire
Warnings** under Billy and then lethal **Fire Columns**.
_Avoid_: fire under feet

**Firehead**:
An Egypt **Monster** that appears at the edge of some **Levels**, slides in,
blows fire, and slides back out in a continuous loop.
_Avoid_: fire head

**Alarm Clock**:
A pickup (token `xtratid`) that restores the **Level** timer and interrupts
**Darkness**.
_Avoid_: clock, timer pickup

**High Score**:
A persisted score record for a **World** and difficulty.
_Avoid_: leaderboard

## Player state

**Billy**:
The player character.
_Avoid_: hero, avatar

**Heart**:
One life. The HUD shows up to ten hearts; above ten it shows a compact
`❤️xN` form. The internal count is not capped at ten, and zero hearts is still
playable; a death that takes the count below zero ends the **Run**.
_Avoid_: life, HP, health

**Score**:
The accumulated total of **Points**. **Score** survives death and carries
across **Worlds**.
_Avoid_: points

**Point**:
A single scoring award from a pickup, monster, or level bonus.
_Avoid_: score

**Score 1UP**:
A bonus **Heart** awarded at the fixed **Score** thresholds 20,000; 50,000;
100,000; 500,000; and 1,000,000. At most five are awarded per **Run**.
_Avoid_: extra life threshold

**Extra Life**:
A **Heart** pickup found in a level or loot. It is repeatable and has no
five-award cap.
_Avoid_: 1UP

**Inventory**:
The tools and items Billy currently carries. It is shown at the top of the
screen and contains **Permanents**, temporary upgrades, and consumables.
_Avoid_: bag, backpack

**Permanent**:
An item granted by the **Arcane Book** that survives death and **World**
transitions. The fixed order is **Ring**, **Tennis Shoes**, **Power Glove**,
**Jackhammer**, **Tennis Racket**.
_Avoid_: upgrade, unlock

**Temporary**:
An item or upgrade that is lost on death. Consumables are never kept after a
respawn.
_Avoid_: buff

**Death Reset**:
The sequence after monster contact: the death animation, the level reload,
one **Heart** lost, and the level paused at **Get Ready**. If the lost Heart
takes Billy below zero, the **Run** ends. **Score** and **Permanents** are
kept; level state and **Temporaries** are reset.
_Avoid_: respawn, restart

**Get Ready**:
The paused state at the start of every **Level**, regardless of how it was
reached. The timer and monster movement wait for Billy's first input.
_Avoid_: ready screen, countdown

## Tools and actions

**Pickaxe**:
Billy's permanent default digging tool; it cannot be lost.
_Avoid_: shovel

**Hammer**:
Billy's permanent default filling tool; it cannot be lost and is not shown in
the **Inventory**.
_Avoid_: mallet

**Bomb**:
A consumable dig tool. A Bomb pickup is a **Pause Pickup** granting five Bombs;
while Billy has at least one, it replaces the Pickaxe/Jackhammer in the
**Inventory**. One charge is spent per **Dig** and instantly completes the
target hole to Fully Dug, regardless of its starting **Dig State**. A Bomb
cannot be used on a **Hole** containing a **Monster**. Bombs only dig; they do
not damage Monsters.
_Avoid_: explosive, throwable bomb

**Jackhammer**:
A continuous drill that reaches the next **Dig State** in half the time of a
pickaxe **Swing**; it is time-equivalent to three pickaxe Swings for a full
hole.
_Avoid_: power drill, drill

**Power Glove**:
An upgrade that makes one hammer **Pound** advance one **Dig State**, so three
Pounds fill a hole. It does not replace the **Hammer**.
_Avoid_: glove

**Magic Ladder**:
A deployable temporary ladder carried as charges (token `tryllestav`, whose
Danish name means “magic staff” but whose sprite is the ladder item). Deploying
it creates a temporary ladder above or below Billy so he can reach an otherwise
inaccessible cell.
_Avoid_: staff, wizard staff

**Arcane Book**:
A pickup that grants the next **Permanent** in the fixed sequence; extra books
have no effect.
_Avoid_: magic book, book

**Dig Target**:
The **Hole Center** two cells ahead of Billy in his facing direction. Billy
stands next to the near **Hole Edge**; the target is not under his feet.
_Avoid_: underfoot, adjacent cell

**Dig**:
The held work action that advances the **Dig State** of the **Dig Target**.
A **Hole** containing a **Monster** cannot be dug; only **Fill** is allowed.
The work key is **Space**.
_Avoid_: mine, shovel

**Fill**:
The held work action that reverses the **Dig State** of the **Dig Target**.
It is the only action allowed on a **Monster-Filled Hole** until **Self-fill**
starts. **Space** fills a **Full Hole** (which cannot be dug deeper); **Shift**
explicitly fills a **Partial Hole** instead of digging it deeper.
_Avoid_: close, repair

**Work State**:
The locked player state while **Digging** or **Filling**. Movement input is
ignored until the key is released.
_Avoid_: action state

**Dig State**:
One of four states of a floor tile: Normal, Partly Dug, Mostly Dug, and Fully
Dug. Digging advances one state; filling reverses one state. Completed states
persist indefinitely; swing progress inside a state is not remembered. The
first digging transition releases **Buried Loot**.
_Avoid_: depth stage, hole level

**Swing**:
One pickaxe animation. Two Swings advance one **Dig State**; six Swings take a
tile from Normal to Fully Dug. A **Jackhammer** is time-equivalent to three
Swings but is a continuous drill rather than a swing.
_Avoid_: hit, strike

**Pound**:
One hammer animation. Two Pounds advance one **Dig State**; six Pounds fully
fill a tile. A **Power Glove** makes one Pound advance one state (three
Pounds total).
_Avoid_: hammer swing, hit

**Hole**:
A floor tile that is not in the Normal **Dig State**. A Hole occupies a
three-cell **Hole Span**.
_Avoid_: pit, gap

**Hole Span**:
The three horizontal cells occupied by a **Hole**: one **Hole Center** and one
**Hole Edge** on each side. Holes cannot overlap. A falling Billy passes
through the span if his **Landing Check** column is the center or either edge.
_Avoid_: hole width

**Hole Center**:
The dug tile of a **Hole**; a **Monster** falls into this cell.
_Avoid_: center cell

**Hole Edge**:
One of the two cells beside a **Hole Center**. The platform texture is masked
there so the hole reads as three cells wide. The near **Hole Edge** is the cell
adjacent to Billy when he **Digs**.
_Avoid_: lip

**Hole Island**:
The one-cell platform left between two **Holes** dug on either side of Billy.
A falling Billy can land on it only if his **Landing Check** column is exactly
the island cell.
_Avoid_: pillar

**Hole Spacing**:
Four cells between **Hole Centers**. Digging on both sides of Billy leaves a
one-cell **Hole Island**.
_Avoid_: hole gap

**Partial Hole**:
A **Hole** in the Partly Dug or Mostly Dug state. It blocks Billy's horizontal
movement across its **Hole Span**; he cannot walk over or past it. A
**Monster** can be trapped in it, and **Buried Loot** does not fall through.
_Avoid_: shallow hole

**Full Hole**:
A **Hole** in the Fully Dug state. Billy can fall through its **Hole Span**
when it is empty, loot can fall through, and a **Monster-Filled Hole** blocks
horizontal entry but can be dropped into from above.
_Avoid_: complete hole, open hole

**Monster-Filled Hole**:
A **Full Hole** containing a **Trapped Monster** that has not started
self-fill. It blocks horizontal entry from the same level, but Billy can drop
into it from above to **Push-through**. Billy can **Fill** it but cannot
**Dig** it deeper.
_Avoid_: occupied hole

**Locked Hole**:
A **Hole** containing an **Escaping Monster**. A falling Billy lands on it as
solid platform and may move left/right while on it, but the cell blocks
horizontal entry from an adjacent cell.
_Avoid_: escaping hole

**Push-through**:
Billy dropping into a **Monster-Filled Hole** from above. The monster is pushed
down one layer with no horizontal offset and continues falling if the cell
below is a **Hole Span** cell or a gap; otherwise it lands. Billy follows the
same fall rules as an empty hole and lands on the next platform; if the monster
lands on that platform, normal monster contact applies.
_Avoid_: stomp

**Monster Drop**:
A monster falling straight down after a **Push-through** or **Self-fill**. It
continues through **Hole Span** cells and gaps and lands on the first solid
cell.
_Avoid_: monster fall

**Trapped Monster**:
A **Monster** occupying a **Hole**.
_Avoid_: stuck monster

**Escaping Monster**:
A **Trapped Monster** that has started **Self-fill**. Its **Hole** is locked,
and the monster is harmless while it remains in the Hole.
_Avoid_: escaping enemy

**Self-fill**:
A **Trapped Monster**'s own filling of its **Hole**. If the Hole was partial
when the monster entered, self-fill starts immediately; a **Full Hole** grants
a **Grace Period**. Once self-fill starts, the Hole is locked: Billy cannot
Dig or Fill it.
_Avoid_: monster fill

**Ledge Drop**:
Billy leaving a platform edge or a hole above. He begins on the **Fall Start
Cell** and follows the **Fall Trajectory**; a **Landing Check** decides whether
he lands or passes through a **Hole Span**.
_Avoid_: jump

**Fall Start Cell**:
The cell just beyond a platform edge where Billy begins a **Ledge Drop**.
_Avoid_: jump-off cell

**Fall Trajectory**:
The diagonal path of a **Ledge Drop**: one cell horizontally per row fallen.
_Avoid_: arc

**Landing Check**:
At each platform row, the cell at the **Fall Trajectory** column. If it is a
**Hole Center** or **Hole Edge**, the drop continues; if it is solid, Billy
lands.
_Avoid_: collision check

**Landing Cell**:
The solid cell where a **Ledge Drop** ends.
_Avoid_: impact cell

**Grace Period**:
The delay before a **Trapped Monster** in a **Full Hole** begins **Self-fill**.
It is broadly consistent across most monsters, with per-monster exceptions
(the spray can is nearly immediate), and is shorter for an **Annoyed** monster.
_Avoid_: escape delay

**Annoyed**:
A **Monster** that has escaped a **Hole** at least once. It has a shorter
**Grace Period** on subsequent traps.
_Avoid_: angry

**Escape Pause**:
The brief pause after a **Trapped Monster** finishes **Self-fill** and leaves
the **Hole**, before it resumes its AI.
_Avoid_: stun

**Buried Loot**:
Loot stored in a tile's `hulbonus` content. One **Indhold Alternative** is
rolled at load and released as a **Loot Burst** when that tile is first dug.
_Avoid_: treasure

## Objects and level features

**Token**:
A named object type in a `.K` file, such as `banana`, `portal`, or `crate`.
_Avoid_: entity type, object type

**Monster**:
A hostile actor in a **Level**. A Monster may be a type-3 class (bee, fish,
crusher, firehead, fade, ulmeild) or a placed type-0 token such as the
Construction beacon, saw, spraycan, hammer, or cap. The numeric token type is
not the behavior.
_Avoid_: enemy, creature

**Loot Item**:
An inanimate token that grants **Points** and does not attack. It may be
placed in a **Level** or spawned as **Loot** from a **Crate** or **Buried
Loot**.
_Avoid_: prop, pickup

**Loot**:
Items spawned from a **Crate** or **Buried Loot**. Loot is harmless to Billy
and is collected as a **Loot Item**.
_Avoid_: drop

**Anchor**:
A single character in the object layer that marks an object's position.
_Avoid_: marker, node

**Attribute**:
A `key=value` pair on a token line, such as `pos=a` or `tid=30`.
_Avoid_: property, field

**Program**:
The mandatory `program` attribute on a placed type-0 object; a cyclic string of
direction letters. `h` right, `v` left, `o` up, `n` down, `m` greedy toward
Billy, `r` random, `.` no-op. The string chooses patrol, chase, or random
movement per object; there is no pathfinding.
_Avoid_: script

**Patrol**:
A movement pattern produced by a **Program** containing only fixed direction
letters (`h`, `v`, `o`, `n`).
_Avoid_: route

**Chase**:
The `m` **Program** letter: one greedy per-axis step toward Billy’s tile,
filtered by local passability. It is not pathfinding.
_Avoid_: pathfinding, seek

**Edge Wrap**:
The Cave-only `thrusign` feature: walking off one horizontal edge of the **Level**
continues at the opposite edge.
_Avoid_: thrusign, wrap sign

**Portal**:
A **Pause Pickup** that warps to the level named by `banefil` after Billy
stands on it briefly.
_Avoid_: door, warp

**Wrecking Ball**:
The Construction end-of-level **Monster** (token `crusher`); it descends on a
chain at the end of the level.
_Avoid_: crusher

**Crate**:
A **Pause Pickup** that rolls one `indhold` alternative at load and bursts its
contents out in random directions when collected. The contents may be
**Loot** or a **Crate Monster**.
_Avoid_: mystery box, chest

**Indhold Alternative**:
One `|`-separated option in an `indhold` value. Exactly one option is rolled
when the level loads; a multi-character option releases every **Loot Group**
in it.
_Avoid_: loot table entry, A/B option

**Loot Group**:
A single character in an **Indhold Alternative** that selects what is spawned.
`a`–`e` are world-themed props; `f`–`z` are direct items; `u`/`v`/`w` are point
bursts.
_Avoid_: bonus letter, code

**Loot Burst**:
The visual and physical scatter of spawned **Loot** away from the Crate or
dug cell in random directions.
_Avoid_: explosion

**Instant Pickup**:
An item collected immediately when Billy walks over its cell. Points-only
loot, tennis balls, Power Glove, Ring, Rainbow, magic ladders, and Jackhammer
are instant.
_Avoid_: auto pickup

**Pause Pickup**:
An item collected only after Billy stands still on its cell for a brief
moment. Bombs, Crates, and Portals are Pause Pickups.
_Avoid_: manual pickup, stand pickup

**Savegame**:
The persisted per-**World** state: **Hearts**, **Score**, and **Permanent**
items. It is written on **World** transitions, not on death or ordinary level
completion.
_Avoid_: save file

## Project terms

**World File**:
A per-world TOML document pointed to by the main `open-billy.toml`. It owns
that world's **Monster Roster**, labels, colors, tiles, physics, and token
allow-list.
_Avoid_: world config

**Ruleset**:
The configurable simulation values: tick rate, swing counts, grace periods,
and point awards. Separate from the **World File**.
_Avoid_: settings
