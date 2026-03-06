// statsSidebar.ts — Right sidebar: nation stats, armies, sector info

import { getUiTheme } from './uiThemes';
import { GameState, getSector } from '../state/gameState';
import {
  RACE_NAMES, CLASS_NAMES, DESIGNATION_NAMES, ALTITUDE_NAMES,
  VEGETATION_NAMES, ARMY_STATUS_NAMES,
  seasonFromTurn, yearFromTurn,
} from '../types';

export class StatsSidebar {
  private container: HTMLElement;
  private _themeId = 'terminal';

  constructor(container: HTMLElement) {
    this.container = container;
  }

  set themeId(id: string) { this._themeId = id; }

  update(state: GameState): void {
    if (!state.nation || !state.gameInfo) {
      this.container.innerHTML = '<div style="padding:8px;opacity:0.5;">Loading...</div>';
      return;
    }

    const t = getUiTheme(this._themeId);
    const n = state.nation;
    const turn = state.gameInfo.current_turn;
    const season = seasonFromTurn(turn);
    const year = yearFromTurn(turn);

    // Sector under cursor
    const absX = state.cursorX + state.xOffset;
    const absY = state.cursorY + state.yOffset;
    const sector = getSector(state, absX, absY);

    let sectorHtml = `<div class="stat-row"><span style="opacity:0.5;">(${absX},${absY}) Fog of war</span></div>`;
    if (sector) {
      const desName = DESIGNATION_NAMES[sector.designation] ?? '?';
      const altName = ALTITUDE_NAMES[sector.altitude] ?? '?';
      const vegName = VEGETATION_NAMES[sector.vegetation] ?? '?';
      const ownerStr = sector.owner === 0 ? 'Unowned' :
        (state.publicNations.find(nn => nn.nation_id === sector.owner)?.name ?? `N${sector.owner}`);
      const armiesHere = state.armies.filter(a => a.soldiers > 0 && a.x === absX && a.y === absY);
      const armyStr = armiesHere.map(a => `A${a.index}:${a.soldiers}`).join(' ');

      sectorHtml = `
        <div class="stat-row"><span class="stat-label">Pos</span><span class="stat-value">(${absX},${absY})</span></div>
        <div class="stat-row"><span class="stat-label">Terrain</span><span class="stat-value">${altName}</span></div>
        <div class="stat-row"><span class="stat-label">Veg</span><span class="stat-value">${vegName}</span></div>
        <div class="stat-row"><span class="stat-label">Type</span><span class="stat-value">${desName}</span></div>
        <div class="stat-row"><span class="stat-label">Owner</span><span class="stat-value">${ownerStr}</span></div>
        <div class="stat-row"><span class="stat-label">Pop</span><span class="stat-value">${sector.people.toLocaleString()}</span></div>
        <div class="stat-row"><span class="stat-label">Fort</span><span class="stat-value">${sector.fortress}</span></div>
        ${sector.metal > 0 ? `<div class="stat-row"><span class="stat-label">Metal</span><span class="stat-value">${sector.metal}</span></div>` : ''}
        ${sector.jewels > 0 ? `<div class="stat-row"><span class="stat-label">Jewels</span><span class="stat-value">${sector.jewels}</span></div>` : ''}
        ${armyStr ? `<div class="stat-row"><span class="stat-label">Armies</span><span class="stat-value">${armyStr}</span></div>` : ''}
      `;
    }

    // Army list
    const activeArmies = state.armies.filter(a => a.soldiers > 0);
    const armyRows = activeArmies.slice(0, 15).map((a, i) => {
      const sel = i === state.selectedArmy;
      const status = ARMY_STATUS_NAMES[a.status] ?? '?';
      return `<div class="stat-row" style="${sel ? `color:${t.sidebarAccent};font-weight:bold;` : ''}">
        <span>${sel ? '▸' : ' '}A${a.index}</span>
        <span>${a.soldiers} ${status.substring(0, 4)} (${a.x},${a.y})</span>
      </div>`;
    }).join('');

    this.container.innerHTML = `
      <div class="stat-section" style="border-color:${t.sidebarBorder};">
        <div class="stat-section-title" style="color:${t.sidebarAccent};">${season} Year ${year} — Turn ${turn}</div>
        <div class="stat-row" style="font-weight:bold;font-size:14px;padding:2px 8px;">
          <span>${n.name}</span>
        </div>
        <div class="stat-row" style="opacity:0.7;">
          <span>${RACE_NAMES[n.race] ?? '?'} ${CLASS_NAMES[n.class] ?? '?'}</span>
          <span>Ldr: ${n.leader}</span>
        </div>
      </div>

      <div class="stat-section" style="border-color:${t.sidebarBorder};">
        <div class="stat-section-title" style="color:${t.sidebarDim};">Resources</div>
        <div class="stat-row"><span class="stat-label">💰 Gold</span><span class="stat-value">${n.treasury_gold.toLocaleString()}</span></div>
        <div class="stat-row"><span class="stat-label">🌾 Food</span><span class="stat-value">${n.total_food.toLocaleString()}</span></div>
        <div class="stat-row"><span class="stat-label">⛏ Metal</span><span class="stat-value">${n.metals.toLocaleString()}</span></div>
        <div class="stat-row"><span class="stat-label">💎 Jewels</span><span class="stat-value">${n.jewels.toLocaleString()}</span></div>
      </div>

      <div class="stat-section" style="border-color:${t.sidebarBorder};">
        <div class="stat-section-title" style="color:${t.sidebarDim};">Military & Population</div>
        <div class="stat-row"><span class="stat-label">⚔ Military</span><span class="stat-value">${n.total_mil.toLocaleString()}</span></div>
        <div class="stat-row"><span class="stat-label">👥 Civilians</span><span class="stat-value">${n.total_civ.toLocaleString()}</span></div>
        <div class="stat-row"><span class="stat-label">📍 Sectors</span><span class="stat-value">${n.total_sectors}</span></div>
        <div class="stat-row"><span class="stat-label">🏆 Score</span><span class="stat-value">${n.score ?? 0}</span></div>
        <div class="stat-row"><span class="stat-label">Atk+</span><span class="stat-value">${n.attack_plus}</span></div>
        <div class="stat-row"><span class="stat-label">Def+</span><span class="stat-value">${n.defense_plus}</span></div>
        <div class="stat-row"><span class="stat-label">Spell</span><span class="stat-value">${n.spell_points}</span></div>
      </div>

      <div class="stat-section" style="border-color:${t.sidebarBorder};">
        <div class="stat-section-title" style="color:${t.sidebarDim};">Cursor — Sector</div>
        ${sectorHtml}
      </div>

      <div class="stat-section" style="border-color:${t.sidebarBorder};">
        <div class="stat-section-title" style="color:${t.sidebarDim};">Armies (${activeArmies.length})</div>
        ${armyRows || '<div class="stat-row" style="opacity:0.4;">No armies</div>'}
      </div>

      ${state.isDone ? `<div style="text-align:center;padding:8px;color:${t.sidebarAccent};">✓ TURN ENDED</div>` : ''}
    `;
  }

  destroy(): void {}
}
