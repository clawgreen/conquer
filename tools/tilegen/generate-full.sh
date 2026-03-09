#!/bin/bash
# Full tileset generation runner — reads from tile-list-full.json
# Skips tiles that already exist in the output directory
# Usage: ./generate-full.sh [style_id] [phase] [--dry-run]
#
# Examples:
#   ./generate-full.sh pixel32          # Generate all missing tiles
#   ./generate-full.sh pixel32 4        # Generate only phase 4
#   ./generate-full.sh pixel32 all --dry-run  # Show what would be generated

set -euo pipefail

STYLE=${1:-pixel32}
PHASE=${2:-all}
DRY_RUN=false
[[ "${3:-}" == "--dry-run" ]] && DRY_RUN=true

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GEN_SCRIPT="$HOME/.nvm/versions/node/v22.22.0/lib/node_modules/openclaw/skills/openai-image-gen/scripts/gen.py"
TILE_LIST="$SCRIPT_DIR/tile-list-full.json"
CONFIG="$SCRIPT_DIR/config.json"

# Output directories
SOURCE_DIR="$SCRIPT_DIR/runs/pixel32_full"
FINAL_DIR="$HOME/GitHub/conquer/web/public/tilesets/pixel32"
mkdir -p "$SOURCE_DIR" "$FINAL_DIR"

# Load style master prompt from config
MASTER=$(python3 -c "
import json
with open('$CONFIG') as f:
    cfg = json.load(f)
for s in cfg['styles']:
    if s['id'] == '$STYLE':
        print(s['prompt'])
        break
")

if [ -z "$MASTER" ]; then
  echo "ERROR: Style '$STYLE' not found in config"
  exit 1
fi

echo "═══════════════════════════════════════════════════════"
echo "  Tileset Generation: $STYLE"
echo "  Phase: $PHASE"
echo "  Master prompt: ${MASTER:0:80}..."
echo "  Source: $SOURCE_DIR"
echo "  Final:  $FINAL_DIR"
echo "═══════════════════════════════════════════════════════"
echo ""

# Get tiles to generate from the manifest
TILES_JSON=$(python3 -c "
import json, os, sys

with open('$TILE_LIST') as f:
    data = json.load(f)

tiles = [t for t in data['tiles'] if 'tile_id' in t]
phase_filter = '$PHASE'
final_dir = '$FINAL_DIR'

to_generate = []
skipped = 0
for t in tiles:
    if phase_filter != 'all' and str(t['phase']) != phase_filter:
        continue
    
    # Convert tile_id to filename: elevation.water -> elevation_water.png
    filename = t['tile_id'].replace('.', '_') + '.png'
    
    # Skip if already exists in final dir
    if os.path.exists(os.path.join(final_dir, filename)):
        skipped += 1
        continue
    
    to_generate.append({
        'tile_id': t['tile_id'],
        'phase': t['phase'],
        'filename': filename,
        'prompt': t['prompt_fragment']
    })

print(json.dumps({'tiles': to_generate, 'skipped': skipped}))
")

TOTAL=$(echo "$TILES_JSON" | python3 -c "import json,sys; d=json.load(sys.stdin); print(len(d['tiles']))")
SKIPPED=$(echo "$TILES_JSON" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['skipped'])")

echo "Tiles to generate: $TOTAL"
echo "Already exist (skipped): $SKIPPED"
echo ""

if [ "$TOTAL" -eq 0 ]; then
  echo "Nothing to generate — all tiles exist!"
  exit 0
fi

if $DRY_RUN; then
  echo "=== DRY RUN — would generate: ==="
  echo "$TILES_JSON" | python3 -c "
import json, sys
d = json.load(sys.stdin)
for t in d['tiles']:
    print(f\"  Phase {t['phase']}: {t['filename']:40s} {t['prompt'][:60]}...\")
"
  exit 0
fi

# Generate tiles one at a time
GENERATED=0
FAILED=0
START_TIME=$(date +%s)

echo "$TILES_JSON" | python3 -c "
import json, sys
d = json.load(sys.stdin)
for t in d['tiles']:
    print(f\"{t['tile_id']}|||{t['filename']}|||{t['prompt']}\")
" | while IFS='|||' read -r TILE_ID FILENAME PROMPT; do
  GENERATED=$((GENERATED + 1))
  ELAPSED=$(($(date +%s) - START_TIME))
  
  echo "[$GENERATED/$TOTAL] ($ELAPSED s) Generating: $TILE_ID"
  
  # Generate with OpenAI
  FULL_PROMPT="$MASTER $PROMPT"
  
  python3 "$GEN_SCRIPT" \
    --prompt "$FULL_PROMPT" \
    --count 1 \
    --model gpt-image-1 \
    --size 1024x1024 \
    --quality high \
    --out-dir "$SOURCE_DIR" \
    --output-format png 2>/dev/null
  
  # Find the latest generated file and rename it
  LATEST=$(ls -t "$SOURCE_DIR"/*.png 2>/dev/null | head -1)
  if [ -n "$LATEST" ] && [ -f "$LATEST" ]; then
    # Move to correct name in source dir
    mv "$LATEST" "$SOURCE_DIR/$FILENAME"
    
    # Downscale to 32x32 and copy to final dir
    python3 -c "
from PIL import Image
img = Image.open('$SOURCE_DIR/$FILENAME')
img = img.resize((32, 32), Image.NEAREST)
img.save('$FINAL_DIR/$FILENAME')
print('  → $FILENAME ✓ (1024→32px)')
"
  else
    echo "  → FAILED: $TILE_ID"
    FAILED=$((FAILED + 1))
  fi
  
  # Brief pause to avoid rate limiting
  sleep 1
done

TOTAL_TIME=$(($(date +%s) - START_TIME))
echo ""
echo "═══════════════════════════════════════════════════════"
echo "  Generation complete in ${TOTAL_TIME}s"
echo "  Source (1024px): $SOURCE_DIR"
echo "  Final (32px):    $FINAL_DIR"
echo "  Total: $(ls "$FINAL_DIR"/*.png 2>/dev/null | wc -l) tiles"
echo "═══════════════════════════════════════════════════════"
