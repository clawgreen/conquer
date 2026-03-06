// themes.ts — Pluggable render theme system for the map display
//
// Each theme defines:
//   - fg/bg colors for terrain types (altitude, vegetation, designation)
//   - highlight style (inverse, underline color, glow color)
//   - optional character overrides (e.g. emoji mode)
//   - fog of war appearance
//   - UI accent colors for HUD/panels

export interface SectorStyle {
  fg: string;
  bg: string;
  bold?: boolean;
}

export interface ThemeDef {
  id: string;
  name: string;
  description: string;
  icon: string;  // emoji for selector

  // Base colors
  mapBg: string;           // canvas background
  fogFg: string;           // fog of war text color
  fogBg: string;           // fog of war background
  cursorFg: string;        // cursor color

  // How to color a sector by category
  water: SectorStyle;
  peak: SectorStyle;
  mountain: SectorStyle;
  hill: SectorStyle;
  flat: SectorStyle;

  // Vegetation overlay colors (used in veg display mode)
  vegColors: Record<number, SectorStyle>;

  // Owned sector styling
  ownSector: SectorStyle;          // your sectors
  enemySector: (nationId: number) => SectorStyle;  // other nation sectors

  // Highlight = reverse video? Or custom bg?
  highlightStyle: 'inverse' | 'bg';
  highlightBg: string;     // used when highlightStyle === 'bg'

  // Character overrides (optional — undefined = use default ASCII)
  charOverrides?: Record<string, string>;

  // UI accent
  uiAccent: string;
  uiBg: string;
  uiText: string;
  uiDim: string;
}

// ── Nation color rotation for themes that color by nation ──
const NATION_COLORS = [
  '#ff5555', '#55ff55', '#ffff55', '#5555ff',
  '#ff55ff', '#55ffff', '#ffffff', '#ff8844',
  '#44ff88', '#8855ff', '#ff4488', '#88ffff',
];

function nationColor(id: number): string {
  if (id === 0) return '#aaaaaa';
  return NATION_COLORS[(id - 1) % NATION_COLORS.length];
}

// ══════════════════════════════════════════════
//  CLASSIC — Authentic 1988 green phosphor
// ══════════════════════════════════════════════
const CLASSIC_GREEN: ThemeDef = {
  id: 'classic-green',
  name: 'Classic Green',
  description: 'Authentic 1988 green phosphor terminal',
  icon: '🖥',
  mapBg: '#000000',
  fogFg: '#001100',
  fogBg: '#000000',
  cursorFg: '#55ff55',
  water:    { fg: '#00aa00', bg: '#000000' },
  peak:     { fg: '#00aa00', bg: '#000000' },
  mountain: { fg: '#00aa00', bg: '#000000' },
  hill:     { fg: '#00aa00', bg: '#000000' },
  flat:     { fg: '#00aa00', bg: '#000000' },
  vegColors: {},  // all green — no differentiation
  ownSector: { fg: '#00aa00', bg: '#000000' },
  enemySector: () => ({ fg: '#00aa00', bg: '#000000' }),
  highlightStyle: 'inverse',
  highlightBg: '#00aa00',
  uiAccent: '#55ff55', uiBg: '#001100', uiText: '#00aa00', uiDim: '#004400',
};

// ══════════════════════════════════════════════
//  CLASSIC AMBER — Amber phosphor terminal
// ══════════════════════════════════════════════
const CLASSIC_AMBER: ThemeDef = {
  id: 'classic-amber',
  name: 'Classic Amber',
  description: 'Amber phosphor terminal (Wyse/ADM style)',
  icon: '🟠',
  mapBg: '#000000',
  fogFg: '#110800',
  fogBg: '#000000',
  cursorFg: '#ffbb33',
  water:    { fg: '#cc8800', bg: '#000000' },
  peak:     { fg: '#cc8800', bg: '#000000' },
  mountain: { fg: '#cc8800', bg: '#000000' },
  hill:     { fg: '#cc8800', bg: '#000000' },
  flat:     { fg: '#cc8800', bg: '#000000' },
  vegColors: {},
  ownSector: { fg: '#cc8800', bg: '#000000' },
  enemySector: () => ({ fg: '#cc8800', bg: '#000000' }),
  highlightStyle: 'inverse',
  highlightBg: '#cc8800',
  uiAccent: '#ffbb33', uiBg: '#110800', uiText: '#cc8800', uiDim: '#553300',
};

// ══════════════════════════════════════════════
//  CLASSIC WHITE — White phosphor (VT100)
// ══════════════════════════════════════════════
const CLASSIC_WHITE: ThemeDef = {
  id: 'classic-white',
  name: 'Classic White',
  description: 'White phosphor terminal (VT100/VT220)',
  icon: '⬜',
  mapBg: '#000000',
  fogFg: '#111111',
  fogBg: '#000000',
  cursorFg: '#ffffff',
  water:    { fg: '#aaaaaa', bg: '#000000' },
  peak:     { fg: '#aaaaaa', bg: '#000000' },
  mountain: { fg: '#aaaaaa', bg: '#000000' },
  hill:     { fg: '#aaaaaa', bg: '#000000' },
  flat:     { fg: '#aaaaaa', bg: '#000000' },
  vegColors: {},
  ownSector: { fg: '#aaaaaa', bg: '#000000' },
  enemySector: () => ({ fg: '#aaaaaa', bg: '#000000' }),
  highlightStyle: 'inverse',
  highlightBg: '#aaaaaa',
  uiAccent: '#ffffff', uiBg: '#111111', uiText: '#aaaaaa', uiDim: '#444444',
};

// ══════════════════════════════════════════════
//  ENHANCED — Full color with terrain colors
// ══════════════════════════════════════════════
const ENHANCED: ThemeDef = {
  id: 'enhanced',
  name: 'Enhanced',
  description: 'Full color terrain and nation display',
  icon: '🎨',
  mapBg: '#000000',
  fogFg: '#222222',
  fogBg: '#0a0a0a',
  cursorFg: '#ffffff',
  water:    { fg: '#0088cc', bg: '#000011' },
  peak:     { fg: '#ffffff', bg: '#111111', bold: true },
  mountain: { fg: '#aaaaaa', bg: '#0a0a0a' },
  hill:     { fg: '#aa8833', bg: '#000000' },
  flat:     { fg: '#44aa44', bg: '#000000' },
  vegColors: {
    0:  { fg: '#ff4400', bg: '#110000' },  // Volcano
    1:  { fg: '#ccaa44', bg: '#0a0800' },  // Desert
    2:  { fg: '#aaaacc', bg: '#000000' },  // Tundra
    3:  { fg: '#887744', bg: '#000000' },  // Barren
    4:  { fg: '#44aa44', bg: '#000000' },  // Light Veg
    5:  { fg: '#55ff55', bg: '#001100' },  // Good
    6:  { fg: '#228822', bg: '#000800' },  // Wood
    7:  { fg: '#116611', bg: '#000600' },  // Forest
    8:  { fg: '#00cc44', bg: '#000a00' },  // Jungle
    9:  { fg: '#8844aa', bg: '#080008' },  // Swamp
    10: { fg: '#88ccff', bg: '#000811' },  // Ice
    11: { fg: '#0088cc', bg: '#000011' },  // None/water
  },
  ownSector: { fg: '#ffffff', bg: '#000000', bold: true },
  enemySector: (id: number) => ({ fg: nationColor(id), bg: '#000000' }),
  highlightStyle: 'bg',
  highlightBg: '#333300',
  uiAccent: '#55ff55', uiBg: '#001100', uiText: '#aaffaa', uiDim: '#336633',
};

// ══════════════════════════════════════════════
//  TACTICAL — Military/topo map feel
// ══════════════════════════════════════════════
const TACTICAL: ThemeDef = {
  id: 'tactical',
  name: 'Tactical',
  description: 'Military topographic map overlay',
  icon: '🗺',
  mapBg: '#0a0f0a',
  fogFg: '#0a1a0a',
  fogBg: '#050a05',
  cursorFg: '#ff3333',
  water:    { fg: '#224466', bg: '#0a0f14' },
  peak:     { fg: '#cccccc', bg: '#1a1a1a' },
  mountain: { fg: '#886644', bg: '#0f0a08' },
  hill:     { fg: '#998855', bg: '#0a0f0a' },
  flat:     { fg: '#446633', bg: '#0a0f0a' },
  vegColors: {
    0:  { fg: '#cc3300', bg: '#1a0800' },
    1:  { fg: '#ccaa66', bg: '#0f0a05' },
    2:  { fg: '#99aacc', bg: '#0a0f0a' },
    3:  { fg: '#776644', bg: '#0a0f0a' },
    4:  { fg: '#558844', bg: '#0a0f0a' },
    5:  { fg: '#66cc66', bg: '#0a1a0a' },
    6:  { fg: '#336633', bg: '#0a0f0a' },
    7:  { fg: '#225522', bg: '#080f08' },
    8:  { fg: '#44aa44', bg: '#0a0f0a' },
    9:  { fg: '#664488', bg: '#0a0a0f' },
    10: { fg: '#aaccee', bg: '#0a0f14' },
    11: { fg: '#224466', bg: '#0a0f14' },
  },
  ownSector: { fg: '#66ff66', bg: '#0a1a0a', bold: true },
  enemySector: (id: number) => ({ fg: nationColor(id), bg: '#0a0f0a' }),
  highlightStyle: 'bg',
  highlightBg: '#1a2a1a',
  uiAccent: '#66ff66', uiBg: '#0a1a0a', uiText: '#88cc88', uiDim: '#335533',
};

// ══════════════════════════════════════════════
//  PARCHMENT — Old map / fantasy feel
// ══════════════════════════════════════════════
const PARCHMENT: ThemeDef = {
  id: 'parchment',
  name: 'Parchment',
  description: 'Fantasy parchment map style',
  icon: '📜',
  mapBg: '#1a1408',
  fogFg: '#1a1408',
  fogBg: '#0f0a04',
  cursorFg: '#ff4444',
  water:    { fg: '#4477aa', bg: '#0f1418' },
  peak:     { fg: '#eeeecc', bg: '#1a1408' },
  mountain: { fg: '#aa8866', bg: '#1a1408' },
  hill:     { fg: '#ccaa77', bg: '#1a1408' },
  flat:     { fg: '#88aa66', bg: '#1a1408' },
  vegColors: {
    0:  { fg: '#cc4400', bg: '#1a1408' },
    1:  { fg: '#ddbb66', bg: '#1a1408' },
    2:  { fg: '#aabbcc', bg: '#1a1408' },
    3:  { fg: '#998866', bg: '#1a1408' },
    4:  { fg: '#88aa66', bg: '#1a1408' },
    5:  { fg: '#66bb44', bg: '#1a1408' },
    6:  { fg: '#558833', bg: '#1a1408' },
    7:  { fg: '#337722', bg: '#1a1408' },
    8:  { fg: '#44aa33', bg: '#1a1408' },
    9:  { fg: '#886699', bg: '#1a1408' },
    10: { fg: '#aaccee', bg: '#1a1408' },
    11: { fg: '#4477aa', bg: '#1a1408' },
  },
  ownSector: { fg: '#ffddaa', bg: '#2a1e0f', bold: true },
  enemySector: (id: number) => ({ fg: nationColor(id), bg: '#1a1408' }),
  highlightStyle: 'bg',
  highlightBg: '#2a2010',
  uiAccent: '#ffcc66', uiBg: '#1a1408', uiText: '#ccaa77', uiDim: '#665533',
};

// ══════════════════════════════════════════════
//  BLUEPRINT — Engineering blueprint style
// ══════════════════════════════════════════════
const BLUEPRINT: ThemeDef = {
  id: 'blueprint',
  name: 'Blueprint',
  description: 'Engineering blueprint / cyanotype',
  icon: '📐',
  mapBg: '#0a1428',
  fogFg: '#0a1428',
  fogBg: '#081020',
  cursorFg: '#ffffff',
  water:    { fg: '#1a3355', bg: '#0a1428' },
  peak:     { fg: '#aaccff', bg: '#0a1428', bold: true },
  mountain: { fg: '#7799cc', bg: '#0a1428' },
  hill:     { fg: '#5577aa', bg: '#0a1428' },
  flat:     { fg: '#4466aa', bg: '#0a1428' },
  vegColors: {},
  ownSector: { fg: '#ffffff', bg: '#0a1428', bold: true },
  enemySector: () => ({ fg: '#88aacc', bg: '#0a1428' }),
  highlightStyle: 'bg',
  highlightBg: '#1a2844',
  uiAccent: '#88ccff', uiBg: '#0a1428', uiText: '#6699cc', uiDim: '#334466',
};

// ══════════════════════════════════════════════
//  HEATMAP — Data visualization style
// ══════════════════════════════════════════════
const HEATMAP: ThemeDef = {
  id: 'heatmap',
  name: 'Heatmap',
  description: 'Data visualization with warm colors',
  icon: '🔥',
  mapBg: '#0a0000',
  fogFg: '#0a0000',
  fogBg: '#050000',
  cursorFg: '#ffffff',
  water:    { fg: '#112244', bg: '#050008' },
  peak:     { fg: '#ff4400', bg: '#1a0800', bold: true },
  mountain: { fg: '#cc4400', bg: '#0a0000' },
  hill:     { fg: '#aa6600', bg: '#0a0000' },
  flat:     { fg: '#668800', bg: '#0a0000' },
  vegColors: {
    0:  { fg: '#ff2200', bg: '#1a0000' },
    1:  { fg: '#ff8800', bg: '#0a0000' },
    2:  { fg: '#4466aa', bg: '#0a0000' },
    3:  { fg: '#aa6600', bg: '#0a0000' },
    4:  { fg: '#88aa00', bg: '#0a0000' },
    5:  { fg: '#55cc00', bg: '#0a0000' },
    6:  { fg: '#44aa00', bg: '#0a0000' },
    7:  { fg: '#338800', bg: '#0a0000' },
    8:  { fg: '#55cc00', bg: '#0a0000' },
    9:  { fg: '#8844aa', bg: '#0a0000' },
    10: { fg: '#4488cc', bg: '#0a0000' },
    11: { fg: '#112244', bg: '#050008' },
  },
  ownSector: { fg: '#ffff00', bg: '#1a1a00', bold: true },
  enemySector: (id: number) => ({ fg: nationColor(id), bg: '#0a0000' }),
  highlightStyle: 'bg',
  highlightBg: '#2a1a00',
  uiAccent: '#ff8844', uiBg: '#1a0800', uiText: '#cc8844', uiDim: '#663300',
};

// ══════════════════════════════════════════════
//  Registry
// ══════════════════════════════════════════════
export const ALL_THEMES: ThemeDef[] = [
  CLASSIC_GREEN,
  CLASSIC_AMBER,
  CLASSIC_WHITE,
  ENHANCED,
  TACTICAL,
  PARCHMENT,
  BLUEPRINT,
  HEATMAP,
];

export function getTheme(id: string): ThemeDef {
  return ALL_THEMES.find(t => t.id === id) ?? CLASSIC_GREEN;
}

// ── Helper: get sector style from theme based on altitude ──
export function terrainStyle(theme: ThemeDef, altitude: number): SectorStyle {
  switch (altitude) {
    case 0: return theme.water;
    case 1: return theme.peak;
    case 2: return theme.mountain;
    case 3: return theme.hill;
    case 4: return theme.flat;
    default: return theme.flat;
  }
}
