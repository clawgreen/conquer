// gameLayout.ts — Three-column game layout with CRT bezel, collapsible sidebars

import { getUiTheme, UiTheme } from './uiThemes';

export class GameLayout {
  private root: HTMLElement;
  private _leftBar: HTMLElement;
  private _rightBar: HTMLElement;
  private _canvasWrap: HTMLElement;
  private _canvas: HTMLCanvasElement;
  private _leftToggle: HTMLElement;
  private _rightToggle: HTMLElement;
  private leftVisible = true;
  private rightVisible = true;
  private _uiThemeId = 'terminal';

  constructor(parent: HTMLElement) {
    this.root = document.createElement('div');
    this.root.id = 'game-layout';
    parent.appendChild(this.root);

    // Left sidebar
    this._leftBar = document.createElement('div');
    this._leftBar.id = 'left-sidebar';
    this.root.appendChild(this._leftBar);

    // Center: bezel + canvas
    this._canvasWrap = document.createElement('div');
    this._canvasWrap.id = 'canvas-wrap';

    this._canvas = document.createElement('canvas');
    this._canvas.id = 'game-canvas';
    this._canvasWrap.appendChild(this._canvas);
    this.root.appendChild(this._canvasWrap);

    // Right sidebar
    this._rightBar = document.createElement('div');
    this._rightBar.id = 'right-sidebar';
    this.root.appendChild(this._rightBar);

    // Toggle buttons (float on canvas edges)
    this._leftToggle = document.createElement('button');
    this._leftToggle.className = 'sidebar-toggle left-toggle';
    this._leftToggle.textContent = '◀';
    this._leftToggle.addEventListener('click', () => this.toggleLeft());
    this._canvasWrap.appendChild(this._leftToggle);

    this._rightToggle = document.createElement('button');
    this._rightToggle.className = 'sidebar-toggle right-toggle';
    this._rightToggle.textContent = '▶';
    this._rightToggle.addEventListener('click', () => this.toggleRight());
    this._canvasWrap.appendChild(this._rightToggle);

    this.applyStyles();
    this.applyTheme();
  }

  get canvas(): HTMLCanvasElement { return this._canvas; }
  get leftBar(): HTMLElement { return this._leftBar; }
  get rightBar(): HTMLElement { return this._rightBar; }

  get uiThemeId(): string { return this._uiThemeId; }
  set uiThemeId(id: string) {
    this._uiThemeId = id;
    this.applyTheme();
    localStorage.setItem('conquer_ui_theme', id);
  }

  private get isMobile(): boolean { return window.innerWidth <= 600; }

  toggleLeft(): void {
    this.leftVisible = !this.leftVisible;
    if (this.isMobile) {
      this._leftBar.classList.toggle('mobile-hidden', !this.leftVisible);
      if (this.leftVisible) this._leftBar.style.display = 'flex';
    } else {
      this._leftBar.style.display = this.leftVisible ? 'flex' : 'none';
    }
    this._leftToggle.textContent = this.leftVisible ? '◀' : '▶';
    this.onResize();
  }

  toggleRight(): void {
    this.rightVisible = !this.rightVisible;
    if (this.isMobile) {
      this._rightBar.classList.toggle('mobile-hidden', !this.rightVisible);
      if (this.rightVisible) this._rightBar.style.display = 'flex';
    } else {
      this._rightBar.style.display = this.rightVisible ? 'flex' : 'none';
    }
    this._rightToggle.textContent = this.rightVisible ? '▶' : '◀';
    this.onResize();
  }

  /** Call after construction to set initial mobile state */
  initMobileState(): void {
    if (this.isMobile) {
      this.leftVisible = false;
      this.rightVisible = false;
      this._leftBar.classList.add('mobile-hidden');
      this._rightBar.classList.add('mobile-hidden');
      this._leftBar.style.display = 'flex';
      this._rightBar.style.display = 'flex';
      this._leftToggle.textContent = '▶';
      this._rightToggle.textContent = '◀';
    }
  }

  onResize(): void {
    // Canvas fills available space in center
    const rect = this._canvasWrap.getBoundingClientRect();
    this._canvas.width = rect.width;
    this._canvas.height = rect.height;
  }

  private applyStyles(): void {
    const style = document.createElement('style');
    style.textContent = `
      #game-layout {
        display: flex; width: 100vw; height: 100vh; overflow: hidden;
      }
      #left-sidebar, #right-sidebar {
        display: flex; flex-direction: column; overflow-y: auto;
        width: 200px; min-width: 160px; flex-shrink: 0;
        -webkit-overflow-scrolling: touch;
      }
      #canvas-wrap {
        flex: 1; position: relative; overflow: hidden;
        display: flex; align-items: stretch; justify-content: center;
        padding: 6px;
      }
      #game-canvas {
        display: block; width: 100%; height: 100%; border-radius: 4px;
      }
      .sidebar-toggle {
        position: absolute; top: 50%; transform: translateY(-50%);
        z-index: 10; background: rgba(0,0,0,0.7); border: 1px solid #333;
        color: #888; font-size: 14px; padding: 8px 4px; cursor: pointer;
        border-radius: 3px; font-family: inherit; line-height: 1;
      }
      .sidebar-toggle:hover { color: #fff; border-color: #555; }
      .left-toggle { left: 2px; }
      .right-toggle { right: 2px; }

      .cmd-group { margin-bottom: 2px; }
      .cmd-group-header {
        padding: 4px 8px; font-size: 10px; text-transform: uppercase;
        letter-spacing: 1px; cursor: pointer; user-select: none;
      }
      .cmd-group-header:hover { opacity: 0.8; }
      .cmd-btn {
        display: block; width: 100%; text-align: left;
        padding: 5px 8px; font-size: 11px; font-family: inherit;
        cursor: pointer; border: none; border-left: 2px solid transparent;
        transition: background 0.1s, border-color 0.15s;
      }
      .cmd-btn:hover { opacity: 0.9; }
      .cmd-btn.flash {
        transition: none;
      }

      .stat-row {
        display: flex; justify-content: space-between;
        padding: 2px 8px; font-size: 11px;
      }
      .stat-label { opacity: 0.6; }
      .stat-value { font-weight: bold; }
      .stat-section {
        padding: 6px 0; border-bottom: 1px solid transparent;
      }
      .stat-section-title {
        padding: 4px 8px; font-size: 10px; text-transform: uppercase;
        letter-spacing: 1px;
      }

      @media (max-width: 768px) {
        #left-sidebar, #right-sidebar { width: 150px; min-width: 120px; }
        .cmd-btn { padding: 6px 8px; font-size: 12px; min-height: 36px; }
      }
      @media (max-width: 600px) {
        /* Sidebars start hidden on mobile but can be toggled via buttons */
        #left-sidebar.mobile-hidden { display: none !important; }
        #right-sidebar.mobile-hidden { display: none !important; }
        /* Make sidebars overlay on mobile instead of pushing content */
        #left-sidebar:not(.mobile-hidden) {
          position: absolute; left: 0; top: 0; bottom: 0;
          z-index: 20; box-shadow: 4px 0 12px rgba(0,0,0,0.5);
        }
        #right-sidebar:not(.mobile-hidden) {
          position: absolute; right: 0; top: 0; bottom: 0;
          z-index: 20; box-shadow: -4px 0 12px rgba(0,0,0,0.5);
        }
        .sidebar-toggle { padding: 12px 6px; font-size: 18px; }
      }
    `;
    document.head.appendChild(style);
  }

  private applyTheme(): void {
    const t = getUiTheme(this._uiThemeId);
    this._leftBar.style.cssText = `display:${this.leftVisible ? 'flex' : 'none'};flex-direction:column;width:200px;min-width:160px;flex-shrink:0;overflow-y:auto;background:${t.sidebarBg};border-right:1px solid ${t.sidebarBorder};color:${t.sidebarText};`;
    this._rightBar.style.cssText = `display:${this.rightVisible ? 'flex' : 'none'};flex-direction:column;width:220px;min-width:170px;flex-shrink:0;overflow-y:auto;background:${t.sidebarBg};border-left:1px solid ${t.sidebarBorder};color:${t.sidebarText};`;
    this._canvasWrap.style.background = t.bezelBg;
    this._canvasWrap.style.boxShadow = t.bezelShadow;
    this._canvas.style.boxShadow = t.screenGlow;
    this._canvas.style.borderRadius = t.bezelRadius;
  }

  destroy(): void {
    this.root.remove();
  }
}
