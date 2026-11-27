# Monster data (from the v1.25 binary)

Read-only Ghidra findings; useful when filling in world/token config. This is
reference data, not a glossary.

## `program` — type-0 movement script

Mandatory on every placed type-0 object. Consumed cyclically, one letter per
stand-state tick. No pathfinding.

| Letter | Meaning |
| --- | --- |
| `h` | move right |
| `v` | move left |
| `o` | move up |
| `n` | move down |
| `m` | greedy step toward Billy’s tile, filtered by local passability |
| `r` | random direction (independent vertical/horizontal bits) |
| `.` | no-op (keep previous direction) |

- `m` only compares Billy’s tile to the object’s tile; it does not plan around
  walls.
- `interval`, `hast`, `forkortelse` and `tid` are **not parsed for type-0
  objects**. `program` is the only movement attribute.
- `forkortelse` appears to be dead/stored-only in v1.25.

## Type-0 scores and base speeds

| Token | Score | Base speed | Max speed |
| --- | --- | --- | --- |
| banana | 250 | 1.0 | 1.3 |
| log (stump) | 500 | 1.6 | 1.9 |
| ananas (pineapple) | 350 | 1.3 | 2.2 |
| coconut | 400 | 1.6 | 2.2 |
| pumpkin | 300 | 1.0 | 1.6 |
| cap (hardhat) | 550 | 1.3 | 2.0 |
| beacon | 250 | 1.5 | 2.5 |
| hammer | 450 | 1.3 | 1.5 |
| saw | 350 | 1.3 | 1.3 |
| spraycan | 350 | 1.3 | 1.5 |
| musling (seashell) | 450 | 1.3 | 2.0 |
| chest | 650 | 1.5 | 2.5 |
| conkylie (conch) | 350 | 1.3 | 2.0 |
| star (starfish) | 550 | 1.3 | — |
| tire | 800 | 1.5 | 3.0 |
| dynamite | 400 | 1.3 | 2.2 |
| lamp | 450 | 1.3 | 1.6 |
| pick | 500 | 1.6 | 1.9 |
| ghost | 600 | 1.6 | 2.2 |
| rock | 700 | 1.0 | 1.3 |
| torch | 600 | 1.6 | 2.2 |
| sarkofag (coffin) | 500 | 1.3 | 1.6 |
| urn | 450 | 2.2 | 3.2 |
| pyramid | 950 | 1.6 | 1.9 |
| teabag | 750 | 1.0 | 1.3 |

## Type-3 movement shapes

| Token | Movement |
| --- | --- |
| `bi` bee | vector homing toward a target point, acceleration based on `hast` |
| `fishes` | schooling/vector movement; follows a leader or Billy’s side |
| `crusher` | floor charger; horizontal velocity from `hast`, digs/destroys |
| `firehead` | stationary turret; fires when its shot counter reaches 2 |
| `fade` | timing/contact hazard; Cave Darkness area effect |
| `ulmeild` | static hazard |

## Notes

- The `program` cadence is one letter per stand-state tick; the exact
  letters/second still needs a live measurement.
- `+0x1e` and the tile-bit meanings (`0x04/0x08/0x10/0x40`) are inferred, not
  fully proven.
