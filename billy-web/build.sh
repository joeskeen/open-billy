#!/usr/bin/env bash
set -euo pipefail

# Build the Open Billy wasm shell into billy-web/dist/.
# Requires:
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli --version <matching wasm-bindgen crate version>
#
# Set WASM_BINDGEN=/path/to/wasm-bindgen to use a specific binary.

cd "$(dirname "$0")"

if ! rustup target list --installed | grep -qx wasm32-unknown-unknown; then
  echo "Installing the wasm32-unknown-unknown target…"
  rustup target add wasm32-unknown-unknown
fi

WASM_BINDGEN="${WASM_BINDGEN:-wasm-bindgen}"
if ! command -v "$WASM_BINDGEN" >/dev/null 2>&1; then
  echo "wasm-bindgen-cli is not installed." >&2
  echo "Install the version that matches Cargo.lock, for example:" >&2
  echo "  cargo install wasm-bindgen-cli --version 0.2.100" >&2
  exit 1
fi

BUILD="$(date -u +%Y%m%d-%H%M%S)"
export OPEN_BILLY_BUILD="$BUILD"
cargo build -p billy-web --release --target wasm32-unknown-unknown

rm -rf dist
mkdir -p dist/pkg
"$WASM_BINDGEN" \
  --target web \
  --out-dir dist/pkg \
  --out-name billy_web \
  ../target/wasm32-unknown-unknown/release/billy_web.wasm

# Cache-bust the module, stylesheet and wasm on every build.  Python's
# http.server sends Last-Modified but no cache directive, so a normal refresh
# can otherwise keep the previous wasm alive.
sed -e "s|href=\"style.css\"|href=\"style.css?v=$BUILD\"|" \
    -e "s|src=\"main.js\"|src=\"main.js?v=$BUILD\"|" \
    index.html > dist/index.html
sed -i "s|billy_web_bg.wasm|billy_web_bg.wasm?v=$BUILD|g" dist/pkg/billy_web.js
cp main.js style.css font-map.json dist/
cp ../open-billy.toml dist/open-billy.toml
cp -R ../worlds dist/worlds
if [ -d assets ]; then
  cp -R assets dist/
else
  mkdir -p dist/assets
fi

echo
echo "Built billy-web/dist/"
echo "Serve it with:"
echo "  python3 -m http.server 8080 --directory billy-web/dist"
