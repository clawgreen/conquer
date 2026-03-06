// touchHandler.ts — Touch gesture handling on the game canvas
// Detects: tap, double-tap, pinch-zoom, two-finger pan
// Dispatches GameAction events through the same system as keyboard

import { GameAction } from './inputHandler';

const TAP_THRESHOLD = 200;       // ms max for a tap
const TAP_MOVE_THRESHOLD = 10;   // px max movement for a tap
const DOUBLE_TAP_DELAY = 300;    // ms max between taps for double-tap
const PINCH_THRESHOLD = 10;      // px change before triggering zoom

interface TouchPoint {
  x: number;
  y: number;
  time: number;
}

export class TouchHandler {
  private canvas: HTMLCanvasElement;
  private onAction: (action: GameAction) => void;
  private getCellSize: () => { cellW: number; cellH: number };
  private getViewport: () => { xOffset: number; yOffset: number };

  // State
  private touchStart: TouchPoint | null = null;
  private lastTap: TouchPoint | null = null;
  private pinchStartDist: number = 0;
  private isPinching = false;
  private isPanning = false;
  private panLastX = 0;
  private panLastY = 0;
  private panAccX = 0;
  private panAccY = 0;
  private _enabled = true;

  constructor(
    canvas: HTMLCanvasElement,
    onAction: (action: GameAction) => void,
    getCellSize: () => { cellW: number; cellH: number },
    getViewport: () => { xOffset: number; yOffset: number },
  ) {
    this.canvas = canvas;
    this.onAction = onAction;
    this.getCellSize = getCellSize;
    this.getViewport = getViewport;

    this.onTouchStart = this.onTouchStart.bind(this);
    this.onTouchMove = this.onTouchMove.bind(this);
    this.onTouchEnd = this.onTouchEnd.bind(this);

    canvas.addEventListener('touchstart', this.onTouchStart, { passive: false });
    canvas.addEventListener('touchmove', this.onTouchMove, { passive: false });
    canvas.addEventListener('touchend', this.onTouchEnd, { passive: false });
    canvas.addEventListener('touchcancel', this.onTouchEnd, { passive: false });
  }

  set enabled(v: boolean) { this._enabled = v; }

  destroy(): void {
    this.canvas.removeEventListener('touchstart', this.onTouchStart);
    this.canvas.removeEventListener('touchmove', this.onTouchMove);
    this.canvas.removeEventListener('touchend', this.onTouchEnd);
    this.canvas.removeEventListener('touchcancel', this.onTouchEnd);
  }

  private onTouchStart(e: TouchEvent): void {
    if (!this._enabled) return;
    e.preventDefault();

    if (e.touches.length === 1) {
      // Single finger — potential tap or drag
      const t = e.touches[0];
      this.touchStart = { x: t.clientX, y: t.clientY, time: Date.now() };
      this.isPinching = false;
      this.isPanning = false;
    } else if (e.touches.length === 2) {
      // Two fingers — pinch or pan
      this.isPinching = true;
      this.isPanning = true;
      this.pinchStartDist = this.getTouchDist(e.touches[0], e.touches[1]);
      const mid = this.getTouchMidpoint(e.touches[0], e.touches[1]);
      this.panLastX = mid.x;
      this.panLastY = mid.y;
      this.panAccX = 0;
      this.panAccY = 0;
      this.touchStart = null; // Cancel any tap
    }
  }

  private onTouchMove(e: TouchEvent): void {
    if (!this._enabled) return;
    e.preventDefault();

    if (e.touches.length === 2 && this.isPinching) {
      const dist = this.getTouchDist(e.touches[0], e.touches[1]);
      const delta = dist - this.pinchStartDist;

      if (Math.abs(delta) > PINCH_THRESHOLD) {
        if (delta > 0) {
          this.onAction({ type: 'font_increase' });
        } else {
          this.onAction({ type: 'font_decrease' });
        }
        this.pinchStartDist = dist;
      }

      // Two-finger pan
      const mid = this.getTouchMidpoint(e.touches[0], e.touches[1]);
      const { cellW, cellH } = this.getCellSize();
      // Accumulate sub-cell movement
      this.panAccX += this.panLastX - mid.x;
      this.panAccY += this.panLastY - mid.y;
      this.panLastX = mid.x;
      this.panLastY = mid.y;

      // Move in cell-sized steps (each map cell = 2 columns wide)
      const mapCellW = cellW * 2;
      while (this.panAccX > mapCellW) {
        this.onAction({ type: 'move_cursor', dx: 1, dy: 0 });
        this.panAccX -= mapCellW;
      }
      while (this.panAccX < -mapCellW) {
        this.onAction({ type: 'move_cursor', dx: -1, dy: 0 });
        this.panAccX += mapCellW;
      }
      while (this.panAccY > cellH) {
        this.onAction({ type: 'move_cursor', dx: 0, dy: 1 });
        this.panAccY -= cellH;
      }
      while (this.panAccY < -cellH) {
        this.onAction({ type: 'move_cursor', dx: 0, dy: -1 });
        this.panAccY += cellH;
      }
    } else if (e.touches.length === 1 && this.touchStart) {
      // Single finger drag — check if moved too far for a tap
      const t = e.touches[0];
      const dx = t.clientX - this.touchStart.x;
      const dy = t.clientY - this.touchStart.y;
      if (Math.abs(dx) > TAP_MOVE_THRESHOLD * 3 || Math.abs(dy) > TAP_MOVE_THRESHOLD * 3) {
        // Cancel tap — this is a drag (we don't do anything with single-finger drag currently)
        this.touchStart = null;
      }
    }
  }

  private onTouchEnd(e: TouchEvent): void {
    if (!this._enabled) return;
    e.preventDefault();

    if (this.isPinching || this.isPanning) {
      if (e.touches.length === 0) {
        this.isPinching = false;
        this.isPanning = false;
      }
      return;
    }

    if (!this.touchStart) return;

    const now = Date.now();
    const elapsed = now - this.touchStart.time;

    // Check if it was a tap (short duration, minimal movement)
    if (elapsed < TAP_THRESHOLD) {
      const tapPoint = this.touchStart;

      // Check for double-tap
      if (this.lastTap && (now - this.lastTap.time) < DOUBLE_TAP_DELAY) {
        // Double-tap — select army at position
        this.handleDoubleTap(tapPoint.x, tapPoint.y);
        this.lastTap = null;
      } else {
        // Single tap — move cursor to position
        this.handleTap(tapPoint.x, tapPoint.y);
        this.lastTap = { x: tapPoint.x, y: tapPoint.y, time: now };
      }
    }

    this.touchStart = null;
  }

  private handleTap(screenX: number, screenY: number): void {
    // Convert screen coordinates to canvas-relative coordinates
    const rect = this.canvas.getBoundingClientRect();
    const canvasX = screenX - rect.left;
    const canvasY = screenY - rect.top;

    const { cellW, cellH } = this.getCellSize();
    // Each map cell = 2 terminal columns
    const mapCellW = cellW * 2;

    const gridX = Math.floor(canvasX / mapCellW);
    const gridY = Math.floor(canvasY / cellH);

    // We need to move cursor to this grid position
    // Calculate delta from current cursor position
    // The gameScreen tracks cursorX/cursorY so we dispatch absolute positioning
    // via multiple move_cursor actions... or we can use a special tap action.
    // For simplicity, dispatch a move to the tapped cell by calculating deltas.
    // But we don't have access to cursor state here. Instead, we'll use the
    // onTapPosition callback pattern.
    if (this.onTapCallback) {
      this.onTapCallback(gridX, gridY);
    }
  }

  private handleDoubleTap(screenX: number, screenY: number): void {
    const rect = this.canvas.getBoundingClientRect();
    const canvasX = screenX - rect.left;
    const canvasY = screenY - rect.top;

    const { cellW, cellH } = this.getCellSize();
    const mapCellW = cellW * 2;

    const gridX = Math.floor(canvasX / mapCellW);
    const gridY = Math.floor(canvasY / cellH);

    if (this.onDoubleTapCallback) {
      this.onDoubleTapCallback(gridX, gridY);
    }
  }

  // Callbacks for tap positions (set by GameScreen)
  private onTapCallback: ((gridX: number, gridY: number) => void) | null = null;
  private onDoubleTapCallback: ((gridX: number, gridY: number) => void) | null = null;

  onTap(cb: (gridX: number, gridY: number) => void): void {
    this.onTapCallback = cb;
  }

  onDoubleTap(cb: (gridX: number, gridY: number) => void): void {
    this.onDoubleTapCallback = cb;
  }

  private getTouchDist(a: Touch, b: Touch): number {
    const dx = a.clientX - b.clientX;
    const dy = a.clientY - b.clientY;
    return Math.sqrt(dx * dx + dy * dy);
  }

  private getTouchMidpoint(a: Touch, b: Touch): { x: number; y: number } {
    return {
      x: (a.clientX + b.clientX) / 2,
      y: (a.clientY + b.clientY) / 2,
    };
  }
}
