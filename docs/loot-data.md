# Loot data (`indhold`)

Reference data recovered from the v1.25 binary. This is design data, not
original art.

## Alternatives

`indhold` is a **`|`-separated list of alternatives**. Exactly one alternative
is rolled when the level loads (and again on a fresh load), so the same crate or
secret can produce different outcomes between runs.

```text
indhold=uvv|ju|www
```

- `uvv` → a lot of point loot.
- `ju` → one Bomb pickup (worth five charges) plus a small point burst.
- `www` → more point loot.

This is the “A/B format” seen on Jungle Easy 1: `LBANE1.K` has
`hulbonus:indhold=uvv|ju|www` (confirmed against the user’s memory of the
level).

A crate rolls its alternative at load; a `hulbonus` does the same and releases
the chosen alternative on the **first dig** of its cell only.

## Loot groups

Each character in the chosen alternative spawns one or more items. Items pop
out in random directions and must be collected.

### `a`–`e` — world-themed props (can be monsters)

| group | Jungle | Construction | Cave | Water | Egypt |
| --- | --- | --- | --- | --- | --- |
| `a` | banana | beacon | dynamite | conkylie | urn |
| `b` | pumpkin | spraycan | lamp | musling | sarkofag |
| `c` | ananas | saw | pick | star | torch |
| `d` | coconut | hammer | ghost | chest | teabag |
| `e` | log | cap | rock | tire | pyramid |

### `f`–`z` — direct items

| char | Spawns | Notes |
| --- | --- | --- |
| `f` | `ring` | |
| `g` | `lobesko` | speed boots |
| `h` | `krafthandske` | Power Glove |
| `i` | `tennisbold` | |
| `j` | `bombe` | pickup grants **5** Bomb charges |
| `k` | `luftbor` | Jackhammer |
| `l` | `rodpotion` | |
| `m` | `blaapotion` | |
| `n` | `gronpotion` | |
| `o` | `regnbue` | |
| `p` | `ketcher` | |
| `q` | `tryllestav` | **Magic Ladder** charges (3), not a throwing staff |
| `r` | `tordensky` | |
| `s` | `xtraliv` | extra life |
| `t` | `xtratid` | extra time |
| `u` | four 5-point bursts | |
| `v` | four 12-point bursts | |
| `w` | 14 / 8 / 12 / 14-point bursts | |
| `x` | big bonus | value still approximate |
| `y` | `tryllebog` | Arcane Book |
| `z` | `diamant` | |
| `1` | `bi` | **unused in all shipped levels**; a debug/unused branch in the binary. No bee swarm occurs in retail play. |

## Drop physics

Every spawned item receives a random pop-out velocity: horizontal roughly
−150…+150 px/s, vertical −350…−100 px/s (upward). In the cell-based slice this
is approximated by scattering items across nearby platform cells, then letting
Billy walk over to collect them.

## Determinism

The core rolls loot from a per-object seed. `Sim::with_config` is deterministic
(each replay of the same level gets the same roll). A run seed can be supplied
by a shell to vary outcomes between playthroughs while keeping replays exact.
