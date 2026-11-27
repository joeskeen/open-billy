#!/usr/bin/env bash
# Local-only preview with art extracted from a legal game copy.
#
# The extracted PNGs are NOT part of this repository.  This script copies them
# into the generated dist/ directory so you can evaluate the slice with the
# original look; the normal build keeps using procedural placeholders.
#
#   BILLY_ART_DIR=/path/to/level-editor ./preview-local.sh
#
# The default expects the sibling level-editor checkout from this workspace.

set -euo pipefail
cd "$(dirname "$0")"

./build.sh

ART_DIR="${BILLY_ART_DIR:-../../level-editor}"
DST="dist/assets"

if [ ! -d "$ART_DIR/public/sprites" ]; then
  echo "No extracted art at $ART_DIR/public/sprites" >&2
  echo "Set BILLY_ART_DIR to the level-editor checkout." >&2
  exit 1
fi

mkdir -p "$DST/sprites"
cp "$ART_DIR"/public/sprites/*.png "$DST/sprites/"

# Billy walk/idle animation frames (local-only).
if [ -f ../../from-backup/images/common/billy/walk.sprite.png ]; then
  cp ../../from-backup/images/common/billy/walk.sprite.png "$DST/billy-walk.png"
fi
if [ -f ../../from-backup/images/common/billy/idle.png ]; then
  cp ../../from-backup/images/common/billy/idle.png "$DST/billy-idle.png"
fi

if [ -d ../../from-backup/images/common/monster-hands ]; then
  mkdir -p "$DST/monster-hands"
  cp ../../from-backup/images/common/monster-hands/*.png "$DST/monster-hands/"
fi

for SUB in climb dig drill giggle hammer ouch throw; do
  SRC_DIR="../../from-backup/images/common/billy/$SUB"
  if [ -d "$SRC_DIR" ]; then
    mkdir -p "$DST/billy/$SUB"
    cp "$SRC_DIR"/*.png "$DST/billy/$SUB/"
  fi
done

# Jungle monster animation frames.  The JS uses the first contiguous run as
# the walk cycle; all extracted frames are copied for later states.
for PAIR in \
  "banana:banana" "pumpkin:pumpkin" "pineapple:ananas" \
  "coconut:coconut" "stump:log" "bee:bi"; do
  SRC_NAME="${PAIR%%:*}"
  TOKEN="${PAIR##*:}"
  SRC_DIR="../../from-backup/images/jungle/$SRC_NAME"
  if [ -d "$SRC_DIR" ]; then
    mkdir -p "$DST/monsters/$TOKEN"
    cp "$SRC_DIR"/*.png "$DST/monsters/$TOKEN/"
  fi
done

# Extracted ALL16.DB font glyphs (local-only).
FONT_DIR="${BILLY_FONT_DIR:-../../from-backup/images/common/font}"
if [ -d "$FONT_DIR" ]; then
  mkdir -p "$DST/font"
  cp "$FONT_DIR"/*.png "$DST/font/"
  echo "Preview font copied from $FONT_DIR"
else
  echo "No extracted font at $FONT_DIR; using system text." >&2
fi

# Original three-stage hole masks (local-only).
HOLE_DIR="${BILLY_HOLE_DIR:-../../from-backup/images/common/hole}"
if [ -d "$HOLE_DIR" ]; then
  mkdir -p "$DST/hole"
  cp "$HOLE_DIR"/*.png "$DST/hole/"
  echo "Preview hole art copied from $HOLE_DIR"
fi

# World-specific top adornments (only the extracted ones).
mkdir -p "$DST/jungle" "$DST/construction"
if [ -f ../../from-backup/images/jungle/JUNGLE.DB-0133.png ]; then
  cp ../../from-backup/images/jungle/JUNGLE.DB-0133.png "$DST/jungle/adornment.png"
fi
if [ -f ../../from-backup/images/construction/CONSTR.DB-012f.png ]; then
  cp ../../from-backup/images/construction/CONSTR.DB-012f.png "$DST/construction/adornment.png"
fi

# World-specific partial-hole masks (`01bc` partly, `01bd` mostly).
for PAIR in "jungle:JUNGLE" "construction:CONSTR" "cave:CAVE" "lagoon:WATER" "egypt:EGYPT"; do
  SRC_WORLD="${PAIR%%:*}"
  PREFIX="${PAIR##*:}"
  case "$SRC_WORLD" in
    lagoon) DST_WORLD="water" ;;
    *) DST_WORLD="$SRC_WORLD" ;;
  esac
  mkdir -p "$DST/$DST_WORLD"
  if [ -f "../../from-backup/images/$SRC_WORLD/$PREFIX.DB-01bc.png" ]; then
    cp "../../from-backup/images/$SRC_WORLD/$PREFIX.DB-01bc.png" "$DST/$DST_WORLD/hole-partly.png"
  fi
  if [ -f "../../from-backup/images/$SRC_WORLD/$PREFIX.DB-01bd.png" ]; then
    cp "../../from-backup/images/$SRC_WORLD/$PREFIX.DB-01bd.png" "$DST/$DST_WORLD/hole-mostly.png"
  fi
done

for SUB in heart time magic-ladder inventory; do
  SRC_DIR="../../from-backup/images/common/$SUB"
  if [ -d "$SRC_DIR" ]; then
    mkdir -p "$DST/$SUB"
    cp "$SRC_DIR"/*.png "$DST/$SUB/"
    echo "Preview $SUB art copied from $SRC_DIR"
  fi
done

for world in jungle construction cave water egypt; do
  mkdir -p "$DST/$world"
  cp "$ART_DIR/src/app/tiles/$world-platform-a.png" "$DST/$world/platform-a.png"
  cp "$ART_DIR/src/app/tiles/$world-platform-b.png" "$DST/$world/platform-b.png"
  cp "$ART_DIR/src/app/tiles/$world-ladder.png" "$DST/$world/ladder.png"
  if [ -f "$ART_DIR/public/backgrounds/$world.png" ]; then
    cp "$ART_DIR/public/backgrounds/$world.png" "$DST/$world/background.png"
  fi
done

echo
echo "Preview art copied from $ART_DIR into $DST (local only)."
echo "Serve: python3 -m http.server 8080 --directory billy-web/dist"
