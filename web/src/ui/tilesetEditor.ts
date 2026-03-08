// tilesetEditor.ts — Full tileset editor with image previews, all game states,
// style selector for generated tilesets, and organized sections.

import { TileSet, TileDef, ALL_TILESETS, getTileset, getCachedImage, preloadTilesetImages } from '../renderer/tilesets';

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

// ─── Label maps for every game entity ───

const ELEVATION_LABELS = ['Water', 'Peak', 'Mountain', 'Hill', 'Flat'];
const VEGETATION_LABELS = [
  'Volcano', 'Desert', 'Tundra', 'Barren', 'Lt Veg', 'Good',
  'Wood', 'Forest', 'Jungle', 'Swamp', 'Ice', 'None'
];
const DESIGNATION_LABELS = [
  'Town', 'City', 'Mine', 'Farm', 'Devastated', 'Gold Mine', 'Fort', 'Ruin',
  'Stockade', 'Capitol', 'Special', 'Lumber Yard', 'Blacksmith', 'Road',
  'Mill', 'Granary', 'Church', 'University', 'Undesignated', 'Base Camp'
];

const ARMY_STATUS_LABELS = [
  'March', 'Scout', 'Garrison', 'Traded', 'Militia', 'Flight',
  'Defend', 'Mag Def', 'Attack', 'Mag Att', 'General', 'Sortie',
  'Siege', 'Besieged', 'On Board', 'Rule'
];

const RACE_LABELS = ['Human', 'Orc', 'Elf', 'Dwarf', 'Lizard', 'Pirate', 'Savage', 'Nomad'];

const UNIT_TYPE_LABELS = [
  'Militia', 'Goblins', 'Orcs', 'Infantry', 'Sailors', 'Marines', 'Archers',
  'Uruk-Hai', 'Ninjas', 'Phalanx', 'Olog-Hai', 'Legionaries', 'Dragoons', 'Mercenaries',
  'Trolls', 'Elite', 'Lt Cavalry', 'Hv Cavalry', 'Catapults', 'Siege', 'Rocs',
  'Knights', 'Griffons', 'Elephants', 'Zombies', 'Spy', 'Scout'
];

const LEADER_LABELS = [
  'King', 'Baron', 'Emperor', 'Prince', 'Wizard', 'Mage',
  'Pope', 'Bishop', 'Admiral', 'Captain', 'Warlord', 'Lord',
  'Demon', 'Devil', 'Dragon', 'Wyrm', 'Shadow', 'Nazgul'
];

const MONSTER_LABELS = [
  'Spirit', 'Assassin', 'Efreet', 'Gargoyle', 'Wraith',
  'Hero', 'Centaur', 'Giant', 'Superhero', 'Mummy',
  'Elemental', 'Minotaur', 'Daemon', 'Balrog', 'Dragon'
];

const NAVY_SIZE_LABELS = ['Light', 'Medium', 'Heavy'];

const SEASON_LABELS = ['Winter', 'Spring', 'Summer', 'Fall'];

// Key statuses to show per-race combos for
const KEY_STATUS_LABELS = ['March', 'Defend', 'Attack', 'Siege', 'Flight'];
const KEY_STATUS_IDS = ['march', 'defend', 'attack', 'siege', 'flight'];

// ─── Extended TileSet interface ───
// We extend TileSet with optional fields for the new tile categories.
// The renderer can progressively adopt these.

export interface ExtendedTileSet extends TileSet {
  // Army status tiles (indexed by ArmyStatus value - 1, i.e. 0=March..15=Rule)
  armyStatuses?: TileDef[];
  // Race-specific default army tiles (indexed by race: 0=Human..7=Nomad)
  raceArmies?: TileDef[];
  // Race × Status combos: raceStatusTiles[raceIdx * 5 + statusIdx]
  // statusIdx: 0=march, 1=defend, 2=attack, 3=siege, 4=flight
  raceStatusTiles?: TileDef[];
  // Navy variants: [light, medium, heavy]
  navySizes?: TileDef[];
  // Race-specific navies (indexed by race)
  raceNavies?: TileDef[];
  // Individual unit type sprites (27 base units)
  unitTypes?: TileDef[];
  // Leader sprites (18 leaders)
  leaders?: TileDef[];
  // Monster sprites (15 monsters)
  monsters?: TileDef[];
  // Season variants: seasonTerrain[terrainIdx * 4 + seasonIdx]
  // terrainIdx: 0=flat,1=forest,2=good,3=hill,4=water
  seasonTerrain?: TileDef[];
}

// ─── Style detection (what generated tileset styles are available) ───

interface GeneratedStyle {
  id: string;
  name: string;
  path: string;
  tileCount: number;
  tiles: string[];  // filenames found
}

/** Detect which generated tileset styles exist in /tilesets/ */
async function detectGeneratedStyles(): Promise<GeneratedStyle[]> {
  const styles: GeneratedStyle[] = [];

  // Check known style directories
  const knownStyles = [
    { id: 'pixel32', name: '32px SNES Era', path: '/tilesets/pixel32/' },
    { id: 'pixel16', name: '16px Retro', path: '/tilesets/pixel16/' },
    { id: 'pixel64', name: '64px Modern Pixel', path: '/tilesets/pixel64/' },
    { id: 'painted32', name: '32px Painted Miniature', path: '/tilesets/painted32/' },
    { id: 'iso64', name: '64px Isometric', path: '/tilesets/iso64/' },
  ];

  for (const style of knownStyles) {
    // Try to fetch a known anchor tile to see if this style exists
    try {
      const resp = await fetch(style.path + 'elevation_water.png', { method: 'HEAD' });
      if (resp.ok) {
        // Style exists — enumerate tiles by checking known filenames
        const tileNames = await probeTilesInStyle(style.path);
        styles.push({
          ...style,
          tileCount: tileNames.length,
          tiles: tileNames,
        });
      }
    } catch { /* style not generated yet */ }
  }

  return styles;
}

/** Probe which tile files exist in a style directory */
async function probeTilesInStyle(basePath: string): Promise<string[]> {
  const found: string[] = [];
  const candidates = [
    // Elevation
    'elevation_water.png', 'elevation_peak.png', 'elevation_mountain.png',
    'elevation_hill.png', 'elevation_flat.png',
    // Vegetation
    'vegetation_volcano.png', 'vegetation_desert.png', 'vegetation_tundra.png',
    'vegetation_barren.png', 'vegetation_light_veg.png', 'vegetation_good.png',
    'vegetation_wood.png', 'vegetation_forest.png', 'vegetation_jungle.png',
    'vegetation_swamp.png', 'vegetation_ice.png',
    // Designation
    'designation_town.png', 'designation_city.png', 'designation_mine.png',
    'designation_farm.png', 'designation_devastated.png', 'designation_goldmine.png',
    'designation_fort.png', 'designation_ruin.png', 'designation_stockade.png',
    'designation_capitol.png', 'designation_special.png', 'designation_lumberyard.png',
    'designation_blacksmith.png', 'designation_road.png', 'designation_mill.png',
    'designation_granary.png', 'designation_church.png', 'designation_university.png',
    'designation_nodesig.png', 'designation_basecamp.png',
    // Units
    'units_army.png', 'units_navy.png',
    // Army statuses
    'status_march.png', 'status_scout.png', 'status_garrison.png', 'status_defend.png',
    'status_magdef.png', 'status_attack.png', 'status_magatt.png', 'status_flight.png',
    'status_siege.png', 'status_sieged.png', 'status_sortie.png', 'status_general.png',
    'status_onboard.png', 'status_group.png',
    // Race armies
    'race_human_army.png', 'race_orc_army.png', 'race_elf_army.png', 'race_dwarf_army.png',
    'race_lizard_army.png', 'race_pirate_army.png', 'race_savage_army.png', 'race_nomad_army.png',
    // Race × status combos
    ...RACE_LABELS.flatMap(r => KEY_STATUS_IDS.map(s =>
      `race_${r.toLowerCase()}_${s}.png`
    )),
    // Navy sizes
    'navy_light.png', 'navy_medium.png', 'navy_heavy.png',
    'navy_fleet.png', 'navy_transport.png', 'navy_combat.png',
    // Race navies
    ...RACE_LABELS.map(r => `navy_${r.toLowerCase()}.png`),
    // Unit types
    ...['militia', 'goblin', 'orc', 'infantry', 'sailor', 'marines', 'archer',
        'uruk', 'ninja', 'phalanx', 'olog', 'legion', 'dragoon', 'mercenary',
        'troll', 'elite', 'lt_cavalry', 'cavalry', 'catapult', 'siege_unit', 'roc',
        'knight', 'griffon', 'elephant', 'zombie', 'spy', 'scout_unit'].map(u => `unit_${u}.png`),
    // Leaders
    ...['king', 'baron', 'emperor', 'prince', 'wizard', 'mage', 'pope', 'bishop',
        'admiral', 'captain', 'warlord', 'lord', 'demon_lord', 'devil',
        'dragon_lord', 'wyrm', 'shadow', 'nazgul'].map(l => `leader_${l}.png`),
    // Monsters
    ...['spirit', 'assassin', 'djinni', 'gargoyle', 'wraith', 'hero', 'centaur',
        'giant', 'superhero', 'mummy', 'elemental', 'minotaur', 'daemon',
        'balrog', 'dragon'].map(m => `monster_${m}.png`),
    // Seasons
    ...['flat', 'forest', 'good', 'hill', 'water'].flatMap(t =>
      ['winter', 'spring', 'summer', 'fall'].map(s => `season_${t}_${s}.png`)
    ),
    // Special
    'special_fog.png', 'special_fog_edge.png', 'special_cursor.png',
    'special_explosion.png', 'special_magic_effect.png',
  ];

  // Batch HEAD requests (limited concurrency)
  const batchSize = 20;
  for (let i = 0; i < candidates.length; i += batchSize) {
    const batch = candidates.slice(i, i + batchSize);
    const results = await Promise.all(
      batch.map(async (name) => {
        try {
          const resp = await fetch(basePath + name, { method: 'HEAD' });
          return resp.ok ? name : null;
        } catch { return null; }
      })
    );
    found.push(...results.filter((r): r is string => r !== null));
  }

  return found;
}

// ─── CSS ───

const EDITOR_CSS = `
  .tse-root {
    position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 200;
    background: rgba(0,0,0,0.95); overflow-y: auto; padding: 20px;
    font-family: "Courier New", monospace; color: #aaa;
  }
  .tse-root * { box-sizing: border-box; }
  .tse-container { max-width: 900px; margin: 0 auto; }
  .tse-header {
    display: flex; justify-content: space-between; align-items: center;
    margin-bottom: 16px; border-bottom: 1px solid #333; padding-bottom: 12px;
  }
  .tse-header h2 { margin: 0; color: #55ff55; }
  .tse-tabs {
    display: flex; gap: 2px; margin-bottom: 16px; flex-wrap: wrap;
  }
  .tse-tab {
    padding: 6px 14px; background: #111; color: #666; border: 1px solid #333;
    cursor: pointer; font-family: inherit; font-size: 12px; transition: all 0.15s;
  }
  .tse-tab:hover { color: #aaa; border-color: #555; }
  .tse-tab.active { color: #55ff55; border-color: #55ff55; background: #0a1a0a; }
  .tse-tab .tab-count {
    font-size: 10px; color: #555; margin-left: 4px;
  }
  .tse-tab.active .tab-count { color: #338833; }
  .tse-panel { display: none; }
  .tse-panel.active { display: block; }

  .tse-section {
    margin-bottom: 20px;
  }
  .tse-section h3 {
    color: #55ff55; border-bottom: 1px solid #222; padding-bottom: 4px;
    margin: 0 0 8px 0; font-size: 13px;
  }
  .tse-section h4 {
    color: #448844; margin: 12px 0 4px 0; font-size: 12px;
  }

  .tse-tile-grid {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 6px;
  }
  .tse-tile-card {
    background: #0a0a0a; border: 1px solid #222; padding: 6px;
    display: flex; flex-direction: column; align-items: center; gap: 4px;
    transition: border-color 0.15s;
  }
  .tse-tile-card:hover { border-color: #444; }
  .tse-tile-card.missing { opacity: 0.4; border-style: dashed; }
  .tse-tile-card.has-image { border-color: #334433; }

  .tse-tile-preview {
    width: 64px; height: 64px; display: flex; align-items: center; justify-content: center;
    background: #000; border: 1px solid #1a1a1a; image-rendering: pixelated;
    font-size: 28px; overflow: hidden;
  }
  .tse-tile-preview img {
    width: 100%; height: 100%; object-fit: contain; image-rendering: pixelated;
  }
  .tse-tile-label {
    font-size: 10px; color: #888; text-align: center; white-space: nowrap;
    overflow: hidden; text-overflow: ellipsis; width: 100%;
  }
  .tse-tile-status {
    font-size: 9px; padding: 1px 6px; border-radius: 3px;
  }
  .tse-tile-status.present { color: #55ff55; background: #0a1a0a; }
  .tse-tile-status.missing { color: #aa5555; background: #1a0a0a; }

  .tse-tile-input {
    background: #111; color: #55ff55; border: 1px solid #222;
    padding: 2px 6px; width: 100%; font-family: inherit; font-size: 12px;
    text-align: center;
  }

  .tse-style-selector {
    display: flex; gap: 8px; margin-bottom: 16px; flex-wrap: wrap;
  }
  .tse-style-btn {
    padding: 8px 16px; background: #111; color: #888; border: 1px solid #333;
    cursor: pointer; font-family: inherit; font-size: 12px;
  }
  .tse-style-btn:hover { border-color: #555; color: #aaa; }
  .tse-style-btn.active { color: #55ff55; border-color: #55ff55; background: #0a1a0a; }
  .tse-style-btn .style-count {
    display: block; font-size: 10px; color: #555; margin-top: 2px;
  }
  .tse-style-btn.active .style-count { color: #338833; }

  .tse-meta-row {
    display: flex; gap: 16px; margin-bottom: 12px; align-items: center; flex-wrap: wrap;
  }
  .tse-meta-row label { color: #888; font-size: 12px; }
  .tse-meta-row input, .tse-meta-row select {
    background: #111; color: #55ff55; border: 1px solid #333;
    padding: 4px 8px; font-family: inherit;
  }

  .tse-btn {
    padding: 8px 20px; cursor: pointer; font-family: inherit; font-size: 13px;
    border: 1px solid; transition: all 0.15s;
  }
  .tse-btn-save { background: #030; color: #5f5; border-color: #5f5; }
  .tse-btn-save:hover { background: #050; }
  .tse-btn-cancel { background: #111; color: #888; border-color: #333; }
  .tse-btn-cancel:hover { background: #1a1a1a; }
  .tse-btn-close { background: #200; color: #f55; border-color: #f55; }
  .tse-btn-close:hover { background: #300; }

  .tse-summary {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 8px; margin-bottom: 16px;
  }
  .tse-summary-card {
    background: #0a0a0a; border: 1px solid #222; padding: 8px;
    font-size: 11px;
  }
  .tse-summary-card .sc-name { color: #55ff55; font-size: 12px; }
  .tse-summary-card .sc-stat { color: #888; }
  .tse-summary-card .sc-bar {
    height: 4px; background: #111; margin-top: 4px; border-radius: 2px; overflow: hidden;
  }
  .tse-summary-card .sc-bar-fill {
    height: 100%; background: #55ff55; border-radius: 2px; transition: width 0.3s;
  }

  .tse-race-grid {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(600px, 1fr)); gap: 12px;
  }
  .tse-race-block {
    background: #050505; border: 1px solid #1a1a1a; padding: 8px;
  }
  .tse-race-block h4 {
    color: #55ff55; margin: 0 0 6px 0; font-size: 12px;
    border-bottom: 1px solid #1a1a1a; padding-bottom: 4px;
  }
  .tse-race-row {
    display: flex; gap: 6px; flex-wrap: wrap;
  }
`;

// ─── Tab definitions ───

interface TabDef {
  id: string;
  label: string;
  icon: string;
}

const TABS: TabDef[] = [
  { id: 'overview', label: 'Overview', icon: '📊' },
  { id: 'terrain', label: 'Terrain', icon: '🌍' },
  { id: 'armies', label: 'Armies & Status', icon: '⚔️' },
  { id: 'races', label: 'Races', icon: '🧝' },
  { id: 'navy', label: 'Navy', icon: '⛵' },
  { id: 'units', label: 'Unit Types', icon: '🗡️' },
  { id: 'leaders', label: 'Leaders', icon: '👑' },
  { id: 'monsters', label: 'Monsters', icon: '🐉' },
  { id: 'seasons', label: 'Seasons', icon: '🍂' },
  { id: 'ui', label: 'UI & Effects', icon: '✨' },
];

// ─── Editor class ───

export class TilesetEditor {
  private overlay: HTMLDivElement;
  private editing: ExtendedTileSet;
  private onSave: (ts: TileSet) => void;
  private onClose: () => void;
  private activeTab = 'overview';
  private generatedStyles: GeneratedStyle[] = [];
  private selectedStyleId: string = '';
  private selectedStylePath: string = '';
  private styleFiles: Set<string> = new Set();

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
    this.editing = JSON.parse(JSON.stringify(base)) as ExtendedTileSet;

    // Initialize extended arrays if not present
    if (!this.editing.armyStatuses) this.editing.armyStatuses = this.makeEmptyTiles(16);
    if (!this.editing.raceArmies) this.editing.raceArmies = this.makeEmptyTiles(8);
    if (!this.editing.raceStatusTiles) this.editing.raceStatusTiles = this.makeEmptyTiles(40);
    if (!this.editing.navySizes) this.editing.navySizes = this.makeEmptyTiles(3);
    if (!this.editing.raceNavies) this.editing.raceNavies = this.makeEmptyTiles(8);
    if (!this.editing.unitTypes) this.editing.unitTypes = this.makeEmptyTiles(27);
    if (!this.editing.leaders) this.editing.leaders = this.makeEmptyTiles(18);
    if (!this.editing.monsters) this.editing.monsters = this.makeEmptyTiles(15);
    if (!this.editing.seasonTerrain) this.editing.seasonTerrain = this.makeEmptyTiles(20);

    this.overlay = document.createElement('div');
    this.overlay.className = 'tse-root';

    // Inject CSS
    const style = document.createElement('style');
    style.textContent = EDITOR_CSS;
    this.overlay.appendChild(style);

    parent.appendChild(this.overlay);

    // Detect available generated styles, then render
    detectGeneratedStyles().then(styles => {
      this.generatedStyles = styles;
      // Auto-select if the current tileset matches a generated style
      const match = styles.find(s => this.editing.id.includes(s.id));
      if (match) {
        this.selectedStyleId = match.id;
        this.selectedStylePath = match.path;
        this.styleFiles = new Set(match.tiles);
      } else if (styles.length > 0) {
        this.selectedStyleId = styles[0].id;
        this.selectedStylePath = styles[0].path;
        this.styleFiles = new Set(styles[0].tiles);
      }
      this.render();
    });
  }

  private makeEmptyTiles(count: number): TileDef[] {
    return Array.from({ length: count }, () => ({ type: 'char' as const, value: '?' }));
  }

  private render(): void {
    // Keep the <style> element
    const styleEl = this.overlay.querySelector('style');
    this.overlay.innerHTML = '';
    if (styleEl) this.overlay.appendChild(styleEl);

    const container = document.createElement('div');
    container.className = 'tse-container';
    this.overlay.appendChild(container);

    container.innerHTML = `
      ${this.renderHeader()}
      ${this.renderStyleSelector()}
      ${this.renderTabs()}
      <div class="tse-panels">
        ${this.renderOverviewPanel()}
        ${this.renderTerrainPanel()}
        ${this.renderArmiesPanel()}
        ${this.renderRacesPanel()}
        ${this.renderNavyPanel()}
        ${this.renderUnitsPanel()}
        ${this.renderLeadersPanel()}
        ${this.renderMonstersPanel()}
        ${this.renderSeasonsPanel()}
        ${this.renderUiPanel()}
      </div>
      <div style="margin-top:20px;display:flex;gap:12px;border-top:1px solid #333;padding-top:16px;">
        <button class="tse-btn tse-btn-save" id="tse-save">💾 Save Tileset</button>
        <button class="tse-btn tse-btn-cancel" id="tse-cancel">Cancel</button>
      </div>
    `;

    this.wireEvents(container);
  }

  private renderHeader(): string {
    return `
      <div class="tse-header">
        <div>
          <h2>⚙ Tileset Editor</h2>
          <div class="tse-meta-row" style="margin-top:8px;">
            <label>Name:</label>
            <input id="tse-name" value="${this.esc(this.editing.name)}" style="width:220px;" />
            <label>Type:</label>
            <select id="tse-type">
              <option value="char" ${this.editing.tileType === 'char' ? 'selected' : ''}>Character</option>
              <option value="emoji" ${this.editing.tileType === 'emoji' ? 'selected' : ''}>Emoji</option>
              <option value="image" ${this.editing.tileType === 'image' ? 'selected' : ''}>Image</option>
            </select>
            <label>Cell:</label>
            <input id="tse-cw" type="number" value="${this.editing.cellWidth}" min="8" max="128" style="width:50px;" />
            <span style="color:#555;">×</span>
            <input id="tse-ch" type="number" value="${this.editing.cellHeight}" min="8" max="128" style="width:50px;" />
          </div>
        </div>
        <button class="tse-btn tse-btn-close" id="tse-close">✕ Close</button>
      </div>
    `;
  }

  private renderStyleSelector(): string {
    if (this.generatedStyles.length === 0) {
      return `<div style="color:#555;font-size:11px;margin-bottom:12px;">No generated image tilesets found in /tilesets/. Generate tiles first.</div>`;
    }

    const btns = this.generatedStyles.map(s => `
      <button class="tse-style-btn ${s.id === this.selectedStyleId ? 'active' : ''}" data-style-id="${s.id}">
        🎨 ${s.name}
        <span class="style-count">${s.tileCount} tiles</span>
      </button>
    `).join('');

    return `
      <div class="tse-section">
        <h3>Generated Styles</h3>
        <div class="tse-style-selector">${btns}</div>
      </div>
    `;
  }

  private renderTabs(): string {
    const tabs = TABS.map(t => {
      const count = this.getTabTileCount(t.id);
      return `<button class="tse-tab ${t.id === this.activeTab ? 'active' : ''}" data-tab="${t.id}">
        ${t.icon} ${t.label}<span class="tab-count">(${count})</span>
      </button>`;
    }).join('');
    return `<div class="tse-tabs">${tabs}</div>`;
  }

  private getTabTileCount(tabId: string): string {
    const present = (arr: TileDef[] | undefined, filePrefix: string, names: string[]) => {
      if (!arr) return { has: 0, total: names.length };
      let has = 0;
      names.forEach((_, i) => {
        if (arr[i] && arr[i].type === 'image' && arr[i].value) has++;
        // Also check if file exists in selected style
      });
      return { has, total: names.length };
    };

    switch (tabId) {
      case 'overview': return '—';
      case 'terrain': return `${5 + 12 + 20}`;
      case 'armies': return `${16}`;
      case 'races': return `${8 + 40}`;
      case 'navy': return `${3 + 8}`;
      case 'units': return `${27}`;
      case 'leaders': return `${18}`;
      case 'monsters': return `${15}`;
      case 'seasons': return `${20}`;
      case 'ui': return `${19}`;
      default: return '?';
    }
  }

  // ─── Panel renderers ───

  private renderOverviewPanel(): string {
    const sections = [
      { name: 'Elevation', total: 5, files: ['elevation_water.png', 'elevation_peak.png', 'elevation_mountain.png', 'elevation_hill.png', 'elevation_flat.png'] },
      { name: 'Vegetation', total: 12, files: VEGETATION_LABELS.map((_, i) => `vegetation_${['volcano','desert','tundra','barren','light_veg','good','wood','forest','jungle','swamp','ice','none'][i]}.png`) },
      { name: 'Designation', total: 20, files: DESIGNATION_LABELS.map((_, i) => `designation_${['town','city','mine','farm','devastated','goldmine','fort','ruin','stockade','capitol','special','lumberyard','blacksmith','road','mill','granary','church','university','nodesig','basecamp'][i]}.png`) },
      { name: 'Army Status', total: 16, files: ARMY_STATUS_LABELS.map((_, i) => `status_${['march','scout','garrison','traded','militia','flight','defend','magdef','attack','magatt','general','sortie','siege','sieged','onboard','rule'][i]}.png`) },
      { name: 'Race Armies', total: 8, files: RACE_LABELS.map(r => `race_${r.toLowerCase()}_army.png`) },
      { name: 'Race × Status', total: 40, files: RACE_LABELS.flatMap(r => KEY_STATUS_IDS.map(s => `race_${r.toLowerCase()}_${s}.png`)) },
      { name: 'Navy', total: 11, files: ['navy_light.png','navy_medium.png','navy_heavy.png', ...RACE_LABELS.map(r => `navy_${r.toLowerCase()}.png`)] },
      { name: 'Unit Types', total: 27, files: ['militia','goblin','orc','infantry','sailor','marines','archer','uruk','ninja','phalanx','olog','legion','dragoon','mercenary','troll','elite','lt_cavalry','cavalry','catapult','siege_unit','roc','knight','griffon','elephant','zombie','spy','scout_unit'].map(u => `unit_${u}.png`) },
      { name: 'Leaders', total: 18, files: ['king','baron','emperor','prince','wizard','mage','pope','bishop','admiral','captain','warlord','lord','demon_lord','devil','dragon_lord','wyrm','shadow','nazgul'].map(l => `leader_${l}.png`) },
      { name: 'Monsters', total: 15, files: ['spirit','assassin','djinni','gargoyle','wraith','hero','centaur','giant','superhero','mummy','elemental','minotaur','daemon','balrog','dragon'].map(m => `monster_${m}.png`) },
      { name: 'Seasons', total: 20, files: ['flat','forest','good','hill','water'].flatMap(t => ['winter','spring','summer','fall'].map(s => `season_${t}_${s}.png`)) },
    ];

    const cards = sections.map(s => {
      const present = s.files.filter(f => this.styleFiles.has(f)).length;
      const pct = s.total > 0 ? Math.round((present / s.total) * 100) : 0;
      return `
        <div class="tse-summary-card">
          <div class="sc-name">${s.name}</div>
          <div class="sc-stat">${present}/${s.total} tiles (${pct}%)</div>
          <div class="sc-bar"><div class="sc-bar-fill" style="width:${pct}%"></div></div>
        </div>
      `;
    }).join('');

    const totalFiles = this.styleFiles.size;
    const totalPossible = sections.reduce((sum, s) => sum + s.total, 0);

    return `
      <div class="tse-panel ${this.activeTab === 'overview' ? 'active' : ''}" data-panel="overview">
        <div style="margin-bottom:12px;color:#888;font-size:12px;">
          Selected style: <strong style="color:#55ff55;">${this.selectedStyleId || 'none'}</strong>
          — ${totalFiles}/${totalPossible} tiles generated
        </div>
        <div class="tse-summary">${cards}</div>
      </div>
    `;
  }

  private renderTerrainPanel(): string {
    return `
      <div class="tse-panel ${this.activeTab === 'terrain' ? 'active' : ''}" data-panel="terrain">
        <div class="tse-section">
          <h3>Elevation (${ELEVATION_LABELS.length})</h3>
          <div class="tse-tile-grid">
            ${this.renderTileCards(this.editing.elevation, ELEVATION_LABELS, 'elevation',
              ['water', 'peak', 'mountain', 'hill', 'flat'])}
          </div>
        </div>
        <div class="tse-section">
          <h3>Vegetation (${VEGETATION_LABELS.length})</h3>
          <div class="tse-tile-grid">
            ${this.renderTileCards(this.editing.vegetation, VEGETATION_LABELS, 'vegetation',
              ['volcano', 'desert', 'tundra', 'barren', 'light_veg', 'good', 'wood', 'forest', 'jungle', 'swamp', 'ice', 'none'])}
          </div>
        </div>
        <div class="tse-section">
          <h3>Designation (${DESIGNATION_LABELS.length})</h3>
          <div class="tse-tile-grid">
            ${this.renderTileCards(this.editing.designation, DESIGNATION_LABELS, 'designation',
              ['town', 'city', 'mine', 'farm', 'devastated', 'goldmine', 'fort', 'ruin', 'stockade', 'capitol', 'special', 'lumberyard', 'blacksmith', 'road', 'mill', 'granary', 'church', 'university', 'nodesig', 'basecamp'])}
          </div>
        </div>
      </div>
    `;
  }

  private renderArmiesPanel(): string {
    const statusFileIds = ['march', 'scout', 'garrison', 'traded', 'militia', 'flight',
      'defend', 'magdef', 'attack', 'magatt', 'general', 'sortie', 'siege', 'sieged', 'onboard', 'rule'];
    return `
      <div class="tse-panel ${this.activeTab === 'armies' ? 'active' : ''}" data-panel="armies">
        <div class="tse-section">
          <h3>Generic Army (current)</h3>
          <div class="tse-tile-grid">
            ${this.renderSingleTileCard(this.editing.army, 'Army', 'units', 'army')}
            ${this.renderSingleTileCard(this.editing.navy, 'Navy', 'units', 'navy')}
          </div>
        </div>
        <div class="tse-section">
          <h3>Army Status States (${ARMY_STATUS_LABELS.length})</h3>
          <p style="font-size:11px;color:#555;margin:0 0 8px;">Each status changes how the army looks on the map.</p>
          <div class="tse-tile-grid">
            ${this.renderTileCards(this.editing.armyStatuses!, ARMY_STATUS_LABELS, 'status', statusFileIds)}
          </div>
        </div>
      </div>
    `;
  }

  private renderRacesPanel(): string {
    const raceIds = RACE_LABELS.map(r => r.toLowerCase());

    const raceBlocks = RACE_LABELS.map((race, ri) => {
      const raceId = raceIds[ri];
      const defaultTile = this.editing.raceArmies![ri];
      const defaultFile = `race_${raceId}_army.png`;
      const statusTiles = KEY_STATUS_LABELS.map((status, si) => {
        const tile = this.editing.raceStatusTiles![ri * 5 + si];
        const file = `race_${raceId}_${KEY_STATUS_IDS[si]}.png`;
        return this.renderTileCard(tile, status, file, `rs-${ri}-${si}`);
      }).join('');

      return `
        <div class="tse-race-block">
          <h4>${race}</h4>
          <div class="tse-race-row">
            ${this.renderTileCard(defaultTile, 'Default', defaultFile, `ra-${ri}`)}
            ${statusTiles}
          </div>
        </div>
      `;
    }).join('');

    return `
      <div class="tse-panel ${this.activeTab === 'races' ? 'active' : ''}" data-panel="races">
        <div class="tse-section">
          <h3>Race Armies — Default + Key Statuses</h3>
          <p style="font-size:11px;color:#555;margin:0 0 8px;">Each race gets a default army tile plus march/defend/attack/siege/flight variants.</p>
          ${raceBlocks}
        </div>
      </div>
    `;
  }

  private renderNavyPanel(): string {
    const raceIds = RACE_LABELS.map(r => r.toLowerCase());
    return `
      <div class="tse-panel ${this.activeTab === 'navy' ? 'active' : ''}" data-panel="navy">
        <div class="tse-section">
          <h3>Ship Sizes (${NAVY_SIZE_LABELS.length})</h3>
          <div class="tse-tile-grid">
            ${this.renderTileCards(this.editing.navySizes!, NAVY_SIZE_LABELS, 'navy',
              ['light', 'medium', 'heavy'])}
          </div>
        </div>
        <div class="tse-section">
          <h3>Race Navies (${RACE_LABELS.length})</h3>
          <div class="tse-tile-grid">
            ${this.renderTileCards(this.editing.raceNavies!, RACE_LABELS, 'navy',
              raceIds)}
          </div>
        </div>
      </div>
    `;
  }

  private renderUnitsPanel(): string {
    const unitIds = ['militia', 'goblin', 'orc', 'infantry', 'sailor', 'marines', 'archer',
      'uruk', 'ninja', 'phalanx', 'olog', 'legion', 'dragoon', 'mercenary',
      'troll', 'elite', 'lt_cavalry', 'cavalry', 'catapult', 'siege_unit', 'roc',
      'knight', 'griffon', 'elephant', 'zombie', 'spy', 'scout_unit'];
    return `
      <div class="tse-panel ${this.activeTab === 'units' ? 'active' : ''}" data-panel="units">
        <div class="tse-section">
          <h3>Base Unit Types (${UNIT_TYPE_LABELS.length})</h3>
          <div class="tse-tile-grid">
            ${this.renderTileCards(this.editing.unitTypes!, UNIT_TYPE_LABELS, 'unit', unitIds)}
          </div>
        </div>
      </div>
    `;
  }

  private renderLeadersPanel(): string {
    const leaderIds = ['king', 'baron', 'emperor', 'prince', 'wizard', 'mage', 'pope', 'bishop',
      'admiral', 'captain', 'warlord', 'lord', 'demon_lord', 'devil', 'dragon_lord', 'wyrm', 'shadow', 'nazgul'];
    return `
      <div class="tse-panel ${this.activeTab === 'leaders' ? 'active' : ''}" data-panel="leaders">
        <div class="tse-section">
          <h3>Leader Types (${LEADER_LABELS.length})</h3>
          <div class="tse-tile-grid">
            ${this.renderTileCards(this.editing.leaders!, LEADER_LABELS, 'leader', leaderIds)}
          </div>
        </div>
      </div>
    `;
  }

  private renderMonstersPanel(): string {
    const monsterIds = ['spirit', 'assassin', 'djinni', 'gargoyle', 'wraith', 'hero', 'centaur',
      'giant', 'superhero', 'mummy', 'elemental', 'minotaur', 'daemon', 'balrog', 'dragon'];
    return `
      <div class="tse-panel ${this.activeTab === 'monsters' ? 'active' : ''}" data-panel="monsters">
        <div class="tse-section">
          <h3>Monster Types (${MONSTER_LABELS.length})</h3>
          <div class="tse-tile-grid">
            ${this.renderTileCards(this.editing.monsters!, MONSTER_LABELS, 'monster', monsterIds)}
          </div>
        </div>
      </div>
    `;
  }

  private renderSeasonsPanel(): string {
    const terrainLabels = ['Flat', 'Forest', 'Good Land', 'Hills', 'Water'];
    const terrainIds = ['flat', 'forest', 'good', 'hill', 'water'];

    const blocks = terrainLabels.map((terrain, ti) => {
      const tiles = SEASON_LABELS.map((season, si) => {
        const tile = this.editing.seasonTerrain![ti * 4 + si];
        const file = `season_${terrainIds[ti]}_${season.toLowerCase()}.png`;
        return this.renderTileCard(tile, season, file, `sn-${ti}-${si}`);
      }).join('');

      return `
        <div class="tse-race-block">
          <h4>${terrain}</h4>
          <div class="tse-race-row">${tiles}</div>
        </div>
      `;
    }).join('');

    return `
      <div class="tse-panel ${this.activeTab === 'seasons' ? 'active' : ''}" data-panel="seasons">
        <div class="tse-section">
          <h3>Seasonal Terrain Variants</h3>
          <p style="font-size:11px;color:#555;margin:0 0 8px;">5 terrain types × 4 seasons. Other terrains can use runtime tinting.</p>
          ${blocks}
        </div>
      </div>
    `;
  }

  private renderUiPanel(): string {
    return `
      <div class="tse-panel ${this.activeTab === 'ui' ? 'active' : ''}" data-panel="ui">
        <div class="tse-section">
          <h3>Special Tiles</h3>
          <div class="tse-tile-grid">
            ${this.renderSingleTileCard(this.editing.cursor, 'Cursor', 'special', 'cursor')}
            ${this.renderSingleTileCard(this.editing.fog, 'Fog of War', 'special', 'fog')}
          </div>
        </div>
      </div>
    `;
  }

  // ─── Tile card rendering helpers ───

  private renderTileCards(tiles: TileDef[], labels: string[], prefix: string, fileIds: string[]): string {
    return tiles.map((t, i) => {
      const file = `${prefix}_${fileIds[i]}.png`;
      return this.renderTileCard(t, labels[i] ?? `#${i}`, file, `${prefix}-${i}`);
    }).join('');
  }

  private renderTileCard(tile: TileDef, label: string, expectedFile: string, inputId: string): string {
    const hasFile = this.styleFiles.has(expectedFile);
    const imgSrc = hasFile ? this.selectedStylePath + expectedFile : '';
    const cardClass = hasFile ? 'has-image' : 'missing';
    const statusClass = hasFile ? 'present' : 'missing';
    const statusText = hasFile ? '✓' : '—';

    let preview = '';
    if (hasFile && imgSrc) {
      preview = `<img src="${imgSrc}" alt="${label}" />`;
    } else if (tile.type === 'image' && tile.value) {
      preview = `<img src="${tile.value}" alt="${label}" />`;
    } else if (tile.value && tile.value !== '?') {
      preview = `<span style="font-size:${tile.type === 'emoji' ? '28' : '20'}px;color:${tile.fg || '#55ff55'};">${tile.value}</span>`;
    } else {
      preview = `<span style="color:#333;font-size:14px;">?</span>`;
    }

    return `
      <div class="tse-tile-card ${cardClass}">
        <div class="tse-tile-preview">${preview}</div>
        <div class="tse-tile-label">${label}</div>
        <span class="tse-tile-status ${statusClass}">${statusText} ${expectedFile.replace('.png', '')}</span>
      </div>
    `;
  }

  private renderSingleTileCard(tile: TileDef, label: string, prefix: string, fileId: string): string {
    return this.renderTileCard(tile, label, `${prefix}_${fileId}.png`, `${prefix}-${fileId}`);
  }

  // ─── Event wiring ───

  private wireEvents(container: HTMLElement): void {
    // Close / Cancel / Save
    container.querySelector('#tse-close')?.addEventListener('click', () => this.close());
    container.querySelector('#tse-cancel')?.addEventListener('click', () => this.close());
    container.querySelector('#tse-save')?.addEventListener('click', () => this.save());

    // Meta fields
    container.querySelector('#tse-name')?.addEventListener('input', (e) => {
      this.editing.name = (e.target as HTMLInputElement).value;
    });
    container.querySelector('#tse-type')?.addEventListener('change', (e) => {
      this.editing.tileType = (e.target as HTMLSelectElement).value as any;
    });
    container.querySelector('#tse-cw')?.addEventListener('change', (e) => {
      this.editing.cellWidth = parseInt((e.target as HTMLInputElement).value) || 14;
    });
    container.querySelector('#tse-ch')?.addEventListener('change', (e) => {
      this.editing.cellHeight = parseInt((e.target as HTMLInputElement).value) || 16;
    });

    // Tab switching
    container.querySelectorAll('.tse-tab').forEach(btn => {
      btn.addEventListener('click', () => {
        this.activeTab = (btn as HTMLElement).dataset.tab!;
        this.render();
      });
    });

    // Style selector
    container.querySelectorAll('.tse-style-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const id = (btn as HTMLElement).dataset.styleId!;
        const style = this.generatedStyles.find(s => s.id === id);
        if (style) {
          this.selectedStyleId = style.id;
          this.selectedStylePath = style.path;
          this.styleFiles = new Set(style.tiles);
          this.render();
        }
      });
    });
  }

  private save(): void {
    this.editing.id = this.editing.id || `custom-${Date.now()}`;

    // If a generated style is selected, wire image tiles from it
    if (this.selectedStylePath && this.editing.tileType === 'image') {
      this.wireImageTilesFromStyle();
    }

    saveCustomTileset(this.editing);
    this.onSave(this.editing);
    this.close();
  }

  /** Auto-populate image tile values from the selected generated style */
  private wireImageTilesFromStyle(): void {
    const p = this.selectedStylePath;

    // Helper to set tile if file exists
    const setIfExists = (tile: TileDef, file: string) => {
      if (this.styleFiles.has(file)) {
        tile.type = 'image';
        tile.value = p + file;
      }
    };

    // Elevation
    const elevIds = ['water', 'peak', 'mountain', 'hill', 'flat'];
    this.editing.elevation.forEach((t, i) => setIfExists(t, `elevation_${elevIds[i]}.png`));

    // Vegetation
    const vegIds = ['volcano', 'desert', 'tundra', 'barren', 'light_veg', 'good', 'wood', 'forest', 'jungle', 'swamp', 'ice', 'none'];
    this.editing.vegetation.forEach((t, i) => setIfExists(t, `vegetation_${vegIds[i]}.png`));

    // Designation
    const desIds = ['town', 'city', 'mine', 'farm', 'devastated', 'goldmine', 'fort', 'ruin', 'stockade', 'capitol', 'special', 'lumberyard', 'blacksmith', 'road', 'mill', 'granary', 'church', 'university', 'nodesig', 'basecamp'];
    this.editing.designation.forEach((t, i) => setIfExists(t, `designation_${desIds[i]}.png`));

    // Army / Navy
    setIfExists(this.editing.army, 'units_army.png');
    setIfExists(this.editing.navy, 'units_navy.png');

    // Army statuses
    const statusIds = ['march', 'scout', 'garrison', 'traded', 'militia', 'flight', 'defend', 'magdef', 'attack', 'magatt', 'general', 'sortie', 'siege', 'sieged', 'onboard', 'rule'];
    this.editing.armyStatuses?.forEach((t, i) => setIfExists(t, `status_${statusIds[i]}.png`));

    // Race armies
    const raceIds = RACE_LABELS.map(r => r.toLowerCase());
    this.editing.raceArmies?.forEach((t, i) => setIfExists(t, `race_${raceIds[i]}_army.png`));

    // Race × Status
    this.editing.raceStatusTiles?.forEach((t, i) => {
      const ri = Math.floor(i / 5);
      const si = i % 5;
      setIfExists(t, `race_${raceIds[ri]}_${KEY_STATUS_IDS[si]}.png`);
    });

    // Navy sizes
    const navySizeIds = ['light', 'medium', 'heavy'];
    this.editing.navySizes?.forEach((t, i) => setIfExists(t, `navy_${navySizeIds[i]}.png`));

    // Race navies
    this.editing.raceNavies?.forEach((t, i) => setIfExists(t, `navy_${raceIds[i]}.png`));

    // Unit types
    const unitIds = ['militia', 'goblin', 'orc', 'infantry', 'sailor', 'marines', 'archer', 'uruk', 'ninja', 'phalanx', 'olog', 'legion', 'dragoon', 'mercenary', 'troll', 'elite', 'lt_cavalry', 'cavalry', 'catapult', 'siege_unit', 'roc', 'knight', 'griffon', 'elephant', 'zombie', 'spy', 'scout_unit'];
    this.editing.unitTypes?.forEach((t, i) => setIfExists(t, `unit_${unitIds[i]}.png`));

    // Leaders
    const leaderIds = ['king', 'baron', 'emperor', 'prince', 'wizard', 'mage', 'pope', 'bishop', 'admiral', 'captain', 'warlord', 'lord', 'demon_lord', 'devil', 'dragon_lord', 'wyrm', 'shadow', 'nazgul'];
    this.editing.leaders?.forEach((t, i) => setIfExists(t, `leader_${leaderIds[i]}.png`));

    // Monsters
    const monsterIds = ['spirit', 'assassin', 'djinni', 'gargoyle', 'wraith', 'hero', 'centaur', 'giant', 'superhero', 'mummy', 'elemental', 'minotaur', 'daemon', 'balrog', 'dragon'];
    this.editing.monsters?.forEach((t, i) => setIfExists(t, `monster_${monsterIds[i]}.png`));

    // Seasons
    const terrainIds = ['flat', 'forest', 'good', 'hill', 'water'];
    const seasonIds = ['winter', 'spring', 'summer', 'fall'];
    this.editing.seasonTerrain?.forEach((t, i) => {
      const ti = Math.floor(i / 4);
      const si = i % 4;
      setIfExists(t, `season_${terrainIds[ti]}_${seasonIds[si]}.png`);
    });
  }

  private close(): void {
    this.overlay.remove();
    this.onClose();
  }

  private esc(s: string): string {
    return s.replace(/"/g, '&quot;').replace(/</g, '&lt;');
  }
}
