// commandSidebar.ts — Left sidebar with command buttons that highlight on keypress

import { getUiTheme } from './uiThemes';
import { DisplayMode, HighlightMode } from '../types';
import { ALL_TILESETS } from '../renderer/tilesets';

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
    name: 'Layers',
    collapsed: true,
    cmds: [
      { label: '🗺 Terrain', cmd: 'layer_terrain' },
      { label: '🌿 Vegetation', cmd: 'layer_vegetation' },
      { label: '🏘 Designations', cmd: 'layer_designation' },
      { label: '💎 Resources', cmd: 'layer_resources' },
      { label: '🏳 Ownership', cmd: 'layer_ownership' },
      { label: '⚔ Units', cmd: 'layer_units' },
      { label: '📊 All Layers', cmd: 'layer_all' },
      { label: '🗺 Mode Default', cmd: 'layer_mode_default' },
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
      { label: 'Game Font +', cmd: 'font_up', key: '+' },
      { label: 'Game Font -', cmd: 'font_down', key: '-' },
      { label: 'Menu Font +', cmd: 'sidebar_font_up' },
      { label: 'Menu Font -', cmd: 'sidebar_font_down' },
      { label: 'Center Map', cmd: 'center_map' },
      { label: 'Focus Mode', cmd: 'toggle_sidebars' },
      { label: '── UI Theme ──', cmd: '_sep' },
      { label: '💻 Terminal', cmd: 'uitheme_terminal' },
      { label: '🪨 Slate', cmd: 'uitheme_slate' },
      { label: '🟠 Amber CRT', cmd: 'uitheme_amber' },
      { label: '🎖 Military', cmd: 'uitheme_military' },
      { label: '── Map Theme ──', cmd: '_sep' },
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
    name: 'Tileset',
    collapsed: true,
    cmds: [
      // Dynamically built from ALL_TILESETS registry
      ...ALL_TILESETS.map(ts => ({
        label: ts.tileType === 'image' ? `🎨 ${ts.name}` : ts.tileType === 'emoji' ? `😀 ${ts.name}` : ts.name,
        cmd: `tileset_${ts.id}`,
      })),
      { label: '✏️ Edit Tileset...', cmd: 'tileset_editor' },
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
  private _fontSize: number = parseInt(localStorage.getItem('conquer_sidebar_font') ?? '12');

  constructor(container: HTMLElement, callback: CmdCallback) {
    this.container = container;
    this.callback = callback;
    this.render();

    // Listen for keyboard to flash buttons
    window.addEventListener('keydown', (e) => this.onKey(e));
  }

  set themeId(id: string) { this._themeId = id; this.render(); }
  get fontSize(): number { return this._fontSize; }

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
    this.container.style.fontSize = `${this._fontSize}px`;

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
            ? `<span style="opacity:0.5;margin-right:4px;">(${cmd.key})</span>${cmd.label}`
            : cmd.label;
          btn.addEventListener('click', () => {
            if (cmd.cmd === 'sidebar_font_up') {
              this._fontSize = Math.min(20, this._fontSize + 1);
              localStorage.setItem('conquer_sidebar_font', String(this._fontSize));
              this.render();
              this.callback('_sidebar_font_changed');
            } else if (cmd.cmd === 'sidebar_font_down') {
              this._fontSize = Math.max(9, this._fontSize - 1);
              localStorage.setItem('conquer_sidebar_font', String(this._fontSize));
              this.render();
              this.callback('_sidebar_font_changed');
            } else if (cmd.cmd !== '_sep') {
              this.callback(cmd.cmd);
              this.flash(cmd.cmd);
            }
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
