# billy-web

The browser shell. Rust owns the deterministic simulation; JavaScript owns the
Canvas 2D renderer, keyboard/touch input, file import and language switch.

## Build

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.100   # match Cargo.lock
./build.sh
python3 -m http.server 8080 --directory dist
```

Open <http://localhost:8080/?lang=en> (or `da` / `eo`).

## Assets

The shell runs with procedural placeholder art. To use extracted original art,
drop native-size PNGs into `assets/` before building:

```
assets/
  billy.png
  sprites/<token>.png       # crate.png, portal.png, xtratid.png, …
  jungle/platform-a.png     # 800×32 band, odd rows
  jungle/platform-b.png     # 800×32 band, even rows
  jungle/ladder.png         # 48×96 ladder strip
  construction/… cave/… water/… egypt/…
```

The platform bands are windowed per 16-pixel column, matching the level
editor's renderer. Images are never resampled; the canvas uses
`image-rendering: pixelated`.

## Input

| Action | Keys |
|---|---|
| Move | arrows / WASD |
| Dig (and fill a Full Hole) | Space |
| Fill a Partial Hole explicitly | Shift |
| Deploy Magic Ladder up / down | Alt/M + ↑ / ↓ |
| Restart | R |

Touch controls appear on narrow screens.
