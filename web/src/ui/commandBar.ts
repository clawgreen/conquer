// commandBar.ts — Bottom command button bar for touch/click interaction

import { ALL_THEMES } from '../renderer/themes';

export type CommandCallback = (cmd: string) => void;

interface CmdButton {
  label: string;
  cmd: string;
  title: string;
  color?: string;
}

const COMMAND_GROUPS: { name: string; buttons: CmdButton[] }[] = [
  {
    name: 'Move',
    buttons: [
      { label: '⬆', cmd: 'move_up', title: 'Move Up' },
      { label: '⬇', cmd: 'move_down', title: 'Move Down' },
      { label: '⬅', cmd: 'move_left', title: 'Move Left' },
      { label: '➡', cmd: 'move_right', title: 'Move Right' },
    ],
  },
  {
    name: 'Army',
    buttons: [
      { label: '⇥ Army', cmd: 'next_army', title: 'Next Army (Tab)' },
      { label: '⇤ Prev', cmd: 'prev_army', title: 'Previous Army' },
      { label: '⚔ Move', cmd: 'army_move', title: 'Move Army (arrows after)', color: '#ff5555' },
      { label: '🚢 Navy', cmd: 'toggle_navy', title: 'Toggle Army/Navy view' },
    ],
  },
  {
    name: 'Display',
    buttons: [
      { label: 'Veg', cmd: 'disp_veg', title: 'Vegetation (v)' },
      { label: 'Des', cmd: 'disp_des', title: 'Designation (d)' },
      { label: 'Cnt', cmd: 'disp_cnt', title: 'Contour (c)' },
      { label: 'Food', cmd: 'disp_food', title: 'Food (f)' },
      { label: 'Race', cmd: 'disp_race', title: 'Race (r)' },
      { label: 'Ntn', cmd: 'disp_ntn', title: 'Nation (n)' },
      { label: 'Move', cmd: 'disp_move', title: 'Movement (m)' },
      { label: 'Def', cmd: 'disp_def', title: 'Defense (M)' },
      { label: 'Pop', cmd: 'disp_pop', title: 'People (D)' },
      { label: 'Gold', cmd: 'disp_gold', title: 'Gold (p)' },
      { label: 'Metal', cmd: 'disp_mtl', title: 'Metal (J)' },
      { label: 'Items', cmd: 'disp_itm', title: 'Items (i)' },
    ],
  },
  {
    name: 'Highlight',
    buttons: [
      { label: '👑 Own', cmd: 'hl_own', title: 'Own sectors (o)' },
      { label: '⚔ Army', cmd: 'hl_army', title: 'All armies (a)' },
      { label: '🎯 Yours', cmd: 'hl_yours', title: 'Your armies (y)' },
      { label: '🏃 Move', cmd: 'hl_move', title: 'Movement range (L)' },
      { label: '🤝 Trade', cmd: 'hl_trade', title: 'Trading partners (s)' },
      { label: '✕ None', cmd: 'hl_none', title: 'No highlight (x)' },
    ],
  },
  {
    name: 'Actions',
    buttons: [
      { label: '🏠 Capitol', cmd: 'jump_capitol', title: 'Jump to Capitol' },
      { label: '📊 Scores', cmd: 'show_scores', title: 'Show Scores' },
      { label: '📰 News', cmd: 'show_news', title: 'Show News' },
      { label: '💬 Chat', cmd: 'toggle_chat', title: 'Toggle Chat (T)' },
      { label: '🔄 Refresh', cmd: 'refresh', title: 'Refresh game data' },
      { label: '✅ End Turn', cmd: 'end_turn', title: 'End Turn (E)', color: '#55ff55' },
      { label: '🚪 Lobby', cmd: 'back_to_lobby', title: 'Back to game lobby' },
    ],
  },
  {
    name: 'View',
    buttons: [
      // Theme buttons are dynamically generated from ALL_THEMES
      ...ALL_THEMES.map(t => ({
        label: `${t.icon} ${t.name}`,
        cmd: `theme_${t.id}`,
        title: t.description,
      })),
      { label: 'A+', cmd: 'font_up', title: 'Increase font size (+)' },
      { label: 'A-', cmd: 'font_down', title: 'Decrease font size (-)' },
      { label: '🔲 Center', cmd: 'center_map', title: 'Center map on cursor' },
    ],
  },
];

export class CommandBar {
  private container: HTMLElement;
  private callback: CommandCallback;
  private activeGroup: number = 0;

  constructor(parent: HTMLElement, callback: CommandCallback) {
    this.callback = callback;

    this.container = document.createElement('div');
    this.container.id = 'command-bar';
    this.container.style.cssText = `
      position: fixed; bottom: 0; left: 0; right: 0;
      background: rgba(0,17,0,0.95); border-top: 1px solid #338833;
      z-index: 100; font-family: "Courier New", monospace;
      padding: env(safe-area-inset-bottom, 0) 0 0 0;
      touch-action: manipulation;
    `;
    parent.appendChild(this.container);
    this.render();
  }

  private render(): void {
    const tabsHtml = COMMAND_GROUPS.map((g, i) => {
      const active = i === this.activeGroup;
      return `<button class="cmd-tab${active ? ' active' : ''}" data-tab="${i}"
        style="flex:1;padding:6px 2px;font-size:11px;font-family:inherit;cursor:pointer;border:none;
        background:${active ? '#003300' : '#001100'};color:${active ? '#55ff55' : '#338833'};
        border-bottom:${active ? '2px solid #55ff55' : '2px solid transparent'};">${g.name}</button>`;
    }).join('');

    const group = COMMAND_GROUPS[this.activeGroup];
    const btnsHtml = group.buttons.map(b => {
      const bg = b.color ? b.color.replace('#', 'rgba(') + ',0.15)' : 'rgba(0,51,0,0.8)';
      const border = b.color ?? '#338833';
      const fg = b.color ?? '#55ff55';
      return `<button class="cmd-btn" data-cmd="${b.cmd}" title="${b.title}"
        style="padding:8px 4px;font-size:12px;font-family:inherit;cursor:pointer;
        background:${bg};color:${fg};border:1px solid ${border};border-radius:4px;
        min-width:44px;min-height:44px;touch-action:manipulation;">${b.label}</button>`;
    }).join('');

    this.container.innerHTML = `
      <div style="display:flex;border-bottom:1px solid #113311;">${tabsHtml}</div>
      <div style="display:flex;flex-wrap:wrap;gap:4px;padding:6px;justify-content:center;">${btnsHtml}</div>
    `;

    // Tab click handlers
    this.container.querySelectorAll('.cmd-tab').forEach(el => {
      el.addEventListener('click', (e) => {
        this.activeGroup = parseInt((e.currentTarget as HTMLElement).dataset.tab!);
        this.render();
      });
    });

    // Button click handlers
    this.container.querySelectorAll('.cmd-btn').forEach(el => {
      el.addEventListener('click', (e) => {
        e.preventDefault();
        e.stopPropagation();
        const cmd = (e.currentTarget as HTMLElement).dataset.cmd!;
        this.callback(cmd);
      });
    });
  }

  getHeight(): number {
    return this.container.offsetHeight;
  }

  destroy(): void {
    this.container.remove();
  }
}
