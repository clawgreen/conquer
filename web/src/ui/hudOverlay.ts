// hudOverlay.ts — HTML overlay HUD replacing the canvas-rendered right panel

import { GameState } from '../state/gameState';
import {
  RACE_NAMES, CLASS_NAMES, SEASON_NAMES,
  seasonFromTurn, yearFromTurn,
} from '../types';
import { getTheme } from '../renderer/themes';

export class HudOverlay {
  private el: HTMLElement;
  private collapsed: boolean = false;

  constructor(parent: HTMLElement) {
    this.el = document.createElement('div');
    this.el.id = 'hud-overlay';
    this.el.style.cssText = `
      position: fixed; top: calc(env(safe-area-inset-top, 0px) + 4px); right: 4px; z-index: 50;
      background: rgba(0,17,0,0.9); border: 1px solid #338833; border-radius: 6px;
      padding: 6px 8px; font-family: "Courier New", monospace; font-size: 11px;
      color: #aaffaa; min-width: 120px; max-width: 150px;
      pointer-events: auto; user-select: none;
      backdrop-filter: blur(4px); -webkit-backdrop-filter: blur(4px);
    `;
    parent.appendChild(this.el);

    // Toggle collapse on click
    this.el.addEventListener('click', (e) => {
      if ((e.target as HTMLElement).tagName === 'BUTTON') return;
      this.collapsed = !this.collapsed;
      this.el.style.opacity = this.collapsed ? '0.4' : '1';
    });
  }

  update(state: GameState): void {
    if (!state.nation || !state.gameInfo || this.collapsed) {
      if (this.collapsed && state.nation) {
        this.el.innerHTML = `<span style="color:#55ff55;font-size:10px;">▶ ${state.nation.name}</span>`;
      }
      return;
    }

    const n = state.nation;
    const turn = state.gameInfo.current_turn;
    const season = seasonFromTurn(turn);
    const year = yearFromTurn(turn);
    const theme = getTheme(state.themeId);

    // Update HUD styling to match theme
    this.el.style.background = `${theme.uiBg}ee`;
    this.el.style.borderColor = theme.uiDim;
    this.el.style.color = theme.uiText;

    this.el.innerHTML = `
      <div style="color:${theme.uiAccent};font-size:11px;margin-bottom:4px;">${season} Y${year} T${turn}</div>
      <div style="color:${theme.uiAccent};font-weight:bold;">${n.name}</div>
      <div style="color:${theme.uiText};font-size:10px;">${RACE_NAMES[n.race] ?? '?'} ${CLASS_NAMES[n.class] ?? '?'}</div>
      <div style="margin:4px 0;border-top:1px solid ${theme.uiDim};padding-top:4px;">
        <div>💰 ${this.fmt(n.treasury_gold)}</div>
        <div>🌾 ${this.fmt(n.total_food)}</div>
        <div>⛏ ${this.fmt(n.metals)}</div>
        <div>💎 ${this.fmt(n.jewels)}</div>
      </div>
      <div style="border-top:1px solid ${theme.uiDim};padding-top:4px;font-size:11px;">
        <div>⚔ ${this.fmt(n.total_mil)} 👥 ${this.fmt(n.total_civ)}</div>
        <div>📍 ${n.total_sectors} scts 🏆 ${n.score ?? 0}</div>
      </div>
    `;
  }

  private fmt(n: number): string {
    if (n >= 1000000) return (n / 1000000).toFixed(1) + 'M';
    if (n >= 10000) return (n / 1000).toFixed(1) + 'K';
    return n.toLocaleString();
  }

  destroy(): void {
    this.el.remove();
  }
}
