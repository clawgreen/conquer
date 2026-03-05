// sidePanel.ts — Right side panel and bottom panel matching original curses UI
// T349-T353: Nation info, army/navy lists, sector detail, command prompt

import { TerminalRenderer } from '../renderer/terminal';
import { CURSES_COLORS } from '../renderer/colors';
import { GameState, getSector } from '../state/gameState';
import {
  RACE_NAMES, CLASS_NAMES, DESIGNATION_NAMES, VEGETATION_NAMES,
  ALTITUDE_NAMES, ARMY_STATUS_NAMES, SEASON_NAMES,
  seasonFromTurn, yearFromTurn,
  DisplayMode, HighlightMode,
} from '../types';
import { screenSize } from '../game/mapView';

const DISPLAY_MODE_NAMES: Record<number, string> = {
  [DisplayMode.Vegetation]: 'VEG',
  [DisplayMode.Designation]: 'DES',
  [DisplayMode.Contour]: 'CNT',
  [DisplayMode.Food]: 'FOD',
  [DisplayMode.Nation]: 'NTN',
  [DisplayMode.Race]: 'RAC',
  [DisplayMode.Move]: 'MOV',
  [DisplayMode.Defense]: 'DEF',
  [DisplayMode.People]: 'POP',
  [DisplayMode.Gold]: 'GLD',
  [DisplayMode.Metal]: 'MTL',
  [DisplayMode.Items]: 'ITM',
};

const HIGHLIGHT_MODE_NAMES: Record<number, string> = {
  [HighlightMode.Own]: 'OWN',
  [HighlightMode.Army]: 'ARM',
  [HighlightMode.None]: 'NON',
  [HighlightMode.YourArmy]: 'YRS',
  [HighlightMode.Move]: 'MVL',
  [HighlightMode.Good]: 'TRD',
};

/** Render the right side panel — nation info */
export function renderSidePanel(term: TerminalRenderer, state: GameState): void {
  if (!state.nation || !state.gameInfo) return;

  const { screenX } = screenSize(term);
  const startCol = screenX * 2 + 1; // Right side starts after map
  const nation = state.nation;
  const turn = state.gameInfo.current_turn;

  let row = 0;
  const w = (text: string, fg?: string) => {
    if (row < term.rows - 5) {
      term.writeStr(startCol, row, text.padEnd(term.cols - startCol), fg ?? CURSES_COLORS.white);
      row++;
    }
  };

  // Season / Year
  w(`${seasonFromTurn(turn)} Year ${yearFromTurn(turn)}`, CURSES_COLORS.brightYellow);
  w(`Turn ${turn}`, CURSES_COLORS.brightYellow);
  w('');

  // Nation info
  w(nation.name, CURSES_COLORS.brightWhite);
  w(`${RACE_NAMES[nation.race] ?? '?'} ${CLASS_NAMES[nation.class] ?? '?'}`, CURSES_COLORS.brightCyan);
  w(`Leader: ${nation.leader}`);
  w('');

  // Resources
  w(`Gold:  ${nation.treasury_gold.toLocaleString()}`, CURSES_COLORS.brightYellow);
  w(`Food:  ${nation.total_food.toLocaleString()}`, CURSES_COLORS.brightGreen);
  w(`Metal: ${nation.metals.toLocaleString()}`, CURSES_COLORS.white);
  w(`Jewel: ${nation.jewels.toLocaleString()}`, CURSES_COLORS.brightMagenta);
  w('');

  // Military
  w(`Mil:  ${nation.total_mil.toLocaleString()}`, CURSES_COLORS.brightRed);
  w(`Civ:  ${nation.total_civ.toLocaleString()}`, CURSES_COLORS.brightGreen);
  w(`Scts: ${nation.total_sectors}`, CURSES_COLORS.white);
  w(`Atk+: ${nation.attack_plus}  Def+: ${nation.defense_plus}`);
  w(`Spell: ${nation.spell_points}`);
  w('');

  // Display/Highlight mode
  w(`D:${DISPLAY_MODE_NAMES[state.displayMode] ?? '?'} H:${HIGHLIGHT_MODE_NAMES[state.highlightMode] ?? '?'}`, CURSES_COLORS.brightCyan);
  w('');

  // Army list (abbreviated)
  if (state.armyOrNavy === 'army') {
    w('--- ARMIES ---', CURSES_COLORS.brightWhite);
    const activeArmies = state.armies.filter(a => a.soldiers > 0);
    const startIdx = Math.max(0, state.selectedArmy - 5);
    const endIdx = Math.min(activeArmies.length, startIdx + (term.rows - 5 - row - 2));
    for (let i = startIdx; i < endIdx; i++) {
      const a = activeArmies[i];
      const sel = i === state.selectedArmy ? '>' : ' ';
      const status = ARMY_STATUS_NAMES[a.status] ?? `G${a.status - 17}`;
      const fg = i === state.selectedArmy ? CURSES_COLORS.brightYellow : CURSES_COLORS.white;
      w(`${sel}${String(a.index).padStart(2)} ${String(a.soldiers).padStart(5)} ${status.substring(0, 4).padEnd(4)} ${a.x},${a.y} m${a.movement}`, fg);
    }
  } else {
    w('--- NAVIES ---', CURSES_COLORS.brightWhite);
    const activeNavies = state.navies.filter(n => n.warships > 0 || n.merchant > 0 || n.galleys > 0);
    for (let i = 0; i < Math.min(activeNavies.length, term.rows - 5 - row - 2); i++) {
      const n = activeNavies[i];
      const sel = i === state.selectedNavy ? '>' : ' ';
      const fg = i === state.selectedNavy ? CURSES_COLORS.brightYellow : CURSES_COLORS.white;
      w(`${sel}${String(n.index).padStart(2)} W${n.warships} M${n.merchant} G${n.galleys} ${n.x},${n.y}`, fg);
    }
  }
}

/** Render the bottom panel — sector detail + status */
export function renderBottomPanel(term: TerminalRenderer, state: GameState, statusMessage: string): void {
  if (!state.mapData) return;

  const bottomStart = term.rows - 5;
  const absX = state.cursorX + state.xOffset;
  const absY = state.cursorY + state.yOffset;
  const sector = getSector(state, absX, absY);

  // Clear bottom area
  for (let row = bottomStart; row < term.rows; row++) {
    term.clearRow(row);
  }

  if (sector) {
    const desName = DESIGNATION_NAMES[sector.designation] ?? 'UNKNOWN';
    const vegName = VEGETATION_NAMES[sector.vegetation] ?? 'UNKNOWN';
    const altName = ALTITUDE_NAMES[sector.altitude] ?? 'UNKNOWN';

    const ownerStr = sector.owner === 0 ? 'Unowned' :
      (state.publicNations.find(n => n.nation_id === sector.owner)?.name ?? `Nation ${sector.owner}`);

    term.writeStr(0, bottomStart, `Sector (${absX},${absY}): ${desName}  ${altName}  ${vegName}`, CURSES_COLORS.brightWhite);
    term.writeStr(0, bottomStart + 1, `Owner: ${ownerStr}  Pop: ${sector.people}  Fort: ${sector.fortress}  Metal: ${sector.metal}  Jewel: ${sector.jewels}`, CURSES_COLORS.white);

    // Show armies at this location
    const armiesHere = state.armies.filter(a => a.soldiers > 0 && a.x === absX && a.y === absY);
    if (armiesHere.length > 0) {
      const armyStr = armiesHere.map(a => `A${a.index}:${a.soldiers}`).join(' ');
      term.writeStr(0, bottomStart + 2, `Armies: ${armyStr}`, CURSES_COLORS.brightYellow);
    }
  } else {
    term.writeStr(0, bottomStart, `(${absX},${absY}): Not visible (fog of war)`, CURSES_COLORS.brightBlack);
  }

  // Status message / command prompt
  if (statusMessage) {
    term.writeStr(0, bottomStart + 3, statusMessage, CURSES_COLORS.brightGreen);
  }

  // Waiting indicator
  if (state.isDone) {
    term.writeStr(0, bottomStart + 4, '[ DONE — waiting for other players ]', CURSES_COLORS.brightYellow);
  }

  // Notifications
  if (state.notifications.length > 0) {
    const latest = state.notifications[state.notifications.length - 1];
    term.writeStr(0, bottomStart + 4, latest, CURSES_COLORS.brightRed);
  }
}
