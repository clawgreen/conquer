#!/bin/bash
# Tileset generation runner — generates tiles using OpenAI image API
# Usage: ./generate.sh <style_id> [phase]
# Example: ./generate.sh pixel32 1

STYLE=${1:-pixel32}
PHASE=${2:-all}
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GEN_SCRIPT="$HOME/.nvm/versions/node/v22.22.0/lib/node_modules/openclaw/skills/openai-image-gen/scripts/gen.py"
RUN_DIR="$SCRIPT_DIR/runs/${STYLE}_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RUN_DIR"

echo "=== Tileset Generation: style=$STYLE phase=$PHASE ==="
echo "Output: $RUN_DIR"

# Master prompt for pixel32
MASTER="Create a 32x32 pixel art tile for a medieval fantasy strategy game viewed from top-down. Style: detailed retro pixel art, SNES/GBA era quality. Rich but controlled palette. Subtle shading with 2-3 shade levels per color. Consistent top-left lighting. The output image should be EXACTLY 32x32 pixels of pixel art, scaled up to fill the canvas. No border, no frame, no text."

generate_tile() {
  local tile_id="$1"
  local fragment="$2"
  local filename=$(echo "$tile_id" | tr '.' '_')
  
  echo "  Generating: $tile_id"
  python3 "$GEN_SCRIPT" \
    --prompt "$MASTER $fragment" \
    --count 1 \
    --model gpt-image-1 \
    --size 1024x1024 \
    --quality high \
    --out-dir "$RUN_DIR" \
    --output-format png 2>/dev/null
  
  # Rename the output file
  local latest=$(ls -t "$RUN_DIR"/*.png 2>/dev/null | head -1)
  if [ -n "$latest" ] && [ "$latest" != "$RUN_DIR/${filename}.png" ]; then
    mv "$latest" "$RUN_DIR/${filename}.png"
    echo "    → ${filename}.png ✓"
  else
    echo "    → FAILED"
  fi
}

# Phase 1: Anchors
if [ "$PHASE" = "all" ] || [ "$PHASE" = "1" ]; then
  echo "--- Phase 1: Anchors ---"
  generate_tile "elevation.water" "WATER terrain. Oceans, lakes, rivers. Blue water with waves, depth variation. Seamlessly tiles with adjacent water."
  generate_tile "elevation.flat" "FLAT terrain. Open plains, grassland, fertile ground. Light green with hints of dirt paths. Inviting and buildable."
  generate_tile "elevation.mountain" "MOUNTAIN terrain. Rocky mountain terrain with passable trails. Grey-brown rock, some snow. Less extreme than peaks."
  generate_tile "vegetation.forest" "Dense FOREST. Thick tree canopy, dark green evergreens and deciduous mix. Hard to traverse. Showing canopy from above."
  generate_tile "vegetation.good" "GOOD LAND. Rich fertile farmland, lush green fields, golden wheat hints, tall grass. The most desirable land."
fi

# Phase 2: Remaining terrain
if [ "$PHASE" = "all" ] || [ "$PHASE" = "2" ]; then
  echo "--- Phase 2: Terrain Complete ---"
  generate_tile "elevation.peak" "PEAK. Impassable mountain peaks. Snow-capped, jagged, forbidding. Sharp rocky summits. Dangerous and blocking."
  generate_tile "elevation.hill" "HILL terrain. Rolling gentle hills, grassy slopes. Moderate elevation. Green with brown earth showing."
  generate_tile "vegetation.volcano" "VOLCANO. Active volcanic terrain with lava cracks, smoke, ash, sulfur. Orange-red glow on dark rock. Hostile."
  generate_tile "vegetation.desert" "DESERT. Arid sand dunes, cracked dry earth, occasional dead tree. Yellow/tan palette. Harsh sun."
  generate_tile "vegetation.tundra" "TUNDRA. Frozen ground, patchy snow, sparse low scrub. Grey-white-brown. Cold, bleak, windswept."
  generate_tile "vegetation.barren" "BARREN ground. Rocky terrain with occasional tough grass or scrub. Grey-brown with hints of green. Sparse."
  generate_tile "vegetation.light_veg" "LIGHT VEGETATION. Green grassland with wildflowers, scattered low bushes. Pleasant, pastoral. Light green."
  generate_tile "vegetation.wood" "WOOD. Scattered deciduous trees with clearings. Dappled sunlight. Medium green, less dense than forest."
  generate_tile "vegetation.jungle" "JUNGLE. Thick tropical vegetation, vines, dense canopy. Dark green, humid. Extremely difficult terrain."
  generate_tile "vegetation.swamp" "SWAMP. Murky standing water, dead trees, reeds, moss. Dark green-brown, sickly."
  generate_tile "vegetation.ice" "ICE. Frozen ice sheets, blue-white, crystalline. Cracks and glacial features. Cold palette."
fi

# Phase 3: Buildings
if [ "$PHASE" = "all" ] || [ "$PHASE" = "3" ]; then
  echo "--- Phase 3: Buildings ---"
  generate_tile "designation.town" "TOWN. Small medieval settlement. Few buildings, thatched roofs, central square. On green terrain."
  generate_tile "designation.city" "CITY. Large walled medieval city. Multiple buildings, church spire, stone walls. On green terrain."
  generate_tile "designation.mine" "MINE. Mine shaft entrance in hillside, timber supports, ore cart tracks. On rocky terrain."
  generate_tile "designation.farm" "FARM. Plowed fields, a barn, haystacks, crop rows. On green fertile terrain."
  generate_tile "designation.devastated" "DEVASTATED. War-ravaged ruins. Burned buildings, smoke, rubble, craters. Dark and scorched."
  generate_tile "designation.goldmine" "GOLD MINE. Rich mine with visible gold veins, glittering jewel deposits. On rocky terrain."
  generate_tile "designation.fort" "FORT. Stone fortress. Walls, corner towers, battlements, gate. Imposing. On terrain."
  generate_tile "designation.ruin" "RUIN. Crumbling stone walls, collapsed buildings, overgrown weeds. Once grand, now fallen."
  generate_tile "designation.stockade" "STOCKADE. Wooden palisade walls, log towers, a gate. Rough frontier fortification."
  generate_tile "designation.capitol" "CAPITOL. Grand palace/castle, banners flying. Crown jewel of a nation. Should STAND OUT."
  generate_tile "designation.special" "SPECIAL. Mysterious glowing structure, ancient monument, magical aura. Magical energy."
  generate_tile "designation.lumberyard" "LUMBER YARD. Stacked logs, sawmill with water wheel, wood chips. On wooded terrain."
  generate_tile "designation.blacksmith" "BLACKSMITH. Forge with chimney smoke, anvil, glowing metal, weapon racks."
  generate_tile "designation.road" "ROAD. Cobblestone or packed dirt road cutting through green terrain."
  generate_tile "designation.mill" "MILL. Windmill with turning blades on green terrain. Grain processing."
  generate_tile "designation.granary" "GRANARY. Large storage building, grain sacks, full bins. On terrain."
  generate_tile "designation.church" "CHURCH. Medieval church with steeple and cross, small graveyard."
  generate_tile "designation.university" "UNIVERSITY. Grand stone building with columns, tall windows. Academic."
  generate_tile "designation.nodesig" "UNDESIGNATED. Raw cleared land with a boundary marker or small flag. Undeveloped."
  generate_tile "designation.basecamp" "BASE CAMP. Military tents, campfire, weapon stacks, supply wagons."
fi

# Phase 4: Units
if [ "$PHASE" = "all" ] || [ "$PHASE" = "4" ]; then
  echo "--- Phase 4: Units ---"
  generate_tile "units.army" "ARMY unit. Group of medieval soldiers with weapons and banner on green terrain. Military force."
  generate_tile "units.navy" "NAVY fleet. Medieval warship with sails on blue water. Wooden hull, flag."
fi

echo ""
echo "=== Generation complete ==="
echo "Output: $RUN_DIR"
echo "Files: $(ls "$RUN_DIR"/*.png 2>/dev/null | wc -l) tiles"
