# Open Questions

These are unresolved behaviours that need either more reverse-engineering or a
custom playtest level. They are deliberately **not** in `GLOSSARY.md` until
settled.

## Victory Grace Period

- **Crate Monster during the grace period**: Does the level timer/Darkness
  resume while Billy fights a monster released from a crate? Does the Victory
  Grace Period restart after the new last monster is defeated? No shipped level
  reaches this state; a custom late-crate level is needed.
- **Resolved**: the normal window is about **10 seconds with no monsters**.
  The binary gate is an empty monster list plus a level-time condition
  involving a `+10.0` constant; other conditions can stretch or interrupt it.
  The timer does not freeze.

## Monsters

- **AI/pathfinding**: resolved — `program` is a cyclic direction script
  (`h/v/o/n/m/r/.`), not pathfinding. `m` is greedy per-axis toward Billy.
  Type-3 monsters use hard-coded movement. Remaining uncertainty: the exact
  stand-state tick rate (letters per second) needs live confirmation.
- **Self-fill timing / Locked Hole**: base grace period before self-fill
  starts (user believes longer than 5 s), per-monster exceptions, and how much
  the Annoyed state shortens it. RE pass in progress. The slice currently keeps
  a trapped monster in place indefinitely; self-fill, Locked Holes and the
  Escape Pause are not implemented yet.
- **Bomb depth**: resolved — one Bomb instantly completes the hole to Fully
  Dug and cannot be used on a hole containing a monster.
- **Digging a Monster-Filled Hole**: resolved — only Fill is allowed once a
  monster is in the hole.

## Weapons and tools

- **Magic Ladder (`tryllestav`)**: resolved — it is a deployable temporary
  ladder with charges (sprite confirmed against the game art), deployed with
  **Alt + ↑/↓**. Open: exact lifetime (the slice uses a placeholder
  `magic_ladder_ticks = 180`, about 3 s) and whether one charge should create
  more than one cell.
- **Edge Wrap (`thrusign`)**: resolved — Cave-only level feature that wraps
  Billy from one horizontal edge to the opposite edge. Not a Magic Ladder.
- **Bee swarm**: the binary’s loot switch has a `'1'` branch that spawns `bi`,
  but no shipped `indhold` uses it (checked across all 181 distinct values).
  `bi` is the Jungle end-of-level threat; the loot branch is unused/debug.

## Pickups

- **Pause Pickup duration**: how many ticks must Billy stand still on a Bomb,
  Crate, or Portal? Does moving/digging reset the timer? The slice uses a
  placeholder `pause_pickup_ticks = 18` (0.3 s) and resets on any movement or
  work input.
- **Crate loot release**: resolved. `indhold` is `|`-separated alternatives;
  one is rolled at load, and every group letter in it spawns loot. The slice
  scatters items across nearby platform cells rather than using the original
  per-item ballistic velocity. Exact per-item flight/landing is still open.
- **Loot roll variation**: the core is deterministic for a given run seed.
  Shells should pass a fresh `with_config_seed` seed per playthrough if they
  want the “sometimes lots of points, sometimes a bomb” variation between
  runs.
- **Full pickup classes**: current list puts Bombs, Crates and Portals in
  Pause; points loot, tennis balls, Power Glove, Ring, Rainbow, magic ladders
  and Jackhammer are Instant. Confirm the unlisted potions, extra life/time,
  magic book, boots and racket.

## Level start

- **Global monster-start delay**: the user observes at least 1–2 seconds after
  Billy’s first input before monsters move; the static pass found no separate
  global timer. A controlled level with one stationary monster is needed to
  measure the exact delay.
- **Swing timing**: the slice uses `DEFAULT_SWING_TICKS = 30` (about 0.5 s per
  pickaxe swing, 1 s per state) from a live observation. Confirm against the
  DOS build at 60 Hz.
- **Ledge Drop drift**: the rule (one cell horizontally per row fallen) is
  recorded, but the slice currently falls straight down. Implement with the
  next movement pass.

## Final levels

- **Escape Door identity**: the final levels all use a `portal` whose `tid`
  varies (0/2/130/110/110). Confirm that the door is always this portal and not
  a different token/trigger in some worlds. Current confidence: high, not 100%.

## End-of-level threats

- **Egypt “Ground fire”**: confirmed as the `ulmeild` end-of-level threat.
  `firehead` is a separate edge-loop monster, not part of the threat.
- **Firehead loop**: exact period, how far it slides in, and whether its fire
  can be avoided by timing.
- **Lagoon “Shark/Piranhas”**: confirmed as `fishes` in `WATER20.K`.
- **Cave “Darkness”**: confirmed as the `fade` area effect, not a positional
  monster.
