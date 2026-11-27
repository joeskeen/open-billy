# Open Billy

An **unofficial, clean-room reimplementation** of the 1997 DOS game
*The Worlds of Billy* (Drilling Billy), built around a deterministic core that
can run in a browser, on the desktop, or headless in tests.

> This project is a fan work. It is **not** affiliated with, endorsed by, or
> supported by Northwind Soft or any current rights holder. Original game art,
> audio, levels and code are **not** included in this repository. You need a
> legal copy of the game to use the asset and level import tools.

## Why this layout

The repository is a Cargo workspace with hard boundaries. Each crate can be
built and tested on its own:

```
open-billy/
├── billy-core/      pure simulation + .K codec + runtime config types
├── billy-config/    TOML/JSON loader for open-billy.toml
├── billy-assets/    decoders for the original .DB sprite atlases / palettes
├── billy-web/       wasm-bindgen shell (Canvas 2D, keyboard/touch/gamepad)
└── billy-native/    macroquad desktop shell
```

`billy-core` is the contract. Rendering, input, audio and file dialogs live in
the shells. That keeps `cargo test -p billy-core` fast and makes the game rules
easy to change without touching a windowing framework.

Worlds are **data, not an enum**. `open-billy.toml` defines the five original
worlds (or as many as you like), their token sets, labels, colours, tile paths
and per-world timings. The shells never `match` on a world id.

## Quick start

```bash
# Level codec + simulation tests (no game files needed)
cargo test -p billy-core

# Asset decoder tests (synthetic fixture)
cargo test -p billy-assets

# Optional: test against a real install
BILLY_GAME_DIR=/path/to/BILLY cargo test -p billy-core -p billy-assets -- --ignored

# Desktop shell
cargo run -p billy-native -- /path/to/BILLY/BANER/BANE1.K

# Browser shell
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130
./billy-web/build.sh
python3 -m http.server 8080 --directory billy-web/dist

# Local-only: add art extracted from your legal game copy
BILLY_ART_DIR=../level-editor ./billy-web/preview-local.sh
```

## Configuration

`open-billy.toml` is the single source of truth for project data:

```toml
[game]
title = "Open Billy"
default_world = "jungle"
default_locale = "en"

[simulation]
tick_hz = 60
move_ticks = 4
fall_ticks = 4
dig_cooldown_ticks = 12
pause_pickup_ticks = 18
extra_time_ticks = 1800
lives = 3

[simulation.points]
crate = 50
diamond = 100

[[worlds]]
id = "cave"
labels = { en = "Cave", da = "Hulen", eo = "Kaverno" }
tokens = ["crate", "dynamite", "fade", "ghost", "portal", "thrusign"]

[worlds.colors]
sky = "#100f18"
platform = "#4a4058"
platform_edge = "#8f7fae"
ladder = "#bea5dc"
hole = "#000000b4"
player = "#3d6fb6"
hazard = "#eb6446"
collectible = "#f2c14e"
portal = "#6ee39b"
text = "#e8f0f7"

[worlds.tiles]
platform_odd = "assets/cave/platform-a.png"
platform_even = "assets/cave/platform-b.png"
ladder = "assets/cave/ladder.png"

[worlds.physics]
move_ticks = 4
fall_ticks = 5
```

Add a new `[[worlds]]` table with `id = "moon"` and a level with
`bane:verden=moon` becomes playable immediately. A world that omits
`[worlds.colors]` inherits the built-in palette for the same id; a new world
with no palette uses the default one. Token behaviour, points, default
attributes and sprite paths can be overridden with `[[tokens]]`.

Native:

```bash
cargo run -p billy-native -- --config=open-billy.toml BANER/CAVE1.K
# or OPEN_BILLY_CONFIG=open-billy.toml, or a local open-billy.toml
```

Web: `billy-web/build.sh` copies `open-billy.toml` into `dist/`; `main.js`
fetches it and passes it to the wasm constructor. Without a config file, the
embedded default is used.

**Source identifiers are English.** The original `.K` attribute keys and Danish
token names (`bane`, `verden`, `tid`, `thrusign`, …) remain unchanged because
they are file-format data, not source-code identifiers.

## Current status

This list tracks implementation. Owner-certified behaviour is tracked separately
in [`docs/checklist.md`](docs/checklist.md) — a box there is ticked only after the
owner confirms the feature is correct in the running game.

- [x] `.K` decoder/encoder (the 107-byte keystream + running delta)
- [x] Level model: 50×6 object/floor layers and `<token>:<attr>=value;` objects
- [x] Game-accurate token/world registry and validation warnings
- [x] World/token/palette/tile/timing data in `open-billy.toml`
- [x] Deterministic tick simulation: walking, ladders, held Dig/Fill work
      with four Dig States, three-cell Hole Spans, falling, timed object
      appearance, Instant/Pause pickups, portals, contact hazards,
      score/lives/time
- [x] Type-0 `program` movement (`h/v/o/n/m/r/.`) with greedy chase, no
      pathfinding; per-token scores from the original binary
- [x] Monsters trapped in Full Holes, Fill-drops, and Push-through drops
- [x] `indhold` loot alternatives (`|`), world-themed and direct loot groups,
      buried loot released on the first dig, crate loot bursts
- [x] Cave `thrusign` horizontal **Edge Wrap**
- [x] **Magic Ladder** charges (`tryllestav`), deployed with **Alt/M + ↑/↓**
- [x] Original **ALL16.DB bitmap font** for the in-game HUD, status card and
      world/level labels (local art pack; white 0/1/2 recolored from the
      0-white/1-white/2-white sprites)
- [x] Three-stage original hole art; ladders pass through platform cells;
      monsters patrol until a platform edge/ladder
- [x] Top in-game HUD: score, time, lives, bombs, magic ladders
- [x] `.DB` sprite-atlas decoder and ALL8 palette decoder
- [x] Canvas 2D web shell and macroquad native shell (placeholder art)
- [ ] Pixel-accurate movement/animation (the original is smooth sub-cell)
- [ ] Type-3 monster behaviours, weapons, elevators, bubbles, secret walls,
      crate loot release, self-fill/grace timing
- [ ] Original-art asset pack loader + audio
- [ ] Deterministic input replay / record & verify tooling

The simulation is deliberately a **clean-room approximation** at this stage.
It follows the level grammar, grid dimensions, tile flags, `tid` semantics and
the major Lode-Runner rules documented in `../ghidra/level-format.md`, but it
is not yet frame-identical to the DOS binary.

## Level format

The codec is documented in the source and in the parent research notes. In
short:

```
plain[i]  = ((cipher[i] - plain[i-1]) & 0xff) ^ key[(i+1) mod 108]
cipher[i] = ((plain[i]  ^ key[(i+1) mod 108]) + plain[i-1]) & 0xff
```

A decoded level is 6 rows × 50 columns stored as 12 lines (object layer then
floor layer), followed by one object definition per line. See
`billy-core/src/level.rs`.

## Licence

The Rust code is dual-licensed under MIT or Apache-2.0. Game data is not part
of this repository and remains the property of its rights holders.
