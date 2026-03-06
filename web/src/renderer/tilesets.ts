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
  elevation: [
    emojiTile('🌊'),  // water
    emojiTile('🏔️'),  // peak
    emojiTile('⛰️'),   // mountain
    emojiTile('🏕️'),  // hill
    emojiTile('🟩'),  // flat
  ],
  vegetation: [
    emojiTile('🌿'),  // vine
    emojiTile('🏜️'),  // desert
    emojiTile('🌲'),  // tree
    emojiTile('🌾'),  // brush
    emojiTile('🌳'),  // lush
    emojiTile('🌾'),  // grain
    emojiTile('💧'),  // water veg
    emojiTile('🌲'),  // forest
    emojiTile('🌴'),  // jungle
    emojiTile('🪸'),   // swamp
    emojiTile('🧊'),  // ice
    emojiTile('❄️'),   // tundra
  ],
  designation: [
    emojiTile('🏘️'),  // town
    emojiTile('🏙️'),  // city
    emojiTile('⛏️'),   // mine
    emojiTile('🌾'),  // farm
    emojiTile('🎣'),  // fishery
    emojiTile('👑'),  // capital
    emojiTile('🏰'),  // fort
    emojiTile('🏯'),  // castle
    emojiTile('🪵'),  // stockade
    emojiTile('⭐'),  // capitol
    emojiTile('❓'),  // unknown
    emojiTile('🪓'),  // logging
    emojiTile('🌉'),  // bridge
    emojiTile('🛤️'),  // road
    emojiTile('💰'),  // goldmine
    emojiTile('🏪'),  // granary
    emojiTile('🏫'),  // university
    emojiTile('⚓'),  // harbor
    emojiTile('🏗️'),  // misc
    emojiTile('🏛️'),  // palace
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
    charTile('≈'),  // water
    charTile('▲'),  // peak
    charTile('△'),  // mountain
    charTile('∧'),  // hill
    charTile('·'),  // flat
  ],
  vegetation: [
    charTile('♣'),  // vine
    charTile('░'),  // desert
    charTile('↟'),  // tree
    charTile('≋'),  // brush
    charTile('♠'),  // lush
    charTile('⌇'),  // grain
    charTile('≈'),  // water veg
    charTile('⌘'),  // forest
    charTile('❦'),  // jungle
    charTile('⌁'),  // swamp
    charTile('◇'),  // ice
    charTile('○'),  // tundra
  ],
  designation: [
    charTile('⌂'),  // town
    charTile('▣'),  // city
    charTile('⛏'),  // mine
    charTile('⌗'),  // farm
    charTile('⚓'),  // fishery
    charTile('♛'),  // capital
    charTile('⚑'),  // fort
    charTile('♜'),  // castle
    charTile('⌸'),  // stockade
    charTile('★'),  // capitol
    charTile('?'),  // unknown
    charTile('⌻'),  // logging
    charTile('═'),  // bridge
    charTile('─'),  // road
    charTile('⊛'),  // goldmine
    charTile('⊞'),  // granary
    charTile('⊠'),  // university
    charTile('⚓'),  // harbor
    charTile('⊡'),  // misc
    charTile('⊕'),  // palace
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
    emojiTile('🌊'),
    emojiTile('🗻'),
    emojiTile('⛰️'),
    emojiTile('🏔️'),
    emojiTile('🟢'),
  ],
  vegetation: [
    emojiTile('🌿'), emojiTile('💀'), emojiTile('🌲'), emojiTile('🍂'),
    emojiTile('🌳'), emojiTile('🌾'), emojiTile('🐟'), emojiTile('🌲'),
    emojiTile('🐉'), emojiTile('🦎'), emojiTile('❄️'), emojiTile('🐺'),
  ],
  designation: [
    emojiTile('🏠'), emojiTile('🏰'), emojiTile('⛏️'), emojiTile('🌾'),
    emojiTile('🐟'), emojiTile('👑'), emojiTile('🛡️'), emojiTile('🏰'),
    emojiTile('🪵'), emojiTile('⭐'), emojiTile('❓'), emojiTile('🪓'),
    emojiTile('🌉'), emojiTile('🛤️'), emojiTile('💰'), emojiTile('🍺'),
    emojiTile('📜'), emojiTile('⚓'), emojiTile('🔨'), emojiTile('🏛️'),
  ],
  army: emojiTile('⚔️'),
  navy: emojiTile('🚢'),
  cursor: emojiTile('🔮'),
  fog: emojiTile('🌫️'),
  unknown: emojiTile('❓'),
};

// ─── Registry ───

export const ALL_TILESETS: TileSet[] = [
  TILESET_ASCII,
  TILESET_EMOJI,
  TILESET_UNICODE,
  TILESET_FANTASY,
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
