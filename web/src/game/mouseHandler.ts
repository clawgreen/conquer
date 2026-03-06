// mouseHandler.ts — Mouse/touch pan and zoom for the game map
// Click+drag to pan, mouse wheel to zoom at pointer location
// Touch: one-finger drag to pan, pinch to zoom

import { TerminalRenderer } from '../renderer/terminal';
import { getTileset, getScaledCellSize } from '../renderer/tilesets';

export interface PanZoomCallbacks {
  getOffset(): { x: number; y: number };
  setOffset(x: number, y: number): void;
  getFontSize(): number;
  setFontSize(size: number): void;
  getTilesetId(): string;
}

export class MouseHandler {
  private canvas: HTMLCanvasElement;
  private cb: PanZoomCallbacks;
  private term: TerminalRenderer;

  // Drag state
  private dragging = false;
  private dragStartX = 0;
  private dragStartY = 0;
  private dragOffsetStartX = 0;
  private dragOffsetStartY = 0;

  // Touch pinch state
  private pinchStartDist = 0;
  private pinchStartFontSize = 0;

  constructor(canvas: HTMLCanvasElement, term: TerminalRenderer, cb: PanZoomCallbacks) {
    this.canvas = canvas;
    this.term = term;
    this.cb = cb;

    // Mouse events
    this.canvas.addEventListener('mousedown', this.onMouseDown);
    this.canvas.addEventListener('mousemove', this.onMouseMove);
    this.canvas.addEventListener('mouseup', this.onMouseUp);
    this.canvas.addEventListener('mouseleave', this.onMouseUp);
    this.canvas.addEventListener('wheel', this.onWheel, { passive: false });

    // Touch events
    this.canvas.addEventListener('touchstart', this.onTouchStart, { passive: false });
    this.canvas.addEventListener('touchmove', this.onTouchMove, { passive: false });
    this.canvas.addEventListener('touchend', this.onTouchEnd);

    // Prevent context menu on right-click drag
    this.canvas.addEventListener('contextmenu', (e) => e.preventDefault());
  }

  /** Get cell dimensions based on current tileset, scaled for zoom */
  private getCellSize(): { cw: number; ch: number } {
    const tsId = this.cb.getTilesetId();
    const ts = getTileset(tsId);
    if (ts.tileType !== 'char') {
      return getScaledCellSize(ts, this.cb.getFontSize());
    }
    // Char mode: cell width = 2 chars wide (matching C's 2-column cells)
    return { cw: this.term.cellWidth * 2, ch: this.term.cellHeight };
  }

  // ─── Mouse drag to pan ───

  private onMouseDown = (e: MouseEvent): void => {
    if (e.button !== 0) return; // left click only
    this.dragging = true;
    this.dragStartX = e.clientX;
    this.dragStartY = e.clientY;
    const off = this.cb.getOffset();
    this.dragOffsetStartX = off.x;
    this.dragOffsetStartY = off.y;
    this.canvas.style.cursor = 'grabbing';
  };

  private onMouseMove = (e: MouseEvent): void => {
    if (!this.dragging) {
      this.canvas.style.cursor = 'grab';
      return;
    }
    const { cw, ch } = this.getCellSize();
    const dx = e.clientX - this.dragStartX;
    const dy = e.clientY - this.dragStartY;

    // Convert pixel delta to cell delta (negative because dragging right = scroll left)
    const cellDx = Math.round(-dx / cw);
    const cellDy = Math.round(-dy / ch);

    this.cb.setOffset(
      this.dragOffsetStartX + cellDx,
      this.dragOffsetStartY + cellDy,
    );
  };

  private onMouseUp = (): void => {
    this.dragging = false;
    this.canvas.style.cursor = 'grab';
  };

  // ─── Mouse wheel zoom at pointer ───

  private onWheel = (e: WheelEvent): void => {
    e.preventDefault();

    const rect = this.canvas.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const oldFontSize = this.cb.getFontSize();
    const { cw: oldCW, ch: oldCH } = this.getCellSize();

    // Which map cell is under the mouse?
    const off = this.cb.getOffset();
    const cellUnderMouseX = off.x + mouseX / oldCW;
    const cellUnderMouseY = off.y + mouseY / oldCH;

    // Adjust font size
    const delta = e.deltaY > 0 ? -1 : 1;
    const newFontSize = Math.max(8, Math.min(32, oldFontSize + delta));
    if (newFontSize === oldFontSize) return;

    this.cb.setFontSize(newFontSize);

    // After font change, get new cell size
    const { cw: newCW, ch: newCH } = this.getCellSize();

    // Recalculate offset so the cell under the mouse stays in the same screen position
    const newOffX = Math.round(cellUnderMouseX - mouseX / newCW);
    const newOffY = Math.round(cellUnderMouseY - mouseY / newCH);

    this.cb.setOffset(newOffX, newOffY);
  };

  // ─── Touch: one-finger drag, two-finger pinch zoom ───

  private onTouchStart = (e: TouchEvent): void => {
    if (e.touches.length === 1) {
      // Single finger: drag
      e.preventDefault();
      this.dragging = true;
      this.dragStartX = e.touches[0].clientX;
      this.dragStartY = e.touches[0].clientY;
      const off = this.cb.getOffset();
      this.dragOffsetStartX = off.x;
      this.dragOffsetStartY = off.y;
    } else if (e.touches.length === 2) {
      // Two fingers: pinch zoom
      e.preventDefault();
      this.dragging = false;
      this.pinchStartDist = this.getTouchDist(e.touches);
      this.pinchStartFontSize = this.cb.getFontSize();
    }
  };

  private onTouchMove = (e: TouchEvent): void => {
    if (e.touches.length === 1 && this.dragging) {
      e.preventDefault();
      const { cw, ch } = this.getCellSize();
      const dx = e.touches[0].clientX - this.dragStartX;
      const dy = e.touches[0].clientY - this.dragStartY;
      const cellDx = Math.round(-dx / cw);
      const cellDy = Math.round(-dy / ch);
      this.cb.setOffset(
        this.dragOffsetStartX + cellDx,
        this.dragOffsetStartY + cellDy,
      );
    } else if (e.touches.length === 2) {
      e.preventDefault();
      const dist = this.getTouchDist(e.touches);
      const scale = dist / this.pinchStartDist;
      const newSize = Math.max(8, Math.min(32, Math.round(this.pinchStartFontSize * scale)));
      this.cb.setFontSize(newSize);
    }
  };

  private onTouchEnd = (): void => {
    this.dragging = false;
  };

  private getTouchDist(touches: TouchList): number {
    const dx = touches[0].clientX - touches[1].clientX;
    const dy = touches[0].clientY - touches[1].clientY;
    return Math.sqrt(dx * dx + dy * dy);
  }

  destroy(): void {
    this.canvas.removeEventListener('mousedown', this.onMouseDown);
    this.canvas.removeEventListener('mousemove', this.onMouseMove);
    this.canvas.removeEventListener('mouseup', this.onMouseUp);
    this.canvas.removeEventListener('mouseleave', this.onMouseUp);
    this.canvas.removeEventListener('wheel', this.onWheel);
    this.canvas.removeEventListener('touchstart', this.onTouchStart);
    this.canvas.removeEventListener('touchmove', this.onTouchMove);
    this.canvas.removeEventListener('touchend', this.onTouchEnd);
  }
}
