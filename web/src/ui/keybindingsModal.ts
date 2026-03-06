// keybindingsModal.ts — Keybinding settings with per-user customization

export interface KeyBinding {
  action: string;
  label: string;
  category: string;
  defaultKey: string;
  key: string; // current binding (user-customized or default)
}

// Default keybindings (matching original C game where possible)
const DEFAULT_BINDINGS: Omit<KeyBinding, 'key'>[] = [
  // Cursor Movement
  { action: 'cursor_up',    label: 'Cursor Up',    category: 'Movement', defaultKey: 'ArrowUp' },
  { action: 'cursor_down',  label: 'Cursor Down',  category: 'Movement', defaultKey: 'ArrowDown' },
  { action: 'cursor_left',  label: 'Cursor Left',  category: 'Movement', defaultKey: 'ArrowLeft' },
  { action: 'cursor_right', label: 'Cursor Right',  category: 'Movement', defaultKey: 'ArrowRight' },
  { action: 'cursor_up_alt',    label: 'Cursor Up (vi)',    category: 'Movement', defaultKey: 'k' },
  { action: 'cursor_down_alt',  label: 'Cursor Down (vi)',  category: 'Movement', defaultKey: 'j' },
  { action: 'cursor_left_alt',  label: 'Cursor Left (vi)',  category: 'Movement', defaultKey: 'h' },
  { action: 'cursor_right_alt', label: 'Cursor Right (vi)', category: 'Movement', defaultKey: 'l' },

  // Army/Unit Commands
  { action: 'move_mode',    label: 'Move Unit (enter movement mode)', category: 'Units', defaultKey: 'm' },
  { action: 'move_done',    label: 'Done Moving (exit movement)',     category: 'Units', defaultKey: ' ' },
  { action: 'next_army',    label: 'Next Army (go to & select)',      category: 'Units', defaultKey: 'G' },
  { action: 'prev_army',    label: 'Previous Army',                   category: 'Units', defaultKey: 'Shift+G' },
  { action: 'toggle_navy',  label: 'Toggle Army/Navy',                category: 'Units', defaultKey: '`' },

  // Display Modes (original used 'd' to cycle, we map direct keys)
  { action: 'disp_veg',     label: 'Vegetation Display',  category: 'Display', defaultKey: 'v' },
  { action: 'disp_des',     label: 'Designation Display',  category: 'Display', defaultKey: 'd' },
  { action: 'disp_cnt',     label: 'Contour Display',      category: 'Display', defaultKey: 'c' },
  { action: 'disp_food',    label: 'Food Display',          category: 'Display', defaultKey: 'f' },
  { action: 'disp_race',    label: 'Race Display',          category: 'Display', defaultKey: 'r' },
  { action: 'disp_ntn',     label: 'Nation Display',        category: 'Display', defaultKey: 'n' },

  // Highlight Modes
  { action: 'hl_own',       label: 'Highlight Own Sectors',  category: 'Highlight', defaultKey: 'o' },
  { action: 'hl_army',      label: 'Highlight All Armies',    category: 'Highlight', defaultKey: 'a' },
  { action: 'hl_yours',     label: 'Highlight Your Armies',    category: 'Highlight', defaultKey: 'y' },
  { action: 'hl_none',      label: 'No Highlight',              category: 'Highlight', defaultKey: 'x' },

  // Game Commands
  { action: 'end_turn',     label: 'End Turn',          category: 'Game',    defaultKey: 'E' },
  { action: 'jump_capitol', label: 'Jump to Capitol',   category: 'Game',    defaultKey: 'g' },
  { action: 'show_scores',  label: 'Show Scores',       category: 'Game',    defaultKey: 'S' },
  { action: 'show_news',    label: 'Show News',         category: 'Game',    defaultKey: 'N' },
  { action: 'show_budget',  label: 'Budget',            category: 'Game',    defaultKey: 'B' },
  { action: 'toggle_chat',  label: 'Toggle Chat',       category: 'Game',    defaultKey: 't' },
  { action: 'show_help',    label: 'Help',               category: 'Game',    defaultKey: '?' },

  // Zoom
  { action: 'font_up',      label: 'Zoom In',          category: 'View',    defaultKey: '+' },
  { action: 'font_down',    label: 'Zoom Out',         category: 'View',    defaultKey: '-' },
];

const STORAGE_KEY = 'conquer_keybindings';

export class KeybindingsManager {
  private bindings: KeyBinding[];

  constructor() {
    this.bindings = this.load();
  }

  private load(): KeyBinding[] {
    const saved = localStorage.getItem(STORAGE_KEY);
    const custom: Record<string, string> = saved ? JSON.parse(saved) : {};

    return DEFAULT_BINDINGS.map(b => ({
      ...b,
      key: custom[b.action] ?? b.defaultKey,
    }));
  }

  save(): void {
    const custom: Record<string, string> = {};
    for (const b of this.bindings) {
      if (b.key !== b.defaultKey) {
        custom[b.action] = b.key;
      }
    }
    localStorage.setItem(STORAGE_KEY, JSON.stringify(custom));
  }

  getBindings(): KeyBinding[] {
    return this.bindings;
  }

  setKey(action: string, key: string): void {
    const binding = this.bindings.find(b => b.action === action);
    if (binding) {
      binding.key = key;
      this.save();
    }
  }

  resetAll(): void {
    for (const b of this.bindings) {
      b.key = b.defaultKey;
    }
    localStorage.removeItem(STORAGE_KEY);
  }

  resetOne(action: string): void {
    const binding = this.bindings.find(b => b.action === action);
    if (binding) {
      binding.key = binding.defaultKey;
      this.save();
    }
  }

  /** Look up action for a key event */
  getAction(key: string, shift: boolean): string | null {
    const lookup = shift && key.length === 1 ? `Shift+${key}` : key;
    const binding = this.bindings.find(b => b.key === lookup || b.key === key);
    return binding?.action ?? null;
  }
}

/** Format a key for display */
function displayKey(key: string): string {
  if (key === ' ') return 'Space';
  if (key === 'ArrowUp') return '↑';
  if (key === 'ArrowDown') return '↓';
  if (key === 'ArrowLeft') return '←';
  if (key === 'ArrowRight') return '→';
  if (key === '`') return '`';
  return key;
}

/** Format a key event for storage */
function eventToKey(e: KeyboardEvent): string {
  if (e.key === ' ') return ' ';
  if (e.key.startsWith('Arrow')) return e.key;
  if (e.shiftKey && e.key.length === 1) return `Shift+${e.key}`;
  return e.key;
}

export class KeybindingsModal {
  private overlay: HTMLDivElement;
  private manager: KeybindingsManager;
  private onClose: () => void;
  private rebindingAction: string | null = null;
  private rebindingEl: HTMLElement | null = null;

  constructor(parent: HTMLElement, manager: KeybindingsManager, onClose: () => void) {
    this.manager = manager;
    this.onClose = onClose;

    this.overlay = document.createElement('div');
    this.overlay.style.cssText = `
      position: fixed; inset: 0; z-index: 1000;
      background: rgba(0,0,0,0.85); display: flex;
      align-items: center; justify-content: center;
    `;

    const modal = document.createElement('div');
    modal.style.cssText = `
      background: #111; border: 2px solid #333; border-radius: 8px;
      padding: 24px; max-width: 600px; width: 90vw; max-height: 80vh;
      overflow-y: auto; font-family: "Courier New", monospace;
      color: #ccc;
    `;

    // Header
    const header = document.createElement('div');
    header.style.cssText = 'display:flex;justify-content:space-between;align-items:center;margin-bottom:16px;';
    header.innerHTML = `<h2 style="margin:0;color:#55ff55;font-size:20px;">⌨ Key Bindings</h2>`;
    const closeBtn = document.createElement('button');
    closeBtn.textContent = '✕';
    closeBtn.style.cssText = 'background:none;border:1px solid #555;color:#ff5555;font-size:18px;padding:4px 10px;cursor:pointer;border-radius:4px;';
    closeBtn.onclick = () => this.close();
    header.appendChild(closeBtn);
    modal.appendChild(header);

    // Instructions
    const info = document.createElement('p');
    info.style.cssText = 'color:#888;font-size:13px;margin:0 0 16px;';
    info.textContent = 'Click a key to rebind it. Press the new key, or Escape to cancel.';
    modal.appendChild(info);

    // Group by category
    const categories = new Map<string, KeyBinding[]>();
    for (const b of manager.getBindings()) {
      if (!categories.has(b.category)) categories.set(b.category, []);
      categories.get(b.category)!.push(b);
    }

    for (const [cat, bindings] of categories) {
      const section = document.createElement('div');
      section.style.cssText = 'margin-bottom:16px;';

      const catHeader = document.createElement('div');
      catHeader.style.cssText = 'color:#55ff55;font-size:14px;text-transform:uppercase;letter-spacing:1px;padding:4px 0;border-bottom:1px solid #333;margin-bottom:8px;';
      catHeader.textContent = cat;
      section.appendChild(catHeader);

      for (const b of bindings) {
        const row = document.createElement('div');
        row.style.cssText = 'display:flex;justify-content:space-between;align-items:center;padding:6px 8px;border-radius:3px;';
        row.onmouseenter = () => row.style.background = '#1a1a1a';
        row.onmouseleave = () => row.style.background = 'transparent';

        const label = document.createElement('span');
        label.style.cssText = 'font-size:14px;';
        label.textContent = b.label;

        const keyArea = document.createElement('div');
        keyArea.style.cssText = 'display:flex;gap:8px;align-items:center;';

        const keyBtn = document.createElement('button');
        keyBtn.style.cssText = `
          background: #222; border: 1px solid #444; color: #ffcc00;
          font-family: inherit; font-size: 14px; padding: 4px 12px;
          cursor: pointer; border-radius: 3px; min-width: 60px;
          text-align: center;
        `;
        keyBtn.textContent = displayKey(b.key);
        keyBtn.dataset.action = b.action;
        keyBtn.onclick = () => this.startRebind(b.action, keyBtn);

        const resetBtn = document.createElement('button');
        resetBtn.style.cssText = 'background:none;border:none;color:#666;font-size:11px;cursor:pointer;padding:2px 4px;';
        resetBtn.textContent = b.key !== b.defaultKey ? `(${displayKey(b.defaultKey)})` : '';
        resetBtn.title = 'Reset to default';
        if (b.key !== b.defaultKey) {
          resetBtn.style.color = '#ff8800';
          resetBtn.onclick = () => {
            manager.resetOne(b.action);
            keyBtn.textContent = displayKey(b.defaultKey);
            resetBtn.textContent = '';
            resetBtn.style.color = '#666';
          };
        }

        keyArea.appendChild(keyBtn);
        keyArea.appendChild(resetBtn);
        row.appendChild(label);
        row.appendChild(keyArea);
        section.appendChild(row);
      }

      modal.appendChild(section);
    }

    // Reset all button
    const footer = document.createElement('div');
    footer.style.cssText = 'text-align:right;padding-top:12px;border-top:1px solid #333;';
    const resetAllBtn = document.createElement('button');
    resetAllBtn.textContent = 'Reset All to Defaults';
    resetAllBtn.style.cssText = 'background:none;border:1px solid #555;color:#ff5555;font-family:inherit;font-size:13px;padding:8px 16px;cursor:pointer;border-radius:4px;';
    resetAllBtn.onclick = () => {
      manager.resetAll();
      this.close();
      // Reopen to refresh display
      new KeybindingsModal(parent, manager, onClose);
    };
    footer.appendChild(resetAllBtn);
    modal.appendChild(footer);

    this.overlay.appendChild(modal);
    parent.appendChild(this.overlay);

    // Listen for rebind keys
    this._onKeyDown = this._onKeyDown.bind(this);
    document.addEventListener('keydown', this._onKeyDown, true);

    // Close on overlay click
    this.overlay.addEventListener('click', (e) => {
      if (e.target === this.overlay) this.close();
    });
  }

  private startRebind(action: string, el: HTMLElement): void {
    // Cancel previous rebind
    if (this.rebindingEl) {
      this.rebindingEl.style.borderColor = '#444';
      this.rebindingEl.textContent = displayKey(
        this.manager.getBindings().find(b => b.action === this.rebindingAction)?.key ?? ''
      );
    }

    this.rebindingAction = action;
    this.rebindingEl = el;
    el.textContent = '...press key...';
    el.style.borderColor = '#55ff55';
  }

  private _onKeyDown(e: KeyboardEvent): void {
    if (!this.rebindingAction) {
      if (e.key === 'Escape') this.close();
      return;
    }

    e.preventDefault();
    e.stopPropagation();

    if (e.key === 'Escape') {
      // Cancel rebind
      if (this.rebindingEl) {
        this.rebindingEl.style.borderColor = '#444';
        this.rebindingEl.textContent = displayKey(
          this.manager.getBindings().find(b => b.action === this.rebindingAction)?.key ?? ''
        );
      }
      this.rebindingAction = null;
      this.rebindingEl = null;
      return;
    }

    const newKey = eventToKey(e);
    this.manager.setKey(this.rebindingAction, newKey);

    if (this.rebindingEl) {
      this.rebindingEl.textContent = displayKey(newKey);
      this.rebindingEl.style.borderColor = '#444';
    }

    this.rebindingAction = null;
    this.rebindingEl = null;
  }

  private close(): void {
    document.removeEventListener('keydown', this._onKeyDown, true);
    this.overlay.remove();
    this.onClose();
  }
}
