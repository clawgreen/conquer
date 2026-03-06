// mobileToolbar.ts — Mobile command toolbar for touch input
// Renders a fixed bar at the bottom of the screen with game action buttons
// All buttons dispatch GameAction through the same system as keyboard

import { GameAction } from '../game/inputHandler';
import { DisplayMode, HighlightMode } from '../types';

const DISPLAY_MODES: { mode: DisplayMode; label: string; key: string }[] = [
  { mode: DisplayMode.Vegetation, label: 'Vegetation', key: 'v' },
  { mode: DisplayMode.Designation, label: 'Designation', key: 'd' },
  { mode: DisplayMode.Contour, label: 'Contour', key: 'c' },
  { mode: DisplayMode.Food, label: 'Food', key: 'f' },
  { mode: DisplayMode.Nation, label: 'Nation', key: 'n' },
  { mode: DisplayMode.Race, label: 'Race', key: 'r' },
  { mode: DisplayMode.Move, label: 'Move Cost', key: 'M' },
  { mode: DisplayMode.Defense, label: 'Defense', key: 'D' },
  { mode: DisplayMode.People, label: 'People', key: 'p' },
  { mode: DisplayMode.Gold, label: 'Gold', key: 'J' },
  { mode: DisplayMode.Metal, label: 'Metal', key: 'm' },
  { mode: DisplayMode.Items, label: 'Items', key: 'i' },
];

const HIGHLIGHT_MODES: { mode: HighlightMode; label: string; key: string }[] = [
  { mode: HighlightMode.None, label: 'None', key: 'x' },
  { mode: HighlightMode.Own, label: 'Own', key: 'o' },
  { mode: HighlightMode.Army, label: 'All Army', key: 'a' },
  { mode: HighlightMode.YourArmy, label: 'Your Army', key: 'y' },
  { mode: HighlightMode.Move, label: 'Can Move', key: 'L' },
  { mode: HighlightMode.Good, label: 'Trade Good', key: 's' },
];

export class MobileToolbar {
  private container: HTMLDivElement;
  private onAction: (action: GameAction) => void;
  private infoExpanded = false;
  private currentArmyDir: { dx: number; dy: number } = { dx: 0, dy: -1 }; // default: up

  constructor(parent: HTMLElement, onAction: (action: GameAction) => void) {
    this.onAction = onAction;

    this.container = document.createElement('div');
    this.container.id = 'mobile-toolbar';
    this.container.className = 'mobile-toolbar';

    this.buildToolbar();
    parent.appendChild(this.container);
  }

  private buildToolbar(): void {
    this.container.innerHTML = '';

    // Row 1: D-pad + Army selection
    const row1 = this.createRow('toolbar-row toolbar-row-nav');

    // D-pad
    const dpad = this.el('div', 'dpad');

    //       ↖ ↑ ↗
    //       ← · →
    //       ↙ ↓ ↘
    const dirs: { label: string; dx: number; dy: number; cls: string }[] = [
      { label: '↖', dx: -1, dy: -1, cls: 'dpad-nw' },
      { label: '↑', dx: 0, dy: -1, cls: 'dpad-n' },
      { label: '↗', dx: 1, dy: -1, cls: 'dpad-ne' },
      { label: '←', dx: -1, dy: 0, cls: 'dpad-w' },
      { label: '·', dx: 0, dy: 0, cls: 'dpad-c' },
      { label: '→', dx: 1, dy: 0, cls: 'dpad-e' },
      { label: '↙', dx: -1, dy: 1, cls: 'dpad-sw' },
      { label: '↓', dx: 0, dy: 1, cls: 'dpad-s' },
      { label: '↘', dx: 1, dy: 1, cls: 'dpad-se' },
    ];

    for (const d of dirs) {
      const btn = this.btn(d.label, `dpad-btn ${d.cls}`, () => {
        if (d.dx === 0 && d.dy === 0) {
          // Center: center_map
          this.onAction({ type: 'center_map' });
        } else {
          this.currentArmyDir = { dx: d.dx, dy: d.dy };
          this.onAction({ type: 'move_cursor', dx: d.dx, dy: d.dy });
        }
      });
      dpad.appendChild(btn);
    }
    row1.appendChild(dpad);

    // Army controls
    const armyCtrl = this.el('div', 'army-controls');
    armyCtrl.appendChild(this.btn('◀', 'tb-btn', () => this.onAction({ type: 'select_prev_army' })));
    armyCtrl.appendChild(this.btn('▶', 'tb-btn', () => this.onAction({ type: 'select_next_army' })));
    armyCtrl.appendChild(this.btn('⚔', 'tb-btn tb-btn-attack', () => {
      this.onAction({ type: 'move_army', dx: this.currentArmyDir.dx, dy: this.currentArmyDir.dy });
    }));
    armyCtrl.appendChild(this.btn('A/N', 'tb-btn tb-btn-sm', () => this.onAction({ type: 'toggle_army_navy' })));
    row1.appendChild(armyCtrl);

    this.container.appendChild(row1);

    // Row 2: Action buttons
    const row2 = this.createRow('toolbar-row toolbar-row-actions');
    row2.appendChild(this.btn('🏗 Redes', 'tb-btn', () => this.onAction({ type: 'redesignate' })));
    row2.appendChild(this.btn('👥 Draft', 'tb-btn', () => this.onAction({ type: 'draft' })));
    row2.appendChild(this.btn('🤝 Diplo', 'tb-btn', () => this.onAction({ type: 'diplomacy' })));
    row2.appendChild(this.btn('✨ Magic', 'tb-btn', () => this.onAction({ type: 'magic' })));
    row2.appendChild(this.btn('⏩ End', 'tb-btn tb-btn-end', () => this.onAction({ type: 'end_turn' })));
    row2.appendChild(this.btn('···', 'tb-btn tb-btn-more', () => this.toggleInfo()));
    this.container.appendChild(row2);

    // Row 3: Info buttons (collapsible)
    const row3 = this.createRow('toolbar-row toolbar-row-info');
    row3.id = 'toolbar-info-row';
    row3.style.display = 'none';

    row3.appendChild(this.btn('📊 Score', 'tb-btn', () => this.onAction({ type: 'show_scores' })));
    row3.appendChild(this.btn('📰 News', 'tb-btn', () => this.onAction({ type: 'show_news' })));
    row3.appendChild(this.btn('💰 Budget', 'tb-btn', () => this.onAction({ type: 'show_budget' })));
    row3.appendChild(this.btn('❓ Help', 'tb-btn', () => this.onAction({ type: 'show_help' })));
    row3.appendChild(this.btn('💬 Chat', 'tb-btn', () => this.onAction({ type: 'toggle_chat' })));

    // Display mode dropdown
    const dispSelect = this.createSelect('🗺 Display', DISPLAY_MODES, (item) => {
      this.onAction({ type: 'set_display', mode: item.mode });
    });
    row3.appendChild(dispSelect);

    // Highlight mode dropdown
    const hlSelect = this.createSelect('🔦 Highlight', HIGHLIGHT_MODES, (item) => {
      this.onAction({ type: 'set_highlight', mode: item.mode });
    });
    row3.appendChild(hlSelect);

    // Zoom controls
    row3.appendChild(this.btn('🔍+', 'tb-btn', () => this.onAction({ type: 'font_increase' })));
    row3.appendChild(this.btn('🔍-', 'tb-btn', () => this.onAction({ type: 'font_decrease' })));

    this.container.appendChild(row3);
  }

  private toggleInfo(): void {
    this.infoExpanded = !this.infoExpanded;
    const row = document.getElementById('toolbar-info-row');
    if (row) {
      row.style.display = this.infoExpanded ? 'flex' : 'none';
    }
    // Fire resize event so gameScreen can recalculate canvas size
    window.dispatchEvent(new Event('resize'));
  }

  private createRow(className: string): HTMLDivElement {
    const row = document.createElement('div');
    row.className = className;
    return row;
  }

  private el(tag: string, className: string): HTMLDivElement {
    const el = document.createElement(tag) as HTMLDivElement;
    el.className = className;
    return el;
  }

  private btn(label: string, className: string, onClick: () => void): HTMLButtonElement {
    const btn = document.createElement('button');
    btn.className = className;
    btn.textContent = label;
    btn.addEventListener('touchstart', (e) => {
      e.preventDefault();
      e.stopPropagation();
      onClick();
    }, { passive: false });
    btn.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
      onClick();
    });
    return btn;
  }

  private createSelect<T extends { label: string }>(
    placeholder: string,
    items: T[],
    onSelect: (item: T) => void,
  ): HTMLDivElement {
    const wrapper = document.createElement('div');
    wrapper.className = 'tb-select-wrap';

    const sel = document.createElement('select');
    sel.className = 'tb-select';

    const defaultOpt = document.createElement('option');
    defaultOpt.value = '';
    defaultOpt.textContent = placeholder;
    defaultOpt.disabled = true;
    defaultOpt.selected = true;
    sel.appendChild(defaultOpt);

    for (let i = 0; i < items.length; i++) {
      const opt = document.createElement('option');
      opt.value = String(i);
      opt.textContent = items[i].label;
      sel.appendChild(opt);
    }

    sel.addEventListener('change', () => {
      const idx = parseInt(sel.value);
      if (!isNaN(idx) && items[idx]) {
        onSelect(items[idx]);
      }
      // Reset to placeholder
      sel.selectedIndex = 0;
    });

    wrapper.appendChild(sel);
    return wrapper;
  }

  /** Get the current height of the toolbar for canvas sizing */
  getHeight(): number {
    return this.container.offsetHeight || 0;
  }

  show(): void {
    this.container.style.display = '';
  }

  hide(): void {
    this.container.style.display = 'none';
  }

  destroy(): void {
    this.container.remove();
  }
}
