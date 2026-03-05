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

/** Calculate viewport screen size based on terminal dimensions */
export function screenSize(term: TerminalRenderer): { screenX: number; screenY: number } {
  // Map takes left portion, right 10 cols for side panel
  // Each map cell uses 2 columns (matching C: 2*x), bottom 5 rows for status
  const screenX = Math.floor((term.cols - 10) / 2);
  const screenY = term.rows - 5;
  return { screenX: Math.max(1, screenX), screenY: Math.max(1, screenY) };
}

/** Render the game map onto the terminal renderer */
export function renderMap(term: TerminalRenderer, state: GameState): void {
  if (!state.mapData || !state.nation) return;

  const { screenX, screenY } = screenSize(term);
  const nationId = state.nationId ?? 0;

  // Build nation marks/races lookup
  const nationMarks = new Map<number, string>();
  const nationRaces = new Map<number, string>();
  for (const n of state.publicNations) {
    nationMarks.set(n.nation_id, n.mark);
    nationRaces.set(n.nation_id, n.race);
  }
  // Add own nation
  nationMarks.set(nationId, state.nation.mark);
  nationRaces.set(nationId, state.nation.race);

  for (let sx = 0; sx < screenX; sx++) {
    for (let sy = 0; sy < screenY; sy++) {
      const absX = sx + state.xOffset;
      const absY = sy + state.yOffset;
      const sector = getSector(state, absX, absY);

      const colPos = sx * 2; // 2 columns per cell (matching C)

      if (!sector) {
        // Fog of war — dark/blank
        term.setCell(colPos, sy, { ch: ' ', fg: CURSES_COLORS.black, bg: CURSES_COLORS.black });
        term.setCell(colPos + 1, sy, { ch: ' ', fg: CURSES_COLORS.black, bg: CURSES_COLORS.black });
        continue;
      }

      const ch = getDisplayChar(
        sector, state.displayMode, nationId,
        state.nation.mark, nationMarks, nationRaces,
      );
      const fg = getSectorFgColor(sector, state.displayMode, nationId);
      const inverse = shouldHighlight(sector, state.highlightMode, nationId, state, absX, absY);

      term.setCell(colPos, sy, {
        ch,
        fg,
        bg: CURSES_COLORS.black,
        inverse,
        bold: false,
      });
      // Second column — blank or second display
      term.setCell(colPos + 1, sy, {
        ch: ' ',
        fg: CURSES_COLORS.black,
        bg: inverse ? fg : CURSES_COLORS.black,
      });
    }
  }
}
