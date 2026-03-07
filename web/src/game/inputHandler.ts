// inputHandler.ts — Keyboard input handling
// T354-T372: Map navigation, army commands, display/highlight toggles, command keys

import { GameState } from '../state/gameState';
import { DisplayMode, HighlightMode } from '../types';

export type GameAction =
  | { type: 'move_cursor'; dx: number; dy: number }
  | { type: 'move_or_cursor'; dx: number; dy: number }
  | { type: 'center_map' }
  | { type: 'jump_capitol' }
  | { type: 'set_display'; mode: DisplayMode }
  | { type: 'set_highlight'; mode: HighlightMode }
  | { type: 'select_next_army' }
  | { type: 'select_prev_army' }
  | { type: 'toggle_army_navy' }
  | { type: 'move_army'; dx: number; dy: number }
  | { type: 'toggle_movement_mode' }
  | { type: 'exit_movement_mode' }
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
  | { type: 'set_army_attack' }
  | { type: 'set_army_defend' }
  | { type: 'set_army_garrison' }
  | { type: 'set_army_scout' }
  | { type: 'set_army_rule' }
  | { type: 'set_army_march' }
  | { type: 'split_army' }
  | { type: 'combine_army' }
  | { type: 'divide_army' }
  | { type: 'build_fort' }
  | { type: 'build_road' }
  | { type: 'build_ship' }
  | { type: 'load_fleet' }
  | { type: 'unload_fleet' }
  | { type: 'cast_spell' }
  | { type: 'buy_power' }
  | { type: 'propose_trade' }
  | { type: 'hire_mercs' }
  | { type: 'bribe' }
  | { type: 'send_tribute' }
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

    // Arrow keys — in movement mode they move the army, otherwise the cursor
    if (key === 'ArrowUp' || key === 'k') return { type: 'move_or_cursor', dx: 0, dy: -1 };
    if (key === 'ArrowDown' || key === 'j') return { type: 'move_or_cursor', dx: 0, dy: 1 };
    if (key === 'ArrowLeft' || key === 'h') return { type: 'move_or_cursor', dx: -1, dy: 0 };
    if (key === 'ArrowRight' || key === 'l') return { type: 'move_or_cursor', dx: 1, dy: 0 };

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
    if (key === 'f' && !shift) return { type: 'set_display', mode: DisplayMode.Food };
    if (key === 'c' && !shift) return { type: 'set_display', mode: DisplayMode.Contour };
    if (key === 'v') return { type: 'set_display', mode: DisplayMode.Vegetation };
    if (key === 'J' || (key === 'j' && shift)) return { type: 'set_display', mode: DisplayMode.Gold };
    if (key === 'i') return { type: 'set_display', mode: DisplayMode.Items };

    // Highlight mode toggles
    if (key === 'o') return { type: 'set_highlight', mode: HighlightMode.Own };
    if (key === 'a' && !shift) return { type: 'set_highlight', mode: HighlightMode.Army };
    if (key === 'x') return { type: 'set_highlight', mode: HighlightMode.None };
    if (key === 's' && !shift) return { type: 'set_highlight', mode: HighlightMode.Good };
    if (key === 'L' || (key === 'l' && shift)) return { type: 'set_highlight', mode: HighlightMode.Move };

    // Army/Navy selection
    if (key === 'Tab') return { type: 'select_next_army' };
    if (key === '`') return { type: 'toggle_army_navy' };

    // Center map
    if (key === 'C' || (key === 'c' && shift && e.ctrlKey)) return { type: 'center_map' };
    if (key === 'g' && !shift) return { type: 'jump_capitol' };

    // Movement mode (like original: m = enter move mode, space = done)
    if (key === 'm' && !shift) return { type: 'toggle_movement_mode' };
    if (key === ' ') return { type: 'exit_movement_mode' };

    // Commands
    if (key === 'R' && shift) return { type: 'redesignate' };
    if (key === 'P' && shift) return { type: 'draft' };
    if (key === 'F' && shift) return { type: 'build_fort' };
    if (key === 'W' && shift) return { type: 'build_road' };
    if (key === 'E') return { type: 'end_turn' };
    if (key === 'S' && shift) return { type: 'show_scores' };
    if (key === 'N' && shift) return { type: 'show_news' };
    if (key === 'B' && shift) return { type: 'show_budget' };
    if (key === 'X' && shift) return { type: 'diplomacy' };
    if (key === 'Z' && shift) return { type: 'cast_spell' };
    if (key === 'Q' && shift) return { type: 'buy_power' };
    if (key === '$') return { type: 'propose_trade' };
    if (key === '?') return { type: 'show_help' };

    // Chat toggle (T400)
    if (key === 't' || key === 'T') return { type: 'toggle_chat' };

    // Font size — only '+' and '=' (not '-' which is split_army)
    if (key === '+' || key === '=') return { type: 'font_increase' };

    // Army status commands (context-sensitive — gameScreen checks if army is selected)
    if (key === 'A' && shift) return { type: 'set_army_attack' };
    // 'd' without shift is display mode; army defend via sidebar
    if (key === 'G' && shift) return { type: 'set_army_garrison' };
    // 's' without shift is highlight; army scout via sidebar
    if (key === '-') return { type: 'split_army' };
    // '+' handled above as font_increase; combine via sidebar
    if (key === '/') return { type: 'divide_army' };

    return { type: 'noop' };
  }
}
