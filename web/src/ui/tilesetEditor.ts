// tilesetEditor.ts — In-game tileset/theme editor
// Copy a built-in tileset, rename it, customize individual tile mappings

import { TileSet, TileDef, ALL_TILESETS, getTileset } from '../renderer/tilesets';

interface CustomTileset {
  tileset: TileSet;
  savedAt: number;
}

const STORAGE_KEY = 'conquer_custom_tilesets';

/** Load user's custom tilesets from localStorage */
export function loadCustomTilesets(): TileSet[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const items: CustomTileset[] = JSON.parse(raw);
    return items.map(i => i.tileset);
  } catch { return []; }
}

/** Save a custom tileset */
export function saveCustomTileset(ts: TileSet): void {
  const all = loadCustomTilesets().filter(t => t.id !== ts.id);
  all.push(ts);
  const items: CustomTileset[] = all.map(t => ({ tileset: t, savedAt: Date.now() }));
  localStorage.setItem(STORAGE_KEY, JSON.stringify(items));
}

/** Delete a custom tileset */
export function deleteCustomTileset(id: string): void {
  const all = loadCustomTilesets().filter(t => t.id !== id);
  const items: CustomTileset[] = all.map(t => ({ tileset: t, savedAt: Date.now() }));
  localStorage.setItem(STORAGE_KEY, JSON.stringify(items));
}

// Terrain category labels for the editor
const ELEVATION_LABELS = ['Water', 'Peak', 'Mountain', 'Hill', 'Flat'];
const VEGETATION_LABELS = [
  'Vine', 'Desert', 'Tree', 'Brush', 'Lush', 'Grain',
  'Water Veg', 'Forest', 'Jungle', 'Swamp', 'Ice', 'Tundra'
];
const DESIGNATION_LABELS = [
  'Town', 'City', 'Mine', 'Farm', 'Fishery', 'Capital', 'Fort', 'Castle',
  'Stockade', 'Capitol', 'Unknown', 'Logging', 'Bridge', 'Road',
  'Gold Mine', 'Granary', 'University', 'Harbor', 'Construction', 'Palace'
];

export class TilesetEditor {
  private overlay: HTMLDivElement;
  private editing: TileSet;
  private onSave: (ts: TileSet) => void;
  private onClose: () => void;

  constructor(
    parent: HTMLElement,
    baseId: string,
    onSave: (ts: TileSet) => void,
    onClose: () => void,
  ) {
    this.onSave = onSave;
    this.onClose = onClose;

    // Deep clone the base tileset
    const base = getTileset(baseId) ?? ALL_TILESETS[0];
    this.editing = JSON.parse(JSON.stringify(base));
    this.editing.id = `custom-${Date.now()}`;
    this.editing.name = `${base.name} (Custom)`;

    this.overlay = document.createElement('div');
    this.overlay.style.cssText = `
      position:fixed;top:0;left:0;right:0;bottom:0;z-index:200;
      background:rgba(0,0,0,0.9);overflow-y:auto;padding:20px;
      font-family:"Courier New",monospace;color:#aaa;
    `;
    parent.appendChild(this.overlay);
    this.render();
  }

  private render(): void {
    const ts = this.editing;
    this.overlay.innerHTML = `
      <div style="max-width:700px;margin:0 auto;">
        <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:16px;">
          <h2 style="margin:0;color:#55ff55;">Tileset Editor</h2>
          <button id="tse-close" style="background:#300;color:#f55;border:1px solid #f55;padding:6px 12px;cursor:pointer;font-family:inherit;">✕ Close</button>
        </div>

        <div style="margin-bottom:12px;">
          <label style="color:#888;">Name:</label>
          <input id="tse-name" value="${this.esc(ts.name)}" style="background:#111;color:#55ff55;border:1px solid #333;padding:4px 8px;font-family:inherit;width:250px;margin-left:8px;" />
        </div>

        <div style="margin-bottom:12px;display:flex;gap:16px;">
          <div>
            <label style="color:#888;">Type:</label>
            <select id="tse-type" style="background:#111;color:#55ff55;border:1px solid #333;padding:4px;font-family:inherit;margin-left:8px;">
              <option value="char" ${ts.tileType === 'char' ? 'selected' : ''}>Character</option>
              <option value="emoji" ${ts.tileType === 'emoji' ? 'selected' : ''}>Emoji</option>
              <option value="image" ${ts.tileType === 'image' ? 'selected' : ''}>Image</option>
            </select>
          </div>
          <div>
            <label style="color:#888;">Cell W:</label>
            <input id="tse-cw" type="number" value="${ts.cellWidth}" min="8" max="64" style="background:#111;color:#55ff55;border:1px solid #333;padding:4px;width:50px;font-family:inherit;margin-left:4px;" />
          </div>
          <div>
            <label style="color:#888;">Cell H:</label>
            <input id="tse-ch" type="number" value="${ts.cellHeight}" min="8" max="64" style="background:#111;color:#55ff55;border:1px solid #333;padding:4px;width:50px;font-family:inherit;margin-left:4px;" />
          </div>
        </div>

        <h3 style="color:#55ff55;border-bottom:1px solid #333;padding-bottom:4px;">Elevation</h3>
        ${this.renderTileGroup(ts.elevation, ELEVATION_LABELS, 'elev')}

        <h3 style="color:#55ff55;border-bottom:1px solid #333;padding-bottom:4px;">Vegetation</h3>
        ${this.renderTileGroup(ts.vegetation, VEGETATION_LABELS, 'veg')}

        <h3 style="color:#55ff55;border-bottom:1px solid #333;padding-bottom:4px;">Designation</h3>
        ${this.renderTileGroup(ts.designation, DESIGNATION_LABELS, 'des')}

        <h3 style="color:#55ff55;border-bottom:1px solid #333;padding-bottom:4px;">Special</h3>
        ${this.renderSingleTile(ts.army, 'Army', 'sp-army')}
        ${this.renderSingleTile(ts.navy, 'Navy', 'sp-navy')}
        ${this.renderSingleTile(ts.cursor, 'Cursor', 'sp-cursor')}
        ${this.renderSingleTile(ts.fog, 'Fog', 'sp-fog')}

        <div style="margin-top:20px;display:flex;gap:12px;">
          <button id="tse-save" style="background:#030;color:#5f5;border:1px solid #5f5;padding:8px 20px;cursor:pointer;font-family:inherit;font-size:14px;">💾 Save Tileset</button>
          <button id="tse-cancel" style="background:#111;color:#888;border:1px solid #333;padding:8px 20px;cursor:pointer;font-family:inherit;">Cancel</button>
        </div>
      </div>
    `;

    // Wire events
    this.overlay.querySelector('#tse-close')!.addEventListener('click', () => this.close());
    this.overlay.querySelector('#tse-cancel')!.addEventListener('click', () => this.close());
    this.overlay.querySelector('#tse-save')!.addEventListener('click', () => this.save());

    this.overlay.querySelector('#tse-name')!.addEventListener('input', (e) => {
      this.editing.name = (e.target as HTMLInputElement).value;
    });
    this.overlay.querySelector('#tse-type')!.addEventListener('change', (e) => {
      this.editing.tileType = (e.target as HTMLSelectElement).value as any;
    });
    this.overlay.querySelector('#tse-cw')!.addEventListener('change', (e) => {
      this.editing.cellWidth = parseInt((e.target as HTMLInputElement).value) || 14;
    });
    this.overlay.querySelector('#tse-ch')!.addEventListener('change', (e) => {
      this.editing.cellHeight = parseInt((e.target as HTMLInputElement).value) || 16;
    });

    // Wire tile editing inputs
    this.wireTileInputs('elev', this.editing.elevation, ELEVATION_LABELS);
    this.wireTileInputs('veg', this.editing.vegetation, VEGETATION_LABELS);
    this.wireTileInputs('des', this.editing.designation, DESIGNATION_LABELS);
    this.wireSingleTile('sp-army', this.editing, 'army');
    this.wireSingleTile('sp-navy', this.editing, 'navy');
    this.wireSingleTile('sp-cursor', this.editing, 'cursor');
    this.wireSingleTile('sp-fog', this.editing, 'fog');
  }

  private renderTileGroup(tiles: TileDef[], labels: string[], prefix: string): string {
    return tiles.map((t, i) => `
      <div style="display:flex;align-items:center;gap:8px;margin:3px 0;">
        <span style="width:100px;color:#888;font-size:11px;">${labels[i] ?? `#${i}`}</span>
        <span style="width:28px;text-align:center;font-size:16px;">${t.value}</span>
        <input id="${prefix}-${i}" value="${this.esc(t.value)}" style="background:#111;color:#55ff55;border:1px solid #222;padding:2px 6px;width:60px;font-family:inherit;font-size:14px;" />
        <input id="${prefix}-fg-${i}" value="${t.fg ?? ''}" placeholder="fg" style="background:#111;color:#aaa;border:1px solid #222;padding:2px 4px;width:65px;font-size:10px;font-family:inherit;" />
        <input id="${prefix}-bg-${i}" value="${t.bg ?? ''}" placeholder="bg" style="background:#111;color:#aaa;border:1px solid #222;padding:2px 4px;width:65px;font-size:10px;font-family:inherit;" />
      </div>
    `).join('');
  }

  private renderSingleTile(t: TileDef, label: string, id: string): string {
    return `
      <div style="display:flex;align-items:center;gap:8px;margin:3px 0;">
        <span style="width:100px;color:#888;font-size:11px;">${label}</span>
        <span style="width:28px;text-align:center;font-size:16px;">${t.value}</span>
        <input id="${id}" value="${this.esc(t.value)}" style="background:#111;color:#55ff55;border:1px solid #222;padding:2px 6px;width:60px;font-family:inherit;font-size:14px;" />
        <input id="${id}-fg" value="${t.fg ?? ''}" placeholder="fg" style="background:#111;color:#aaa;border:1px solid #222;padding:2px 4px;width:65px;font-size:10px;font-family:inherit;" />
        <input id="${id}-bg" value="${t.bg ?? ''}" placeholder="bg" style="background:#111;color:#aaa;border:1px solid #222;padding:2px 4px;width:65px;font-size:10px;font-family:inherit;" />
      </div>
    `;
  }

  private wireTileInputs(prefix: string, tiles: TileDef[], labels: string[]): void {
    tiles.forEach((_, i) => {
      const valEl = this.overlay.querySelector(`#${prefix}-${i}`) as HTMLInputElement;
      const fgEl = this.overlay.querySelector(`#${prefix}-fg-${i}`) as HTMLInputElement;
      const bgEl = this.overlay.querySelector(`#${prefix}-bg-${i}`) as HTMLInputElement;
      if (valEl) valEl.addEventListener('input', () => { tiles[i].value = valEl.value; });
      if (fgEl) fgEl.addEventListener('input', () => { tiles[i].fg = fgEl.value || undefined; });
      if (bgEl) bgEl.addEventListener('input', () => { tiles[i].bg = bgEl.value || undefined; });
    });
  }

  private wireSingleTile(id: string, obj: any, key: string): void {
    const valEl = this.overlay.querySelector(`#${id}`) as HTMLInputElement;
    const fgEl = this.overlay.querySelector(`#${id}-fg`) as HTMLInputElement;
    const bgEl = this.overlay.querySelector(`#${id}-bg`) as HTMLInputElement;
    if (valEl) valEl.addEventListener('input', () => { obj[key].value = valEl.value; });
    if (fgEl) fgEl.addEventListener('input', () => { obj[key].fg = fgEl.value || undefined; });
    if (bgEl) bgEl.addEventListener('input', () => { obj[key].bg = bgEl.value || undefined; });
  }

  private save(): void {
    // Update the tileset name from input
    this.editing.id = this.editing.id || `custom-${Date.now()}`;
    saveCustomTileset(this.editing);
    this.onSave(this.editing);
    this.close();
  }

  private close(): void {
    this.overlay.remove();
    this.onClose();
  }

  private esc(s: string): string {
    return s.replace(/"/g, '&quot;').replace(/</g, '&lt;');
  }
}
