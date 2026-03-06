// inputHandler.ts — Keyboard input handling
// T354-T372: Map navigation, army commands, display/highlight toggles, command keys

import { GameState } from '../state/gameState';
import { DisplayMode, HighlightMode } from '../types';

export type GameAction =
  | { type: 'move_cursor'; dx: number; dy: number }
  | { type: 'center_map' }
  | { type: 'jump_capitol' }
  | { type: 'set_display'; mode: DisplayMode }
  | { type: 'set_highlight'; mode: HighlightMode }
  | { type: 'select_next_army' }
  | { type: 'select_prev_army' }
  | { type: 'toggle_army_navy' }
  | { type: 'move_army'; dx: number; dy: number }
  | { type: 'end_turn' }
  | { type: 'show_scores' }
  | { type: 'show_news' }
  | { type: 'show_budget' }
  | { type: 'show_help' }
  | { type: 'redesignate' }
  | { type: 'draft' }
  | { type: 'diplomacy' }
  | { type: 'magic' }
  | { type: 'font_increase' }
  | { type: 'font_decrease' }
  | { type: 'toggle_chat' }
  | { type: 'noop' };

export class InputHandler {
  private onAction: (action: GameAction) => void;
  private _enabled = true;

  constructor(onAction: (action: GameAction) => void) {
    this.onAction = onAction;
    this.handleKeyDown = this.handleKeyDown.bind(this);
    document.addEventListener('keydown', this.handleKeyDown);
  }

  set enabled(v: boolean) { this._enabled = v; }

  destroy(): void {
    document.removeEventListener('keydown', this.handleKeyDown);
  }

  private handleKeyDown(e: KeyboardEvent): void {
    if (!this._enabled) return;

    // Don't capture if user is typing in an input field
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

    const action = this.mapKey(e);
    if (action.type !== 'noop') {
      e.preventDefault();
      this.onAction(action);
    }
  }

  private mapKey(e: KeyboardEvent): GameAction {
    const key = e.key;
    const shift = e.shiftKey;

    // Shift+Arrow = move selected army; Arrow = move cursor
    if (shift && key === 'ArrowUp') return { type: 'move_army', dx: 0, dy: -1 };
    if (shift && key === 'ArrowDown') return { type: 'move_army', dx: 0, dy: 1 };
    if (shift && key === 'ArrowLeft') return { type: 'move_army', dx: -1, dy: 0 };
    if (shift && key === 'ArrowRight') return { type: 'move_army', dx: 1, dy: 0 };

    // Arrow keys — cursor movement
    if (key === 'ArrowUp' || key === 'k') return { type: 'move_cursor', dx: 0, dy: -1 };
    if (key === 'ArrowDown' || key === 'j') return { type: 'move_cursor', dx: 0, dy: 1 };
    if (key === 'ArrowLeft' || key === 'h') return { type: 'move_cursor', dx: -1, dy: 0 };
    if (key === 'ArrowRight' || key === 'l') return { type: 'move_cursor', dx: 1, dy: 0 };

    // Diagonal movement (vi keys)
    if (key === 'y') return { type: 'set_highlight', mode: HighlightMode.YourArmy }; // y = your army highlight
    if (key === 'u') return { type: 'move_cursor', dx: 1, dy: -1 };
    if (key === 'b') return { type: 'move_cursor', dx: -1, dy: 1 };
    if (key === 'n' && !shift) return { type: 'set_display', mode: DisplayMode.Nation };

    // Display mode toggles
    if (key === 'd' && !shift) return { type: 'set_display', mode: DisplayMode.Designation };
    if (key === 'r') return { type: 'set_display', mode: DisplayMode.Race };
    if (key === 'M' || (key === 'm' && shift)) return { type: 'set_display', mode: DisplayMode.Move };
    if (key === 'p' && !shift) return { type: 'set_display', mode: DisplayMode.People };
    if (key === 'D' || (key === 'd' && shift)) return { type: 'set_display', mode: DisplayMode.Defense };
    if (key === 'f') return { type: 'set_display', mode: DisplayMode.Food };
    if (key === 'c') return { type: 'set_display', mode: DisplayMode.Contour };
    if (key === 'v') return { type: 'set_display', mode: DisplayMode.Vegetation };
    if (key === 'm' && !shift) return { type: 'set_display', mode: DisplayMode.Metal };
    // 'j' is cursor down (vi), so we use 'J' for jewels/gold display
    if (key === 'J' || (key === 'j' && shift)) return { type: 'set_display', mode: DisplayMode.Gold };
    if (key === 'i') return { type: 'set_display', mode: DisplayMode.Items };

    // Highlight mode toggles
    if (key === 'o') return { type: 'set_highlight', mode: HighlightMode.Own };
    if (key === 'a') return { type: 'set_highlight', mode: HighlightMode.Army };
    if (key === 'x') return { type: 'set_highlight', mode: HighlightMode.None };
    if (key === 's') return { type: 'set_highlight', mode: HighlightMode.Good };
    // 'l' is cursor right (vi), so Shift+L for move-left highlight
    if (key === 'L' || (key === 'l' && shift)) return { type: 'set_highlight', mode: HighlightMode.Move };

    // Army/Navy selection
    if (key === 'Tab') return { type: 'select_next_army' };
    if (key === '`') return { type: 'toggle_army_navy' };

    // Center map
    if (key === 'C' || (key === 'c' && shift && e.ctrlKey)) return { type: 'center_map' };
    if (key === 'g') return { type: 'jump_capitol' };

    // Commands
    if (key === 'R' && shift) return { type: 'redesignate' };
    if (key === 'P' && shift) return { type: 'draft' };
    if (key === 'E') return { type: 'end_turn' };
    if (key === 'S' && shift) return { type: 'show_scores' };
    if (key === 'N' && shift) return { type: 'show_news' };
    if (key === 'B' && shift) return { type: 'show_budget' };
    if (key === '?') return { type: 'show_help' };

    // Chat toggle (T400)
    if (key === 't' || key === 'T') return { type: 'toggle_chat' };

    // Font size
    if (key === '+' || key === '=') return { type: 'font_increase' };
    if (key === '-') return { type: 'font_decrease' };

    return { type: 'noop' };
  }
}
