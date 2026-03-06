// mapView.ts — MapView: renders the game map using TerminalRenderer
// T340-T348: Sector rendering, display/highlight modes, fog of war, army markers

import { TerminalRenderer } from '../renderer/terminal';
import { CURSES_COLORS } from '../renderer/colors';
import { nationFgColor, vegetationColor, altitudeColor } from '../renderer/colors';
import { GameState, getSector } from '../state/gameState';
import {
  DisplayMode, HighlightMode,
  DES_CHARS, VEG_CHARS, ELE_CHARS,
  Sector,
} from '../types';
import { getTheme, terrainStyle, ThemeDef, SectorStyle } from '../renderer/themes';
import { TileSet, TileDef, getTileset, getScaledCellSize, getCachedImage, TILESET_ASCII } from '../renderer/tilesets';

// Food value per vegetation type (from conquer-core vegfood)
const VEGFOOD = [0, 0, 0, 4, 6, 9, 7, 4, 0, 0, 0, 0];

function tofood(sector: Sector): number {
  // Simplified — real version considers race bonuses
  return VEGFOOD[sector.vegetation] ?? 0;
}

/** Get display character for a sector, matching C get_display_for() */
export function getDisplayChar(
  sector: Sector,
  dmode: DisplayMode,
  nationId: number,
  nationMark: string,
  nationMarks: Map<number, string>,
  nationRaces: Map<number, string>,
): string {
  switch (dmode) {
    case DisplayMode.Vegetation:
      return VEG_CHARS[sector.vegetation] ?? '?';

    case DisplayMode.Designation:
      if (sector.owner === 0) {
        // Unowned: show altitude if habitable, else vegetation
        if (tofood(sector) !== 0) {
          return ELE_CHARS[sector.altitude] ?? '?';
        } else {
          return VEG_CHARS[sector.vegetation] ?? '?';
        }
      } else if (nationId === 0 || sector.owner === nationId) {
        return DES_CHARS[sector.designation] ?? '?';
      } else {
        return nationMarks.get(sector.owner) ?? '*';
      }

    case DisplayMode.Contour:
      return ELE_CHARS[sector.altitude] ?? '?';

    case DisplayMode.Food: {
      const f = tofood(sector);
      if (f === 0) return VEG_CHARS[sector.vegetation] ?? '?';
      if (f < 10) return String(f);
      return '+';
    }

    case DisplayMode.Nation:
      if (sector.owner === 0) return ELE_CHARS[sector.altitude] ?? '?';
      return nationMarks.get(sector.owner) ?? '*';

    case DisplayMode.Race:
      if (sector.owner === 0) return ELE_CHARS[sector.altitude] ?? '?';
      return nationRaces.get(sector.owner) ?? '?';

    case DisplayMode.Move:
      // Simplified — would need movecost grid from server
      if (sector.altitude === 0) return '~'; // Water
      if (sector.altitude === 1) return 'X'; // Peak — impassable
      if (sector.altitude === 2) return '4'; // Mountain
      if (sector.altitude === 3) return '3'; // Hill
      return '2'; // Clear

    case DisplayMode.Defense:
      if (sector.altitude === 0) return '~';
      // Simplified defense bonus calc
      {
        let bonus = 0;
        if (sector.altitude === 2) bonus += 40; // Mountain
        else if (sector.altitude === 3) bonus += 20; // Hill
        if (sector.vegetation === 8) bonus += 30; // Jungle
        else if (sector.vegetation === 7) bonus += 20; // Forest
        else if (sector.vegetation === 6) bonus += 10; // Wood
        bonus += sector.fortress * 5;
        if (bonus < 200) return String(Math.floor(bonus / 20));
        return '+';
      }

    case DisplayMode.People:
      if (sector.altitude === 0) return '~';
      if (sector.people >= 9950) return 'X';
      if (sector.people >= 4950) return 'V';
      if (sector.people >= 950) return 'I';
      return String(Math.floor((50 + sector.people) / 100));

    case DisplayMode.Gold:
      if (sector.altitude === 0) return '~';
      if (tofood(sector) === 0) return 'X';
      if (sector.jewels >= 10) return '+';
      return String(sector.jewels);

    case DisplayMode.Metal:
      if (sector.altitude === 0) return '~';
      if (tofood(sector) === 0) return 'X';
      if (sector.metal >= 10) return '+';
      return String(sector.metal);

    case DisplayMode.Items:
      if (sector.altitude === 0) return '~';
      if (tofood(sector) === 0) return 'X';
      if (sector.trade_good < 61) return DES_CHARS[sector.designation] ?? '-';
      return '-';

    default:
      return '?';
  }
}

/** Get the foreground color for a sector character */
function getSectorFgColor(sector: Sector, dmode: DisplayMode, nationId: number): string {
  switch (dmode) {
    case DisplayMode.Vegetation:
      return vegetationColor(sector.vegetation);
    case DisplayMode.Contour:
      return altitudeColor(sector.altitude);
    case DisplayMode.Designation:
      if (sector.owner === 0) {
        if (tofood(sector) !== 0) return altitudeColor(sector.altitude);
        return vegetationColor(sector.vegetation);
      }
      if (sector.owner === nationId) return CURSES_COLORS.brightWhite;
      return nationFgColor(sector.owner);
    case DisplayMode.Nation:
    case DisplayMode.Race:
      if (sector.owner === 0) return altitudeColor(sector.altitude);
      return nationFgColor(sector.owner);
    default:
      return CURSES_COLORS.white;
  }
}

/** Check if a sector should be highlighted (standout/inverse) */
function shouldHighlight(
  sector: Sector,
  hmode: HighlightMode,
  nationId: number,
  state: GameState,
  absX: number,
  absY: number,
): boolean {
  switch (hmode) {
    case HighlightMode.Own:
      if (nationId === 0) return sector.owner !== 0;
      return sector.owner === nationId;

    case HighlightMode.Army:
      return (state.occupied[absX]?.[absY] ?? 0) !== 0;

    case HighlightMode.YourArmy:
      return state.armies.some(a =>
        a.soldiers > 0 && a.x === absX && a.y === absY
      );

    case HighlightMode.Move:
      return state.armies.some(a =>
        a.soldiers > 0 && a.movement > 0 && a.x === absX && a.y === absY
      );

    case HighlightMode.Good:
      return sector.trade_good < 61 && sector.altitude !== 0;

    case HighlightMode.None:
    default:
      return false;
  }
}

/** Get themed color style for a sector */
function getThemedSectorStyle(
  theme: ThemeDef,
  sector: Sector,
  dmode: DisplayMode,
  nationId: number,
): SectorStyle {
  // Vegetation display mode uses vegColors if defined
  if (dmode === DisplayMode.Vegetation && theme.vegColors[sector.vegetation]) {
    return theme.vegColors[sector.vegetation];
  }

  // Food, Move, Defense, People, Gold, Metal, Items — use terrain base
  if (dmode === DisplayMode.Food || dmode === DisplayMode.Move ||
      dmode === DisplayMode.Defense || dmode === DisplayMode.People ||
      dmode === DisplayMode.Gold || dmode === DisplayMode.Metal ||
      dmode === DisplayMode.Items) {
    // Use veg color if available, otherwise terrain
    if (theme.vegColors[sector.vegetation]) {
      return theme.vegColors[sector.vegetation];
    }
    return terrainStyle(theme, sector.altitude);
  }

  // Designation mode
  if (dmode === DisplayMode.Designation) {
    if (sector.owner === 0) {
      // Unowned: show terrain
      return terrainStyle(theme, sector.altitude);
    }
    if (sector.owner === nationId) {
      return theme.ownSector;
    }
    return theme.enemySector(sector.owner);
  }

  // Nation / Race mode
  if (dmode === DisplayMode.Nation || dmode === DisplayMode.Race) {
    if (sector.owner === 0) return terrainStyle(theme, sector.altitude);
    if (sector.owner === nationId) return theme.ownSector;
    return theme.enemySector(sector.owner);
  }

  // Contour mode — always terrain
  return terrainStyle(theme, sector.altitude);
}

// Number tiles for numeric display modes (food, people, gold, metal, defense, move)
function numberTile(n: number, maxChar: string = '+'): TileDef {
  const s = n >= 10 ? maxChar : String(n);
  return { type: 'emoji', value: ['0️⃣','1️⃣','2️⃣','3️⃣','4️⃣','5️⃣','6️⃣','7️⃣','8️⃣','9️⃣'][n] ?? (n >= 10 ? '🔟' : '❓') };
}

/** Get the tileset tile for a sector based on display mode — per-mode semantic mapping */
function getTileForSector(
  ts: TileSet,
  sector: Sector,
  dmode: DisplayMode,
  nationId: number,
  state: GameState,
  absX: number,
  absY: number,
): TileDef | null {
  // Char tilesets use the existing getDisplayChar path
  if (ts.tileType === 'char') return null;

  // Army overlay — always takes priority
  const hasArmy = state.armies.some(a => a.soldiers > 0 && a.x === absX && a.y === absY);
  if (hasArmy) return ts.army;

  // Per-mode mapping based on sector DATA, not display character
  switch (dmode) {
    case DisplayMode.Vegetation:
      // Show what grows here
      return ts.vegetation[sector.vegetation] ?? ts.unknown;

    case DisplayMode.Designation:
      if (sector.owner === 0) {
        // Unowned: show terrain (habitable → elevation, else vegetation)
        if (tofood(sector) !== 0) return ts.elevation[sector.altitude] ?? ts.unknown;
        return ts.vegetation[sector.vegetation] ?? ts.unknown;
      }
      // Owned: show what it's been developed into
      return ts.designation[sector.designation] ?? ts.unknown;

    case DisplayMode.Contour:
      // Show terrain height
      return ts.elevation[sector.altitude] ?? ts.unknown;

    case DisplayMode.Food: {
      if (sector.altitude === 0) return ts.elevation[0]; // water
      const food = tofood(sector);
      if (food === 0) return ts.vegetation[sector.vegetation] ?? ts.unknown;
      return numberTile(food);
    }

    case DisplayMode.Nation:
      if (sector.owner === 0) return ts.elevation[sector.altitude] ?? ts.unknown;
      // For nations, show their mark — fall back to a colored marker
      return { type: 'emoji', value: sector.owner === nationId ? '🟢' : '🔴' };

    case DisplayMode.Race:
      if (sector.owner === 0) return ts.elevation[sector.altitude] ?? ts.unknown;
      // Show race icon
      const raceIcons: Record<string, string> = {
        'H': '🧑', 'O': '👹', 'E': '🧝', 'D': '⛏️', 'L': '🦎',
        'P': '🏴‍☠️', 'S': '🗡️', 'N': '🐪', 'G': '👁️',
      };
      const race = state.publicNations.find(n => n.nation_id === sector.owner)?.race ?? '?';
      return { type: 'emoji', value: raceIcons[race] ?? '❓' };

    case DisplayMode.Move: {
      if (sector.altitude === 0) return ts.elevation[0]; // water
      if (sector.altitude === 1) return { type: 'emoji', value: '🚫' }; // impassable peak
      // Movement cost by altitude
      const costs = [0, 0, 4, 3, 2]; // rough approx for human
      return numberTile(costs[sector.altitude] ?? 2);
    }

    case DisplayMode.Defense: {
      if (sector.altitude === 0) return ts.elevation[0];
      let bonus = 0;
      if (sector.altitude === 2) bonus += 40;
      else if (sector.altitude === 3) bonus += 20;
      if (sector.vegetation === 8) bonus += 30; // jungle
      else if (sector.vegetation === 7) bonus += 20; // forest
      else if (sector.vegetation === 6) bonus += 10; // wood
      bonus += sector.fortress * 5;
      return numberTile(Math.min(9, Math.floor(bonus / 20)));
    }

    case DisplayMode.People: {
      if (sector.altitude === 0) return ts.elevation[0];
      if (sector.people >= 9950) return { type: 'emoji', value: '🏙️' };
      if (sector.people >= 4950) return { type: 'emoji', value: '🏘️' };
      if (sector.people >= 950) return { type: 'emoji', value: '🏠' };
      const popDigit = Math.floor((50 + sector.people) / 100);
      return numberTile(Math.min(9, popDigit));
    }

    case DisplayMode.Gold: {
      if (sector.altitude === 0) return ts.elevation[0];
      if (tofood(sector) === 0) return { type: 'emoji', value: '🚫' };
      if (sector.jewels >= 10) return { type: 'emoji', value: '💎' };
      return numberTile(sector.jewels);
    }

    case DisplayMode.Metal: {
      if (sector.altitude === 0) return ts.elevation[0];
      if (tofood(sector) === 0) return { type: 'emoji', value: '🚫' };
      if (sector.metal >= 10) return { type: 'emoji', value: '⛏️' };
      return numberTile(sector.metal);
    }

    case DisplayMode.Items: {
      if (sector.altitude === 0) return ts.elevation[0];
      if (tofood(sector) === 0) return { type: 'emoji', value: '🚫' };
      if (sector.trade_good < 61) return ts.designation[sector.designation] ?? ts.unknown;
      return { type: 'emoji', value: '📦' };
    }

    default:
      return ts.elevation[sector.altitude] ?? ts.unknown;
  }
}

/** Calculate viewport screen size based on terminal dimensions */
export function screenSize(term: TerminalRenderer): { screenX: number; screenY: number } {
  // 2 chars per sector: character + padding space (gives roughly square map tiles)
  const screenX = Math.floor(term.cols / 2);
  const screenY = term.rows - 3;
  return { screenX: Math.max(1, screenX), screenY: Math.max(1, screenY) };
}

/** Render the game map onto the terminal renderer */
export function renderMap(term: TerminalRenderer, state: GameState): void {
  if (!state.mapData || !state.nation) return;

  const ts = getTileset(state.tilesetId ?? 'ascii');
  const useDirectCanvas = ts.tileType !== 'char';

  const { screenX, screenY } = screenSize(term);
  const nationId = state.nationId ?? 0;

  // Build nation marks/races lookup
  const nationMarks = new Map<number, string>();
  const nationRaces = new Map<number, string>();
  for (const n of state.publicNations) {
    nationMarks.set(n.nation_id, n.mark);
    nationRaces.set(n.nation_id, n.race);
  }
  nationMarks.set(nationId, state.nation.mark);
  nationRaces.set(nationId, state.nation.race);

  // For emoji/image tilesets, render directly to canvas context
  if (useDirectCanvas) {
    renderTilesetMap(term, state, ts, screenX, screenY, nationId, nationMarks, nationRaces);
    return;
  }

  // Classic char-based rendering
  const theme = getTheme(state.themeId);
  for (let sx = 0; sx < screenX; sx++) {
    for (let sy = 0; sy < screenY; sy++) {
      const absX = sx + state.xOffset;
      const absY = sy + state.yOffset;
      const sector = getSector(state, absX, absY);
      const colPos = sx * 2;

      if (!sector) {
        term.setCell(colPos, sy, { ch: ' ', fg: theme.fogFg, bg: theme.fogBg });
        term.setCell(colPos + 1, sy, { ch: ' ', fg: theme.fogFg, bg: theme.fogBg });
        continue;
      }

      const ch = getDisplayChar(
        sector, state.displayMode, nationId,
        state.nation.mark, nationMarks, nationRaces,
      );
      const isHighlighted = shouldHighlight(sector, state.highlightMode, nationId, state, absX, absY);
      const style = getThemedSectorStyle(theme, sector, state.displayMode, nationId);

      if (isHighlighted) {
        if (theme.highlightStyle === 'inverse') {
          term.setCell(colPos, sy, { ch, fg: style.fg, bg: style.bg, inverse: true, bold: style.bold });
          term.setCell(colPos + 1, sy, { ch: ' ', fg: style.bg, bg: style.fg });
        } else {
          term.setCell(colPos, sy, { ch, fg: style.fg, bg: theme.highlightBg, bold: style.bold });
          term.setCell(colPos + 1, sy, { ch: ' ', fg: theme.highlightBg, bg: theme.highlightBg });
        }
      } else {
        term.setCell(colPos, sy, { ch, fg: style.fg, bg: style.bg, bold: style.bold, inverse: false });
        term.setCell(colPos + 1, sy, { ch: ' ', fg: style.bg, bg: style.bg });
      }
    }
  }
}

/** Draw a blinking cursor overlay on the selected cell */
let cursorBlinkOn = true;
setInterval(() => { cursorBlinkOn = !cursorBlinkOn; }, 500);

export function renderTilesetCursor(
  ctx: CanvasRenderingContext2D,
  state: GameState,
  ts: TileSet,
  fontSize: number,
): void {
  const { cw, ch } = getScaledCellSize(ts, fontSize);
  const sx = state.cursorX;
  const sy = state.cursorY;
  const px = sx * cw;
  const py = sy * ch;

  // Semi-transparent green overlay that blinks
  const alpha = cursorBlinkOn ? 0.35 : 0.15;
  ctx.fillStyle = `rgba(85, 255, 85, ${alpha})`;
  ctx.fillRect(px, py, cw, ch);

  // Green border always visible
  ctx.strokeStyle = `rgba(85, 255, 85, ${cursorBlinkOn ? 0.9 : 0.5})`;
  ctx.lineWidth = 2;
  ctx.strokeRect(px + 1, py + 1, cw - 2, ch - 2);
}

/** Render map using emoji or image tileset directly to canvas */
function renderTilesetMap(
  term: TerminalRenderer,
  state: GameState,
  ts: TileSet,
  screenX: number,
  screenY: number,
  nationId: number,
  nationMarks: Map<number, string>,
  nationRaces: Map<number, string>,
): void {
  const ctx = term.getContext();
  if (!ctx) return;

  const theme = getTheme(state.themeId);
  const { cw, ch } = getScaledCellSize(ts, term.fontSize);

  // How many tiles fit in the canvas
  const canvasW = ctx.canvas.width;
  const canvasH = ctx.canvas.height;
  const tilesX = Math.min(screenX, Math.floor(canvasW / cw));
  const tilesY = Math.min(screenY, Math.floor(canvasH / ch));

  for (let sx = 0; sx < tilesX; sx++) {
    for (let sy = 0; sy < tilesY; sy++) {
      const absX = sx + state.xOffset;
      const absY = sy + state.yOffset;
      const sector = getSector(state, absX, absY);

      const px = sx * cw;
      const py = sy * ch;

      if (!sector) {
        // Fog of war — just black
        ctx.fillStyle = '#000';
        ctx.fillRect(px, py, cw, ch);
        continue;
      }

      const isHighlighted = shouldHighlight(sector, state.highlightMode, nationId, state, absX, absY);
      const tile = getTileForSector(ts, sector, state.displayMode, nationId, state, absX, absY) ?? ts.unknown;

      // Background
      const bg = tile.bg ?? theme.mapBg;
      ctx.fillStyle = isHighlighted ? (theme.highlightBg ?? '#333') : bg;
      ctx.fillRect(px, py, cw, ch);

      if (tile.type === 'emoji') {
        ctx.font = `${Math.max(10, ch - 4)}px serif`;
        ctx.textBaseline = 'middle';
        ctx.textAlign = 'center';
        ctx.fillText(tile.value, px + cw / 2, py + ch / 2);
      } else if (tile.type === 'image') {
        const img = getCachedImage(tile.value);
        if (img) {
          ctx.drawImage(img, px, py, cw, ch);
        } else {
          // Fallback: draw placeholder
          ctx.fillStyle = '#333';
          ctx.fillRect(px + 2, py + 2, cw - 4, ch - 4);
        }
      }

      // Highlight border
      if (isHighlighted) {
        ctx.strokeStyle = theme.highlightBg ?? '#ff0';
        ctx.lineWidth = 1;
        ctx.strokeRect(px + 0.5, py + 0.5, cw - 1, ch - 1);
      }
    }
  }
}
