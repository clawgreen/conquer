// tilesets.ts — Pluggable tile mapping system
// Maps game terrain/vegetation/designation/units to visual representations
// Supports: characters (ASCII/Unicode), emojis, and image sprites

export type TileType = 'char' | 'emoji' | 'image';

export interface TileDef {
  type: TileType;
  value: string;        // character, emoji, or image URL/path
  // Optional per-tile color override (char/emoji mode)
  fg?: string;
  bg?: string;
}

export interface TileSet {
  id: string;
  name: string;
  tileType: TileType;           // primary type for this set
  cellWidth: number;            // pixels per cell X
  cellHeight: number;           // pixels per cell Y
  // Terrain elevation: water, peak, mountain, hill, flat (indices 0-5 map to ~#^%- and flat)
  elevation: TileDef[];
  // Vegetation: vine, desert, tree, brush, lush, grain, water, forest, jungle, swamp, ice, tundra (0-11)
  vegetation: TileDef[];
  // Designation: town, city, mine, farm, fishery, capital, fort, castle, stockade, capitol, ?, logging, bridge, road, goldmine, granary, university, harbor, ? (0-20)
  designation: TileDef[];
  // Units
  army: TileDef;
  navy: TileDef;
  cursor: TileDef;
  // Fog of war
  fog: TileDef;
  // Fallback
  unknown: TileDef;
}

// ─── ASCII Classic (matches original C game exactly) ───

const ELEV_CHARS = '~#^%-';   // water, peak, mountain, hill, flat
const VEG_CHARS  = 'vdtblgwfjsi~';  // from data.c
const DESIG_CHARS = 'tcmfx$!&sC?lb+*g=u-P';

function charTile(ch: string, fg?: string, bg?: string): TileDef {
  return { type: 'char', value: ch, fg, bg };
}
function emojiTile(em: string): TileDef {
  return { type: 'emoji', value: em };
}
function imgTile(url: string): TileDef {
  return { type: 'image', value: url };
}

function charsToTiles(s: string): TileDef[] {
  return [...s].map(c => charTile(c));
}

export const TILESET_ASCII: TileSet = {
  id: 'ascii',
  name: 'ASCII Classic',
  tileType: 'char',
  cellWidth: 14,    // ~2 chars wide at 14px font
  cellHeight: 16,
  elevation: charsToTiles(ELEV_CHARS),
  vegetation: charsToTiles(VEG_CHARS),
  designation: charsToTiles(DESIG_CHARS),
  army: charTile('A'),
  navy: charTile('N'),
  cursor: charTile('+'),
  fog: charTile('.'),
  unknown: charTile('?'),
};

// ─── Emoji ───

export const TILESET_EMOJI: TileSet = {
  id: 'emoji',
  name: 'Emoji',
  tileType: 'emoji',
  cellWidth: 24,
  cellHeight: 24,
  // Elevation: terrain height
  elevation: [
    emojiTile('🌊'),  // 0: WATER — oceans, lakes, rivers
    emojiTile('🗻'),  // 1: PEAK — impassable mountain peak
    emojiTile('⛰️'),   // 2: MOUNTAIN — mountainous, slow movement
    emojiTile('🏔️'),  // 3: HILL — hilly terrain
    emojiTile('🟩'),  // 4: FLAT — plains, easy movement
  ],
  // Vegetation: what grows on the land
  vegetation: [
    emojiTile('🌋'),  // 0: VOLCANO — active volcanic terrain
    emojiTile('🏜️'),  // 1: DESERT — arid wasteland
    emojiTile('🥶'),  // 2: TUNDRA — frozen ground
    emojiTile('🪨'),  // 3: BARREN — rocky, minimal food
    emojiTile('🌱'),  // 4: LT_VEG — light vegetation, some food
    emojiTile('🌾'),  // 5: GOOD — good farmland/grassland, best food
    emojiTile('🌳'),  // 6: WOOD — wooded area
    emojiTile('🌲'),  // 7: FOREST — dense forest
    emojiTile('🌴'),  // 8: JUNGLE — tropical jungle, high defense
    emojiTile('🐊'),  // 9: SWAMP — swampland, slow
    emojiTile('🧊'),  // 10: ICE — frozen ice
    emojiTile('➖'),  // 11: NONE — no vegetation (water/peak)
  ],
  // Designation: what sectors have been developed into
  designation: [
    emojiTile('🏘️'),  // 0: TOWN — small settlement
    emojiTile('🏙️'),  // 1: CITY — large settlement
    emojiTile('⛏️'),   // 2: MINE — metal extraction
    emojiTile('👨‍🌾'), // 3: FARM — agriculture
    emojiTile('💀'),  // 4: DEVASTATED — war-ravaged
    emojiTile('💰'),  // 5: GOLDMINE — gold/jewel extraction
    emojiTile('🏰'),  // 6: FORT — military fortification
    emojiTile('🏚️'),  // 7: RUIN — ruined city/capitol
    emojiTile('🪵'),  // 8: STOCKADE — wooden fortification
    emojiTile('👑'),  // 9: CAPITOL — nation's capital!
    emojiTile('❓'),  // 10: SPECIAL — unique sector
    emojiTile('🪓'),  // 11: LUMBERYARD — wood production
    emojiTile('🔨'),  // 12: BLACKSMITH — metal processing
    emojiTile('🛤️'),  // 13: ROAD — faster movement
    emojiTile('⚙️'),  // 14: MILL — production bonus
    emojiTile('🏪'),  // 15: GRANARY — food storage
    emojiTile('⛪'),  // 16: CHURCH — morale/alignment
    emojiTile('🏫'),  // 17: UNIVERSITY — research
    emojiTile('🟫'),  // 18: NODESIG — undesignated raw land
    emojiTile('⛺'),  // 19: BASECAMP — military base camp
  ],
  army: emojiTile('⚔️'),
  navy: emojiTile('⛵'),
  cursor: emojiTile('📍'),
  fog: emojiTile('🌫️'),
  unknown: emojiTile('❓'),
};

// ─── Unicode Box Drawing / Braille ───

export const TILESET_UNICODE: TileSet = {
  id: 'unicode',
  name: 'Unicode Symbols',
  tileType: 'char',
  cellWidth: 14,
  cellHeight: 16,
  elevation: [
    charTile('≈'),  // WATER
    charTile('▲'),  // PEAK — impassable
    charTile('△'),  // MOUNTAIN
    charTile('∧'),  // HILL
    charTile('·'),  // FLAT
  ],
  vegetation: [
    charTile('♨'),  // VOLCANO
    charTile('░'),  // DESERT
    charTile('※'),  // TUNDRA
    charTile('∴'),  // BARREN
    charTile('♣'),  // LT_VEG — light vegetation
    charTile('♠'),  // GOOD — good farmland
    charTile('↟'),  // WOOD
    charTile('⌘'),  // FOREST
    charTile('❦'),  // JUNGLE
    charTile('≋'),  // SWAMP
    charTile('◇'),  // ICE
    charTile('○'),  // NONE
  ],
  designation: [
    charTile('⌂'),  // TOWN
    charTile('▣'),  // CITY
    charTile('⛏'),  // MINE
    charTile('⌗'),  // FARM
    charTile('✕'),  // DEVASTATED
    charTile('⊛'),  // GOLDMINE
    charTile('⚑'),  // FORT
    charTile('♜'),  // RUIN
    charTile('⌸'),  // STOCKADE
    charTile('★'),  // CAPITOL
    charTile('?'),  // SPECIAL
    charTile('⌻'),  // LUMBERYARD
    charTile('⚒'),  // BLACKSMITH
    charTile('─'),  // ROAD
    charTile('⊙'),  // MILL
    charTile('⊞'),  // GRANARY
    charTile('†'),  // CHURCH
    charTile('⊠'),  // UNIVERSITY
    charTile('·'),  // NODESIG
    charTile('⊕'),  // BASECAMP
  ],
  army: charTile('♞'),
  navy: charTile('♚'),
  cursor: charTile('⊹'),
  fog: charTile('░'),
  unknown: charTile('?'),
};

// ─── Fantasy (emoji-based but with fantasy flavor) ───

export const TILESET_FANTASY: TileSet = {
  id: 'fantasy',
  name: 'Fantasy',
  tileType: 'emoji',
  cellWidth: 24,
  cellHeight: 24,
  elevation: [
    emojiTile('🌊'),  // WATER
    emojiTile('🗻'),  // PEAK
    emojiTile('⛰️'),   // MOUNTAIN
    emojiTile('🏔️'),  // HILL
    emojiTile('🟢'),  // FLAT
  ],
  vegetation: [
    emojiTile('🔥'),  // VOLCANO — fire and brimstone
    emojiTile('💀'),  // DESERT — deathlands
    emojiTile('🐺'),  // TUNDRA — wolves in the snow
    emojiTile('🦴'),  // BARREN — bones and dust
    emojiTile('🍀'),  // LT_VEG — clover fields
    emojiTile('🌻'),  // GOOD — fertile enchanted fields
    emojiTile('🍄'),  // WOOD — mushroom woods
    emojiTile('🌲'),  // FOREST — deep dark forest
    emojiTile('🐉'),  // JUNGLE — dragon territory
    emojiTile('🦎'),  // SWAMP — lizard swamps
    emojiTile('❄️'),   // ICE — frozen wastes
    emojiTile('🕳️'),  // NONE — the void
  ],
  designation: [
    emojiTile('🏠'),  // TOWN — village
    emojiTile('🏰'),  // CITY — castle city
    emojiTile('⛏️'),   // MINE — dwarf mines
    emojiTile('🌾'),  // FARM — peasant farms
    emojiTile('🔥'),  // DEVASTATED — burned ruins
    emojiTile('💎'),  // GOLDMINE — gem mines
    emojiTile('🛡️'),  // FORT — shield wall
    emojiTile('💀'),  // RUIN — haunted ruins
    emojiTile('🪵'),  // STOCKADE — palisade
    emojiTile('👑'),  // CAPITOL — throne room
    emojiTile('✨'),  // SPECIAL — magical
    emojiTile('🪓'),  // LUMBERYARD
    emojiTile('⚒️'),  // BLACKSMITH — forge
    emojiTile('🛤️'),  // ROAD — king's road
    emojiTile('⚙️'),   // MILL — windmill
    emojiTile('🍺'),  // GRANARY — mead hall
    emojiTile('📜'),  // CHURCH — temple
    emojiTile('🧙'),  // UNIVERSITY — wizard tower
    emojiTile('🟤'),  // NODESIG — wild land
    emojiTile('⛺'),  // BASECAMP — war camp
  ],
  army: emojiTile('⚔️'),
  navy: emojiTile('🚢'),
  cursor: emojiTile('🔮'),
  fog: emojiTile('🌫️'),
  unknown: emojiTile('❓'),
};

// ─── Pixel32 SNES-era image tileset ───

const P32 = '/tilesets/pixel32/';

export const TILESET_PIXEL32: TileSet = {
  id: 'pixel32',
  name: 'SNES Pixel Art',
  tileType: 'image',
  cellWidth: 32,
  cellHeight: 32,
  elevation: [
    imgTile(P32 + 'elevation_water.png'),     // 0: WATER
    imgTile(P32 + 'elevation_peak.png'),      // 1: PEAK
    imgTile(P32 + 'elevation_mountain.png'),  // 2: MOUNTAIN
    imgTile(P32 + 'elevation_hill.png'),      // 3: HILL
    imgTile(P32 + 'elevation_flat.png'),      // 4: FLAT
  ],
  vegetation: [
    imgTile(P32 + 'vegetation_volcano.png'),   // 0: VOLCANO
    imgTile(P32 + 'vegetation_desert.png'),    // 1: DESERT
    imgTile(P32 + 'vegetation_tundra.png'),    // 2: TUNDRA
    imgTile(P32 + 'vegetation_barren.png'),    // 3: BARREN
    imgTile(P32 + 'vegetation_light_veg.png'), // 4: LT_VEG
    imgTile(P32 + 'vegetation_good.png'),      // 5: GOOD
    imgTile(P32 + 'vegetation_wood.png'),      // 6: WOOD
    imgTile(P32 + 'vegetation_forest.png'),    // 7: FOREST
    imgTile(P32 + 'vegetation_jungle.png'),    // 8: JUNGLE
    imgTile(P32 + 'vegetation_swamp.png'),     // 9: SWAMP
    imgTile(P32 + 'vegetation_ice.png'),       // 10: ICE
    imgTile(P32 + 'elevation_flat.png'),       // 11: NONE (use flat terrain)
  ],
  designation: [
    imgTile(P32 + 'designation_town.png'),        // 0: TOWN
    imgTile(P32 + 'designation_city.png'),        // 1: CITY
    imgTile(P32 + 'designation_mine.png'),        // 2: MINE
    imgTile(P32 + 'designation_farm.png'),        // 3: FARM
    imgTile(P32 + 'designation_devastated.png'),  // 4: DEVASTATED
    imgTile(P32 + 'designation_goldmine.png'),    // 5: GOLDMINE
    imgTile(P32 + 'designation_fort.png'),        // 6: FORT
    imgTile(P32 + 'designation_ruin.png'),        // 7: RUIN
    imgTile(P32 + 'designation_stockade.png'),    // 8: STOCKADE
    imgTile(P32 + 'designation_capitol.png'),     // 9: CAPITOL
    imgTile(P32 + 'designation_special.png'),     // 10: SPECIAL
    imgTile(P32 + 'designation_lumberyard.png'),  // 11: LUMBERYARD
    imgTile(P32 + 'designation_blacksmith.png'),  // 12: BLACKSMITH
    imgTile(P32 + 'designation_road.png'),        // 13: ROAD
    imgTile(P32 + 'designation_mill.png'),        // 14: MILL
    imgTile(P32 + 'designation_granary.png'),     // 15: GRANARY
    imgTile(P32 + 'designation_church.png'),      // 16: CHURCH
    imgTile(P32 + 'designation_university.png'),  // 17: UNIVERSITY
    imgTile(P32 + 'designation_nodesig.png'),     // 18: NODESIG
    imgTile(P32 + 'designation_basecamp.png'),    // 19: BASECAMP
  ],
  army: imgTile(P32 + 'units_army.png'),
  navy: imgTile(P32 + 'units_navy.png'),
  cursor: charTile('+', '#00ff00'),  // green crosshair for cursor
  fog: { type: 'image', value: '', fg: '#000000', bg: '#000000' },  // solid black fog
  unknown: charTile('?', '#ff0000'),
};

// ─── Registry ───

export const ALL_TILESETS: TileSet[] = [
  TILESET_ASCII,
  TILESET_EMOJI,
  TILESET_UNICODE,
  TILESET_FANTASY,
  TILESET_PIXEL32,
];

/** Register a custom tileset at runtime (e.g. from editor) */
export function registerTileset(ts: TileSet): void {
  const idx = ALL_TILESETS.findIndex(t => t.id === ts.id);
  if (idx >= 0) ALL_TILESETS[idx] = ts;
  else ALL_TILESETS.push(ts);
}

export function getTileset(id: string): TileSet {
  return ALL_TILESETS.find(t => t.id === id) ?? TILESET_ASCII;
}

/**
 * Get effective cell dimensions for a tileset, scaled by zoom factor.
 * For char tilesets, cell size comes from the terminal renderer.
 * For emoji/image tilesets, scale the base cellWidth/cellHeight by zoom.
 * zoomFactor = currentFontSize / defaultFontSize (14)
 */
export function getScaledCellSize(ts: TileSet, fontSize: number): { cw: number; ch: number } {
  if (ts.tileType === 'char') {
    // Char mode uses terminal grid — handled elsewhere
    return { cw: ts.cellWidth, ch: ts.cellHeight };
  }
  const zoomFactor = fontSize / 14; // 14px is the baseline font
  return {
    cw: Math.round(ts.cellWidth * zoomFactor),
    ch: Math.round(ts.cellHeight * zoomFactor),
  };
}

// ─── Image tileset loading (for sprite-based tilesets) ───

const imageCache: Map<string, HTMLImageElement> = new Map();

export function preloadTilesetImages(ts: TileSet): Promise<void[]> {
  if (ts.tileType !== 'image') return Promise.resolve([]);
  const allTiles = [
    ...ts.elevation, ...ts.vegetation, ...ts.designation,
    ts.army, ts.navy, ts.cursor, ts.fog, ts.unknown,
  ];
  const promises = allTiles
    .filter(t => t.type === 'image' && !imageCache.has(t.value))
    .map(t => new Promise<void>((resolve) => {
      const img = new Image();
      img.onload = () => { imageCache.set(t.value, img); resolve(); };
      img.onerror = () => resolve(); // fail silently
      img.src = t.value;
    }));
  return Promise.all(promises);
}

export function getCachedImage(url: string): HTMLImageElement | undefined {
  return imageCache.get(url);
}
