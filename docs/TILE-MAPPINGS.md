# Tile Mappings — What Each Character Actually Means

Source: `gpl-release/data.c` (v4.12)

## Elevation (`ele`) — `~#^%-`
These represent terrain HEIGHT. Shown in Contour display mode.

| Index | Char | Name     | Meaning                                          |
|-------|------|----------|--------------------------------------------------|
| 0     | `~`  | WATER    | Oceans, lakes, rivers — impassable except by navy |
| 1     | `#`  | PEAK     | Mountain peaks — completely impassable            |
| 2     | `^`  | MOUNTAIN | Mountainous terrain — slow, high defense bonus    |
| 3     | `%`  | HILL     | Hilly terrain — moderate movement cost            |
| 4     | `-`  | FLAT     | Plains/flatland — easy movement                   |

## Vegetation (`veg`) — `vdtblgwfjsi~`
These represent what GROWS on the terrain. Shown in Vegetation display mode.
Vegetation determines food production (`vegfood = 0004697400000`).

| Index | Char | Name    | Meaning                               | Food |
|-------|------|---------|---------------------------------------|------|
| 0     | `v`  | VOLCANO | Active volcanic terrain — no food     | 0    |
| 1     | `d`  | DESERT  | Arid wasteland — no food              | 0    |
| 2     | `t`  | TUNDRA  | Frozen ground — no food               | 0    |
| 3     | `b`  | BARREN  | Rocky/barren ground — minimal food    | 4    |
| 4     | `l`  | LT_VEG  | Light vegetation — some food          | 6    |
| 5     | `g`  | GOOD    | Good farmland/grassland — best food   | 9    |
| 6     | `w`  | WOOD    | Wooded area — decent food             | 7    |
| 7     | `f`  | FOREST  | Dense forest — moderate food          | 4    |
| 8     | `j`  | JUNGLE  | Tropical jungle — no food, high defense | 0  |
| 9     | `s`  | SWAMP   | Swampland — no food, slow movement    | 0    |
| 10    | `i`  | ICE     | Frozen ice — no food                  | 0    |
| 11    | `~`  | NONE    | No vegetation (water/peak)            | 0    |

## Designation (`des`) — `tcmfx$!&sC?lb+*g=u-P`
These represent what a sector has been DEVELOPED into. Shown in Designation mode for owned sectors.

| Index | Char | Name       | Meaning                                           |
|-------|------|------------|---------------------------------------------------|
| 0     | `t`  | TOWN       | Small settlement — basic sector                   |
| 1     | `c`  | CITY       | Large settlement — more production                |
| 2     | `m`  | MINE       | Metal extraction — produces metal resource         |
| 3     | `f`  | FARM       | Agricultural — food production bonus               |
| 4     | `x`  | DEVASTATED | War-ravaged — destroyed by combat                 |
| 5     | `$`  | GOLDMINE   | Gold extraction — produces gold/jewels             |
| 6     | `!`  | FORT       | Military fortification — defense bonus             |
| 7     | `&`  | RUIN       | Ruined city/capitol — from destruction             |
| 8     | `s`  | STOCKADE   | Basic wooden fortification — less defense than fort|
| 9     | `C`  | CAPITOL    | Nation's capital — most important sector            |
| 10    | `?`  | SPECIAL    | Special/unique sector                              |
| 11    | `l`  | LUMBERYD   | Lumber yard — wood production                      |
| 12    | `b`  | BLKSMITH   | Blacksmith — equipment/metal processing            |
| 13    | `+`  | ROAD       | Road — faster movement                             |
| 14    | `*`  | MILL       | Grain/lumber mill — production bonus               |
| 15    | `g`  | GRANARY    | Food storage — food bonus                          |
| 16    | `=`  | CHURCH     | Religious building — morale/alignment              |
| 17    | `u`  | UNIVERSITY | Education — research/technology                    |
| 18    | `-`  | NODESIG    | Undesignated — raw land, no development            |
| 19    | `P`  | BASECAMP   | Military base camp                                 |

## Display Modes and What They Show
- **Vegetation**: shows veg chars for every sector
- **Designation**: owned sectors show des char, unowned show elevation
- **Contour**: shows elevation chars everywhere
- **Food**: shows food value digit (0-9, + for 10+)
- **Nation**: shows nation mark char for owned sectors, elevation for unowned
- **Race**: shows race initial (H/O/E/D/L) for owned, elevation for unowned
- **Move**: shows movement cost digit for current race
- **Defense**: shows defense bonus digit
- **People**: shows population magnitude (0-9, I/V/X for thousands)
- **Gold**: shows jewel count per sector
- **Metal**: shows metal count per sector
- **Items**: shows trade goods

## Army Markers
- `A` — Army unit on the map (overlays sector display)
- `N` — Navy fleet (when toggle_navy is active)

## Nation Marks
Each nation has a single character mark (like `T`, `*`, etc.) chosen at creation.
Shown in Designation mode for enemy sectors, and in Nation display mode.
