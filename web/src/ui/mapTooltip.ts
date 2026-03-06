// mapTooltip.ts — Hover tooltip showing sector info per display mode

import { GameState, getSector } from '../state/gameState';
import { DisplayMode, Sector } from '../types';
import { getTileset, getScaledCellSize } from '../renderer/tilesets';

const ELEV_NAMES = ['Water', 'Peak', 'Mountain', 'Hill', 'Flat'];
const VEG_NAMES = ['Volcano', 'Desert', 'Tundra', 'Barren', 'Light Veg', 'Good Land', 'Wood', 'Forest', 'Jungle', 'Swamp', 'Ice', 'None'];
const DES_NAMES = ['Town', 'City', 'Mine', 'Farm', 'Devastated', 'Gold Mine', 'Fort', 'Ruin', 'Stockade', 'Capitol', 'Special', 'Lumber Yard', 'Blacksmith', 'Road', 'Mill', 'Granary', 'Church', 'University', 'Undesignated', 'Base Camp'];
const RACE_NAMES: Record<string, string> = { 'H': 'Human', 'O': 'Orc', 'E': 'Elf', 'D': 'Dwarf', 'L': 'Lizard', 'P': 'Pirate', 'S': 'Savage', 'N': 'Nomad', 'G': 'God' };
const VEGFOOD = [0, 0, 0, 4, 6, 9, 7, 4, 0, 0, 0, 0];

export class MapTooltip {
  private el: HTMLDivElement;
  private canvas: HTMLCanvasElement;
  private visible = false;
  private _getCellSize: (() => { cw: number; ch: number }) | null = null;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.el = document.createElement('div');
    this.el.id = 'map-tooltip';
    this.el.style.cssText = `
      position: fixed; z-index: 100; pointer-events: none;
      background: rgba(0,10,0,0.92); color: #55ff55; border: 1px solid #338833;
      border-radius: 4px; padding: 4px 8px; font-family: "Courier New", monospace;
      font-size: 11px; white-space: nowrap; display: none;
      box-shadow: 0 2px 8px rgba(0,0,0,0.5);
    `;
    document.body.appendChild(this.el);

    canvas.addEventListener('mousemove', (e) => this.onMove(e));
    canvas.addEventListener('mouseleave', () => this.hide());
  }

  private state: GameState | null = null;

  /** Call each frame with current state */
  setState(state: GameState): void {
    this.state = state;
  }

  /** Provide a callback that returns actual cell dimensions from the renderer */
  setCellSizeProvider(fn: () => { cw: number; ch: number }): void {
    this._getCellSize = fn;
  }

  private onMove(e: MouseEvent): void {
    if (!this.state) return;

    const rect = this.canvas.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;

    // Use actual cell dimensions from the renderer
    let cw: number, ch: number;
    if (this._getCellSize) {
      const size = this._getCellSize();
      cw = size.cw;
      ch = size.ch;
    } else {
      // Fallback (shouldn't happen)
      const fontSize = parseInt(localStorage.getItem('conquer_font_size') ?? '14');
      cw = Math.round(fontSize * 0.6) * 2;
      ch = Math.round(fontSize * 1.2);
    }

    const sx = Math.floor(mx / cw);
    const sy = Math.floor(my / ch);
    const absX = sx + this.state.xOffset;
    const absY = sy + this.state.yOffset;

    const sector = getSector(this.state, absX, absY);
    if (!sector) {
      this.el.innerHTML = `<span style="color:#888;">(${absX},${absY}) Fog of war</span>`;
    } else {
      this.el.innerHTML = this.buildTooltip(sector, this.state, absX, absY);
    }

    // Position tooltip near mouse
    this.el.style.left = `${e.clientX + 12}px`;
    this.el.style.top = `${e.clientY + 12}px`;
    this.el.style.display = 'block';
    this.visible = true;

    // Keep tooltip on screen
    const tipRect = this.el.getBoundingClientRect();
    if (tipRect.right > window.innerWidth) {
      this.el.style.left = `${e.clientX - tipRect.width - 8}px`;
    }
    if (tipRect.bottom > window.innerHeight) {
      this.el.style.top = `${e.clientY - tipRect.height - 8}px`;
    }
  }

  private hide(): void {
    this.el.style.display = 'none';
    this.visible = false;
  }

  private buildTooltip(sector: Sector, state: GameState, absX: number, absY: number): string {
    const dmode = state.displayMode;
    const nationId = state.nationId ?? 0;
    const ownerName = sector.owner === 0 ? 'Unowned' :
      (state.publicNations.find(n => n.nation_id === sector.owner)?.name ?? `Nation ${sector.owner}`);

    // Common header
    let html = `<div style="color:#88ff88;margin-bottom:2px;">(${absX},${absY}) ${ownerName}</div>`;

    // Mode-specific detail
    switch (dmode) {
      case DisplayMode.Vegetation:
        html += `<div>🌿 <b>${VEG_NAMES[sector.vegetation] ?? '?'}</b></div>`;
        html += `<div style="color:#888;">Food value: ${VEGFOOD[sector.vegetation] ?? 0}</div>`;
        break;

      case DisplayMode.Designation:
        if (sector.owner !== 0) {
          html += `<div>🏗️ <b>${DES_NAMES[sector.designation] ?? '?'}</b></div>`;
        } else {
          html += `<div>⛰️ ${ELEV_NAMES[sector.altitude] ?? '?'} — ${VEG_NAMES[sector.vegetation] ?? '?'}</div>`;
        }
        break;

      case DisplayMode.Contour:
        html += `<div>⛰️ <b>${ELEV_NAMES[sector.altitude] ?? '?'}</b></div>`;
        break;

      case DisplayMode.Food: {
        const food = VEGFOOD[sector.vegetation] ?? 0;
        html += `<div>🌾 Food: <b>${food}</b></div>`;
        html += `<div style="color:#888;">${VEG_NAMES[sector.vegetation] ?? '?'}</div>`;
        break;
      }

      case DisplayMode.Nation:
        if (sector.owner !== 0) {
          html += `<div>🏳️ <b>${ownerName}</b></div>`;
          const ntn = state.publicNations.find(n => n.nation_id === sector.owner);
          if (ntn) html += `<div style="color:#888;">${ntn.race ? RACE_NAMES[ntn.race] ?? ntn.race : ''}</div>`;
        } else {
          html += `<div style="color:#888;">${ELEV_NAMES[sector.altitude] ?? '?'}</div>`;
        }
        break;

      case DisplayMode.Race:
        if (sector.owner !== 0) {
          const ntn = state.publicNations.find(n => n.nation_id === sector.owner);
          const race = ntn?.race ?? '?';
          html += `<div>🧬 <b>${RACE_NAMES[race] ?? race}</b></div>`;
          html += `<div style="color:#888;">${ownerName}</div>`;
        } else {
          html += `<div style="color:#888;">${ELEV_NAMES[sector.altitude] ?? '?'}</div>`;
        }
        break;

      case DisplayMode.Move: {
        if (sector.altitude === 0) { html += `<div>🌊 Water — impassable</div>`; break; }
        if (sector.altitude === 1) { html += `<div>🗻 Peak — impassable</div>`; break; }
        const costs = [0, 0, 4, 3, 2];
        html += `<div>👢 Move cost: <b>${costs[sector.altitude] ?? '?'}</b></div>`;
        html += `<div style="color:#888;">${ELEV_NAMES[sector.altitude]} — ${VEG_NAMES[sector.vegetation]}</div>`;
        break;
      }

      case DisplayMode.Defense: {
        if (sector.altitude === 0) { html += `<div>🌊 Water</div>`; break; }
        let bonus = 0;
        if (sector.altitude === 2) bonus += 40;
        else if (sector.altitude === 3) bonus += 20;
        if (sector.vegetation === 8) bonus += 30;
        else if (sector.vegetation === 7) bonus += 20;
        else if (sector.vegetation === 6) bonus += 10;
        bonus += sector.fortress * 5;
        html += `<div>🛡️ Defense: <b>${bonus}%</b></div>`;
        html += `<div style="color:#888;">Terrain: ${ELEV_NAMES[sector.altitude]}, Veg: ${VEG_NAMES[sector.vegetation]}, Fort: ${sector.fortress}</div>`;
        break;
      }

      case DisplayMode.People:
        if (sector.altitude === 0) { html += `<div>🌊 Water</div>`; break; }
        html += `<div>👥 Population: <b>${sector.people.toLocaleString()}</b></div>`;
        break;

      case DisplayMode.Gold:
        if (sector.altitude === 0) { html += `<div>🌊 Water</div>`; break; }
        html += `<div>💎 Jewels: <b>${sector.jewels}</b></div>`;
        break;

      case DisplayMode.Metal:
        if (sector.altitude === 0) { html += `<div>🌊 Water</div>`; break; }
        html += `<div>⛏️ Metal: <b>${sector.metal}</b></div>`;
        break;

      case DisplayMode.Items:
        if (sector.altitude === 0) { html += `<div>🌊 Water</div>`; break; }
        html += `<div>📦 Trade good: <b>${sector.trade_good < 61 ? 'Yes' : 'None'}</b></div>`;
        if (sector.trade_good < 61) html += `<div style="color:#888;">Type: ${DES_NAMES[sector.designation] ?? '?'}</div>`;
        break;
    }

    // Always show army info if present
    const armiesHere = state.armies.filter(a => a.soldiers > 0 && a.x === absX && a.y === absY);
    if (armiesHere.length > 0) {
      html += `<div style="border-top:1px solid #333;margin-top:3px;padding-top:3px;">`;
      for (const a of armiesHere) {
        html += `<div>⚔️ Army ${a.index}: ${a.soldiers.toLocaleString()} soldiers</div>`;
      }
      html += `</div>`;
    }

    return html;
  }

  destroy(): void {
    this.el.remove();
  }
}
