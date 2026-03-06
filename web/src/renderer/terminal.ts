// terminal.ts — TerminalRenderer: Canvas 2D engine rendering a character grid
// T333-T339: Monospace character grid, color, cursor, resize, standout

import { CURSES_COLORS } from './colors';

export interface CellAttrs {
  ch: string;        // single character
  fg: string;        // foreground color (hex)
  bg: string;        // background color (hex)
  bold: boolean;
  inverse: boolean;
  blink: boolean;
}

function defaultCell(): CellAttrs {
  return {
    ch: ' ',
    fg: CURSES_COLORS.white,
    bg: CURSES_COLORS.black,
    bold: false,
    inverse: false,
    blink: false,
  };
}

export class TerminalRenderer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private _cols: number = 80;
  private _rows: number = 24;
  private _cellW: number = 0;
  private _cellH: number = 0;
  private _fontSize: number = 16;
  private _fontFamily: string = '"Courier New", "Consolas", "Liberation Mono", monospace';
  private grid: CellAttrs[][] = [];
  private cursorX: number = 0;
  private cursorY: number = 0;
  private cursorVisible: boolean = true;
  private cursorBlinkOn: boolean = true;
  private blinkTimer: number = 0;
  private _dirty: boolean = true;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    const ctx = canvas.getContext('2d');
    if (!ctx) throw new Error('Cannot get 2D context');
    this.ctx = ctx;
    this.measureCell();
    this.initGrid();
    this.startBlinkTimer();
  }

  get cols(): number { return this._cols; }
  get rows(): number { return this._rows; }
  get cellW(): number { return this._cellW; }
  get cellH(): number { return this._cellH; }
  get cellWidth(): number { return this._cellW; }
  get cellHeight(): number { return this._cellH; }
  get fontSize(): number { return this._fontSize; }

  /** Expose canvas context for direct rendering (emoji/image tilesets) */
  getContext(): CanvasRenderingContext2D { return this.ctx; }

  setFontSize(size: number): void {
    this._fontSize = Math.max(8, Math.min(32, size));
    this.measureCell();
    this.resize(this.canvas.width, this.canvas.height);
  }

  private measureCell(): void {
    this.ctx.font = `${this._fontSize}px ${this._fontFamily}`;
    const m = this.ctx.measureText('M');
    // Round to integer pixels to prevent sub-pixel drift across the grid
    this._cellW = Math.round(m.width);
    this._cellH = Math.round(this._fontSize * 1.2);
  }

  private initGrid(): void {
    this.grid = [];
    for (let y = 0; y < this._rows; y++) {
      const row: CellAttrs[] = [];
      for (let x = 0; x < this._cols; x++) {
        row.push(defaultCell());
      }
      this.grid.push(row);
    }
    this._dirty = true;
  }

  resize(width: number, height: number): void {
    this.canvas.width = width;
    this.canvas.height = height;
    const newCols = Math.floor(width / this._cellW);
    const newRows = Math.floor(height / this._cellH);
    if (newCols !== this._cols || newRows !== this._rows) {
      this._cols = Math.max(1, newCols);
      this._rows = Math.max(1, newRows);
      this.initGrid();
    }
    this._dirty = true;
  }

  /** Set a character cell. x = column, y = row. */
  setCell(x: number, y: number, attrs: Partial<CellAttrs>): void {
    if (x < 0 || x >= this._cols || y < 0 || y >= this._rows) return;
    const cell = this.grid[y][x];
    if (attrs.ch !== undefined) cell.ch = attrs.ch;
    if (attrs.fg !== undefined) cell.fg = attrs.fg;
    if (attrs.bg !== undefined) cell.bg = attrs.bg;
    if (attrs.bold !== undefined) cell.bold = attrs.bold;
    if (attrs.inverse !== undefined) cell.inverse = attrs.inverse;
    if (attrs.blink !== undefined) cell.blink = attrs.blink;
    this._dirty = true;
  }

  /** Write a string starting at (x, y). */
  writeStr(x: number, y: number, str: string, fg?: string, bg?: string, bold?: boolean): void {
    for (let i = 0; i < str.length; i++) {
      this.setCell(x + i, y, {
        ch: str[i],
        fg: fg ?? CURSES_COLORS.white,
        bg: bg ?? CURSES_COLORS.black,
        bold: bold ?? false,
        inverse: false,
      });
    }
  }

  /** Clear entire screen. */
  clear(): void {
    for (let y = 0; y < this._rows; y++) {
      for (let x = 0; x < this._cols; x++) {
        this.grid[y][x] = defaultCell();
      }
    }
    this._dirty = true;
  }

  /** Clear a row from col start to end. */
  clearRow(y: number, fromX: number = 0, toX?: number): void {
    const end = toX ?? this._cols;
    for (let x = fromX; x < end && x < this._cols; x++) {
      if (y >= 0 && y < this._rows) {
        this.grid[y][x] = defaultCell();
      }
    }
    this._dirty = true;
  }

  setCursor(x: number, y: number): void {
    this.cursorX = x;
    this.cursorY = y;
    this._dirty = true;
  }

  setCursorVisible(v: boolean): void {
    this.cursorVisible = v;
    this._dirty = true;
  }

  private startBlinkTimer(): void {
    this.blinkTimer = window.setInterval(() => {
      this.cursorBlinkOn = !this.cursorBlinkOn;
      this._dirty = true;
    }, 530);
  }

  destroy(): void {
    if (this.blinkTimer) clearInterval(this.blinkTimer);
  }

  /**
   * Render only the bottom panel rows, starting at canvasY pixel position.
   * Used when emoji/image tilesets handle the map area directly.
   */
  renderPartial(canvasY: number): void {
    const ctx = this.ctx;
    ctx.textBaseline = 'top';

    // Render only the bottom 3 rows (panel area)
    const startRow = Math.max(0, this._rows - 3);
    for (let y = startRow; y < this._rows; y++) {
      const rowIdx = y - startRow;
      for (let x = 0; x < this._cols; x++) {
        const cell = this.grid[y][x];
        const px = x * this._cellW;
        const py = canvasY + rowIdx * this._cellH;

        let fg = cell.fg;
        let bg = cell.bg;
        if (cell.inverse) { [fg, bg] = [bg, fg]; }

        if (bg !== CURSES_COLORS.black && bg !== '#000000' && bg !== '#000') {
          ctx.fillStyle = bg;
          ctx.fillRect(px, py, this._cellW, this._cellH);
        }

        if (cell.ch !== ' ') {
          ctx.font = `${cell.bold ? 'bold ' : ''}${this._fontSize}px ${this._fontFamily}`;
          ctx.fillStyle = fg;
          ctx.fillText(cell.ch, px, py);
        }
      }
    }
  }

  /** Render all dirty cells to canvas. */
  render(): void {
    if (!this._dirty) return;
    this._dirty = false;

    const ctx = this.ctx;
    // Clear canvas
    ctx.fillStyle = CURSES_COLORS.black;
    ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);

    ctx.textBaseline = 'top';

    for (let y = 0; y < this._rows; y++) {
      for (let x = 0; x < this._cols; x++) {
        const cell = this.grid[y][x];
        const px = x * this._cellW;
        const py = y * this._cellH;

        let fg = cell.fg;
        let bg = cell.bg;
        if (cell.inverse) {
          [fg, bg] = [bg, fg];
        }

        // Draw background
        if (bg !== CURSES_COLORS.black) {
          ctx.fillStyle = bg;
          ctx.fillRect(px, py, this._cellW, this._cellH);
        }

        // Draw character
        if (cell.ch !== ' ') {
          ctx.font = `${cell.bold ? 'bold ' : ''}${this._fontSize}px ${this._fontFamily}`;
          ctx.fillStyle = fg;
          ctx.fillText(cell.ch, px, py);
        }
      }
    }

    // Cursor — highlight spans TWO cells (one map tile = 2 char cells)
    if (this.cursorVisible && this.cursorBlinkOn &&
        this.cursorX >= 0 && this.cursorX < this._cols &&
        this.cursorY >= 0 && this.cursorY < this._rows) {
      const px = this.cursorX * this._cellW;
      const py = this.cursorY * this._cellH;
      // Cursor spans 2 cells (one map sector = char + padding)
      const cursorW = this._cellW * 2;
      ctx.fillStyle = CURSES_COLORS.brightGreen;
      ctx.globalAlpha = 0.4;
      ctx.fillRect(px, py, cursorW, this._cellH);
      ctx.globalAlpha = 1.0;
      // Redraw character on cursor in contrasting color
      const cell = this.grid[this.cursorY][this.cursorX];
      if (cell.ch !== ' ') {
        ctx.font = `${cell.bold ? 'bold ' : ''}${this._fontSize}px ${this._fontFamily}`;
        ctx.fillStyle = CURSES_COLORS.black;
        ctx.fillText(cell.ch, px, py);
      }
    }
  }

  markDirty(): void {
    this._dirty = true;
  }
}
