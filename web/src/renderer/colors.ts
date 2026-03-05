// colors.ts — Terminal color palette matching original curses colors

// 8 base curses colors + bold variants
export const CURSES_COLORS = {
  black:   '#000000',
  red:     '#aa0000',
  green:   '#00aa00',
  yellow:  '#aa5500',
  blue:    '#0000aa',
  magenta: '#aa00aa',
  cyan:    '#00aaaa',
  white:   '#aaaaaa',
  // Bold (bright) variants
  brightBlack:   '#555555',
  brightRed:     '#ff5555',
  brightGreen:   '#55ff55',
  brightYellow:  '#ffff55',
  brightBlue:    '#5555ff',
  brightMagenta: '#ff55ff',
  brightCyan:    '#55ffff',
  brightWhite:   '#ffffff',
};

// Index-based access (0-7 normal, 8-15 bright)
export const COLOR_PALETTE: string[] = [
  CURSES_COLORS.black,
  CURSES_COLORS.red,
  CURSES_COLORS.green,
  CURSES_COLORS.yellow,
  CURSES_COLORS.blue,
  CURSES_COLORS.magenta,
  CURSES_COLORS.cyan,
  CURSES_COLORS.white,
  CURSES_COLORS.brightBlack,
  CURSES_COLORS.brightRed,
  CURSES_COLORS.brightGreen,
  CURSES_COLORS.brightYellow,
  CURSES_COLORS.brightBlue,
  CURSES_COLORS.brightMagenta,
  CURSES_COLORS.brightCyan,
  CURSES_COLORS.brightWhite,
];

// Nation colors — each nation gets a unique color combo
// Original curses used COLOR_PAIR(nation_id % 7 + 1) roughly
export function nationFgColor(nationId: number): string {
  if (nationId === 0) return CURSES_COLORS.white;
  const colors = [
    CURSES_COLORS.brightRed,
    CURSES_COLORS.brightGreen,
    CURSES_COLORS.brightYellow,
    CURSES_COLORS.brightBlue,
    CURSES_COLORS.brightMagenta,
    CURSES_COLORS.brightCyan,
    CURSES_COLORS.brightWhite,
  ];
  return colors[(nationId - 1) % colors.length];
}

// Vegetation colors
export function vegetationColor(vegIndex: number): string {
  const vegColors: Record<number, string> = {
    0: CURSES_COLORS.brightRed,    // Volcano
    1: CURSES_COLORS.yellow,       // Desert
    2: CURSES_COLORS.white,        // Tundra
    3: CURSES_COLORS.yellow,       // Barren
    4: CURSES_COLORS.green,        // Light Veg
    5: CURSES_COLORS.brightGreen,  // Good
    6: CURSES_COLORS.green,        // Wood
    7: CURSES_COLORS.green,        // Forest
    8: CURSES_COLORS.brightGreen,  // Jungle
    9: CURSES_COLORS.magenta,      // Swamp
    10: CURSES_COLORS.brightCyan,  // Ice
    11: CURSES_COLORS.cyan,        // None (water)
  };
  return vegColors[vegIndex] ?? CURSES_COLORS.white;
}

// Altitude colors
export function altitudeColor(altIndex: number): string {
  const altColors: Record<number, string> = {
    0: CURSES_COLORS.cyan,          // Water
    1: CURSES_COLORS.brightWhite,   // Peak
    2: CURSES_COLORS.white,         // Mountain
    3: CURSES_COLORS.yellow,        // Hill
    4: CURSES_COLORS.green,         // Clear/Flat
  };
  return altColors[altIndex] ?? CURSES_COLORS.white;
}
