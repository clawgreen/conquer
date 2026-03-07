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
  set fontSize(px: number) { this.container.style.fontSize = `${px}px`; }

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
      const isOwn = sector.owner === state.nationId;
      const armiesHere = state.armies.filter(a => a.soldiers > 0 && a.x === absX && a.y === absY);
      const armyDetails = armiesHere.map(a => {
        const status = ARMY_STATUS_NAMES[a.status] ?? '?';
        return `<div class="stat-row" style="padding-left:12px;font-size:12px;">` +
          `<span>A${a.index}: ${a.soldiers} ${status}</span>` +
          `<span>mv:${a.movement}</span></div>`;
      }).join('');
      const naviesHere = state.navies.filter(nv => nv.x === absX && nv.y === absY);
      const navyDetails = naviesHere.map(nv =>
        `<div class="stat-row" style="padding-left:12px;font-size:12px;">` +
        `<span>F${nv.index}: ${nv.warships}W ${nv.merchant}M ${nv.galleys}G</span></div>`
      ).join('');

      sectorHtml = `
        <div class="stat-row"><span class="stat-label">Pos</span><span class="stat-value">(${absX},${absY})</span></div>
        <div class="stat-row"><span class="stat-label">Terrain</span><span class="stat-value">${altName}</span></div>
        <div class="stat-row"><span class="stat-label">Veg</span><span class="stat-value">${vegName}</span></div>
        <div class="stat-row"><span class="stat-label">Type</span><span class="stat-value">${desName}</span></div>
        <div class="stat-row"><span class="stat-label">Owner</span><span class="stat-value">${isOwn ? '★ ' : ''}${ownerStr}</span></div>
        <div class="stat-row"><span class="stat-label">Pop</span><span class="stat-value">${sector.people.toLocaleString()}</span></div>
        ${sector.fortress > 0 ? `<div class="stat-row"><span class="stat-label">Fort</span><span class="stat-value">Lv ${sector.fortress}</span></div>` : ''}
        ${sector.metal > 0 ? `<div class="stat-row"><span class="stat-label">⛏ Metal</span><span class="stat-value">${sector.metal}</span></div>` : ''}
        ${sector.jewels > 0 ? `<div class="stat-row"><span class="stat-label">💎 Jewels</span><span class="stat-value">${sector.jewels}</span></div>` : ''}
        ${sector.trade_good > 0 ? `<div class="stat-row"><span class="stat-label">📦 Trade</span><span class="stat-value">${sector.trade_good}</span></div>` : ''}
        ${armyDetails ? `<div class="stat-row" style="opacity:0.7;font-size:11px;">⚔ Armies:</div>${armyDetails}` : ''}
        ${navyDetails ? `<div class="stat-row" style="opacity:0.7;font-size:11px;">🚢 Navies:</div>${navyDetails}` : ''}
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

      <div class="stat-section" style="border-color:${t.sidebarBorder};">
        <div class="stat-section-title" style="color:${t.sidebarDim};">Scoreboard</div>
        ${this.renderScores(state, t)}
      </div>
    `;
  }

  private renderScores(state: GameState, t: ReturnType<typeof getUiTheme>): string {
    if (!state.scores || state.scores.length === 0) {
      // Use publicNations as fallback
      const nations = state.publicNations
        .filter(n => n.score != null && n.score > 0)
        .sort((a, b) => (b.score ?? 0) - (a.score ?? 0))
        .slice(0, 10);
      if (nations.length === 0) return '<div class="stat-row" style="opacity:0.4;">No scores yet</div>';
      return nations.map((n, i) => {
        const isYou = n.nation_id === state.nationId;
        return `<div class="stat-row" style="${isYou ? `color:${t.sidebarAccent};font-weight:bold;` : ''}">
          <span>${i + 1}. ${n.name}</span><span>${n.score ?? 0}</span>
        </div>`;
      }).join('');
    }
    return state.scores
      .filter(s => s.score > 0)
      .sort((a, b) => b.score - a.score)
      .slice(0, 10)
      .map((s, i) => {
        const isYou = s.nation_id === state.nationId;
        return `<div class="stat-row" style="${isYou ? `color:${t.sidebarAccent};font-weight:bold;` : ''}">
          <span>${i + 1}. ${s.name}</span><span>${s.score}</span>
        </div>`;
      }).join('') || '<div class="stat-row" style="opacity:0.4;">No scores yet</div>';
  }

  destroy(): void {}
}
