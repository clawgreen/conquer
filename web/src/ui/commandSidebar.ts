// commandSidebar.ts — Left sidebar with command buttons that highlight on keypress

import { getUiTheme } from './uiThemes';
import { DisplayMode, HighlightMode } from '../types';

export type CmdCallback = (cmd: string) => void;

interface CmdDef {
  label: string;
  cmd: string;
  key?: string;   // keyboard shortcut display
}

interface CmdGroup {
  name: string;
  collapsed?: boolean;
  cmds: CmdDef[];
}

const CMD_GROUPS: CmdGroup[] = [
  {
    name: 'Display',
    cmds: [
      { label: 'Vegetation', cmd: 'disp_veg', key: 'v' },
      { label: 'Designation', cmd: 'disp_des', key: 'd' },
      { label: 'Contour', cmd: 'disp_cnt', key: 'c' },
      { label: 'Food', cmd: 'disp_food', key: 'f' },
      { label: 'Race', cmd: 'disp_race', key: 'r' },
      { label: 'Nation', cmd: 'disp_ntn', key: 'n' },
      { label: 'Movement', cmd: 'disp_move', key: 'm' },
      { label: 'Defense', cmd: 'disp_def', key: 'M' },
      { label: 'People', cmd: 'disp_pop', key: 'D' },
      { label: 'Gold', cmd: 'disp_gold', key: 'p' },
      { label: 'Metal', cmd: 'disp_mtl', key: 'J' },
      { label: 'Items', cmd: 'disp_itm', key: 'i' },
    ],
  },
  {
    name: 'Highlight',
    cmds: [
      { label: 'Own Sectors', cmd: 'hl_own', key: 'o' },
      { label: 'All Armies', cmd: 'hl_army', key: 'a' },
      { label: 'Your Armies', cmd: 'hl_yours', key: 'y' },
      { label: 'Move Range', cmd: 'hl_move', key: 'L' },
      { label: 'Trade', cmd: 'hl_trade', key: 's' },
      { label: 'None', cmd: 'hl_none', key: 'x' },
    ],
  },
  {
    name: 'Army',
    cmds: [
      { label: 'Next Army', cmd: 'next_army', key: 'Tab' },
      { label: 'Prev Army', cmd: 'prev_army' },
      { label: 'Move Army', cmd: 'army_move' },
      { label: 'Toggle Navy', cmd: 'toggle_navy' },
    ],
  },
  {
    name: 'Actions',
    cmds: [
      { label: '⚔ End Turn', cmd: 'end_turn', key: 'E' },
      { label: '🏠 Capitol', cmd: 'jump_capitol' },
      { label: '📊 Scores', cmd: 'show_scores' },
      { label: '📰 News', cmd: 'show_news' },
      { label: '💬 Chat', cmd: 'toggle_chat', key: 'T' },
      { label: '🔄 Refresh', cmd: 'refresh' },
    ],
  },
  {
    name: 'View',
    cmds: [
      { label: 'A+ Font', cmd: 'font_up', key: '+' },
      { label: 'A- Font', cmd: 'font_down', key: '-' },
      { label: 'Center Map', cmd: 'center_map' },
      { label: 'Focus Mode', cmd: 'toggle_sidebars' },
      { label: '── Themes ──', cmd: '_sep' },
      { label: '🖥 Classic Green', cmd: 'theme_classic-green' },
      { label: '🟠 Amber', cmd: 'theme_classic-amber' },
      { label: '⬜ White', cmd: 'theme_classic-white' },
      { label: '🎨 Enhanced', cmd: 'theme_enhanced' },
      { label: '🗺 Tactical', cmd: 'theme_tactical' },
      { label: '📜 Parchment', cmd: 'theme_parchment' },
      { label: '📐 Blueprint', cmd: 'theme_blueprint' },
      { label: '🔥 Heatmap', cmd: 'theme_heatmap' },
    ],
  },
  {
    name: 'System',
    cmds: [
      { label: '🚪 Back to Lobby', cmd: 'back_to_lobby' },
    ],
  },
];

// Map keyboard keys to command IDs for flash feedback
const KEY_TO_CMD: Record<string, string> = {};
for (const g of CMD_GROUPS) {
  for (const c of g.cmds) {
    if (c.key) KEY_TO_CMD[c.key] = c.cmd;
  }
}

export class CommandSidebar {
  private container: HTMLElement;
  private callback: CmdCallback;
  private btnElements: Map<string, HTMLElement> = new Map();
  private collapsedGroups: Set<string> = new Set();
  private _themeId = 'terminal';

  constructor(container: HTMLElement, callback: CmdCallback) {
    this.container = container;
    this.callback = callback;
    this.render();

    // Listen for keyboard to flash buttons
    window.addEventListener('keydown', (e) => this.onKey(e));
  }

  set themeId(id: string) { this._themeId = id; this.render(); }

  flash(cmd: string): void {
    const el = this.btnElements.get(cmd);
    if (!el) return;
    const t = getUiTheme(this._themeId);
    el.classList.add('flash');
    el.style.background = t.btnActiveBg;
    el.style.color = t.btnActiveText;
    el.style.borderLeftColor = t.sidebarAccent;
    setTimeout(() => {
      el.classList.remove('flash');
      el.style.background = t.btnBg;
      el.style.color = t.btnText;
      el.style.borderLeftColor = 'transparent';
    }, 200);
  }

  private onKey(e: KeyboardEvent): void {
    const cmd = KEY_TO_CMD[e.key];
    if (cmd) this.flash(cmd);
  }

  private render(): void {
    const t = getUiTheme(this._themeId);
    this.btnElements.clear();
    this.container.innerHTML = '';

    for (const group of CMD_GROUPS) {
      const div = document.createElement('div');
      div.className = 'cmd-group';

      const header = document.createElement('div');
      header.className = 'cmd-group-header';
      header.style.cssText = `background:${t.sidebarHeaderBg};color:${t.sidebarDim};`;
      const collapsed = this.collapsedGroups.has(group.name);
      header.textContent = `${collapsed ? '▸' : '▾'} ${group.name}`;
      header.addEventListener('click', () => {
        if (this.collapsedGroups.has(group.name)) {
          this.collapsedGroups.delete(group.name);
        } else {
          this.collapsedGroups.add(group.name);
        }
        this.render();
      });
      div.appendChild(header);

      if (!collapsed) {
        for (const cmd of group.cmds) {
          const btn = document.createElement('button');
          btn.className = 'cmd-btn';
          btn.style.cssText = `background:${t.btnBg};color:${t.btnText};border-left:2px solid transparent;`;
          btn.innerHTML = cmd.key
            ? `${cmd.label} <span style="float:right;opacity:0.4;font-size:10px;">${cmd.key}</span>`
            : cmd.label;
          btn.addEventListener('click', () => {
            this.callback(cmd.cmd);
            this.flash(cmd.cmd);
          });
          div.appendChild(btn);
          this.btnElements.set(cmd.cmd, btn);
        }
      }

      this.container.appendChild(div);
    }
  }

  destroy(): void {
    // Cleanup if needed
  }
}
