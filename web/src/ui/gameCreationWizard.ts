// gameCreationWizard.ts — Multi-step game creation wizard (T412-T418)

import { GameClient } from '../network/client';
import { GameSettings } from '../types';

const STORAGE_KEY_CUSTOM_PRESETS = 'conquer_custom_map_presets';

const DEFAULT_MAP_PRESETS: Record<string, { x: number; y: number; label: string }> = {
  small: { x: 24, y: 24, label: 'Small (24×24)' },
  medium: { x: 32, y: 32, label: 'Medium (32×32)' },
  large: { x: 48, y: 48, label: 'Large (48×48)' },
  huge: { x: 64, y: 64, label: 'Huge (64×64)' },
  massive: { x: 96, y: 96, label: 'Massive (96×96)' },
  epic: { x: 128, y: 128, label: 'Epic (128×128)' },
};

function loadCustomPresets(): Record<string, { x: number; y: number; label: string }> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY_CUSTOM_PRESETS);
    return raw ? JSON.parse(raw) : {};
  } catch { return {}; }
}

function saveCustomPreset(key: string, preset: { x: number; y: number; label: string }): void {
  const custom = loadCustomPresets();
  custom[key] = preset;
  localStorage.setItem(STORAGE_KEY_CUSTOM_PRESETS, JSON.stringify(custom));
}

function getMapPresets(): Record<string, { x: number; y: number; label: string }> {
  return { ...DEFAULT_MAP_PRESETS, ...loadCustomPresets() };
}

export class GameCreationWizard {
  private container: HTMLDivElement;
  private client: GameClient;
  private onComplete: () => void;
  private step = 1;
  private totalSteps = 3;

  // Wizard state
  private gameName = '';
  private mapSize = 'medium';
  private maxPlayers = 10;
  private npcCount = 10;
  private monsterCount = 5;
  private turnTimerHours: number | null = null;
  private password = '';
  private minPlayers = 2;
  private seed = Math.floor(Math.random() * 100000);
  private publicGame = true;
  private tradeEnabled = true;
  private randomEvents = true;

  constructor(parent: HTMLElement, client: GameClient, onComplete: () => void) {
    this.client = client;
    this.onComplete = onComplete;

    this.container = document.createElement('div');
    this.container.className = 'lobby';
    this.container.style.cssText = `
      font-family: "Courier New", "Consolas", monospace;
      background: #000; color: #aaa; min-height: 100vh; padding: 20px;
      display: flex; flex-direction: column; align-items: center;
    `;
    parent.appendChild(this.container);
    this.render();
  }

  private render(): void {
    switch (this.step) {
      case 1: this.renderStep1(); break;
      case 2: this.renderStep2(); break;
      case 3: this.renderStep3(); break;
    }
  }

  // Custom map size state
  private customX = 64;
  private customY = 64;

  private renderStep1(): void {
    const presets = getMapPresets();
    const isCustom = this.mapSize === 'custom';

    this.container.innerHTML = `
      <h1 style="color:#55ff55;text-shadow:0 0 10px #00aa00;">CREATE GAME</h1>
      <p style="color:#555;">Step 1/${this.totalSteps}: Basic Settings</p>

      <div style="max-width:500px;width:100%;margin:20px 0;">
        <div style="margin:10px 0;">
          <label style="color:#aaa;display:block;margin-bottom:4px;">Game Name:</label>
          <input id="wiz-name" type="text" value="${this.gameName}" placeholder="Enter game name"
            style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:6px 10px;width:100%;box-sizing:border-box;">
        </div>

        <div style="margin:10px 0;">
          <label style="color:#aaa;display:block;margin-bottom:4px;">Map Size:</label>
          ${Object.entries(presets).map(([key, val]) => `
            <label style="display:block;margin:4px 0;cursor:pointer;">
              <input type="radio" name="mapsize" value="${key}" ${this.mapSize === key ? 'checked' : ''} style="accent-color:#55ff55;">
              <span style="color:${this.mapSize === key ? '#55ff55' : '#aaa'};">${val.label}</span>
            </label>
          `).join('')}
          <label style="display:block;margin:4px 0;cursor:pointer;">
            <input type="radio" name="mapsize" value="custom" ${isCustom ? 'checked' : ''} style="accent-color:#55ff55;">
            <span style="color:${isCustom ? '#55ff55' : '#aaa'};">Custom...</span>
          </label>
          <div id="wiz-custom-size" style="margin:8px 0 0 24px;display:${isCustom ? 'block' : 'none'};">
            <div style="display:flex;gap:8px;align-items:center;">
              <label style="color:#aaa;">X:</label>
              <input id="wiz-custom-x" type="number" min="24" max="256" step="8" value="${this.customX}"
                style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:4px 8px;width:60px;">
              <label style="color:#aaa;">Y:</label>
              <input id="wiz-custom-y" type="number" min="24" max="256" step="8" value="${this.customY}"
                style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:4px 8px;width:60px;">
            </div>
            <p style="color:#555;font-size:12px;margin:4px 0;">Must be divisible by 8, minimum 24. Saved as a preset for future games.</p>
            <div id="wiz-custom-error" style="color:#ff5555;font-size:12px;"></div>
          </div>
        </div>

        <div style="margin:10px 0;">
          <label style="color:#aaa;">Max Players (2-35): </label>
          <input id="wiz-max-players" type="number" min="2" max="35" value="${this.maxPlayers}"
            style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:4px 8px;width:60px;">
        </div>

        <div style="margin:10px 0;">
          <label style="color:#aaa;">Min Players to Start: </label>
          <input id="wiz-min-players" type="number" min="1" max="35" value="${this.minPlayers}"
            style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:4px 8px;width:60px;">
        </div>
      </div>

      <div style="display:flex;gap:10px;">
        <button id="wiz-cancel" style="font-family:inherit;background:#222;color:#ff5555;border:1px solid #ff5555;padding:8px 16px;cursor:pointer;">Cancel</button>
        <button id="wiz-next" style="font-family:inherit;background:#222;color:#55ff55;border:1px solid #55ff55;padding:8px 16px;cursor:pointer;">Next →</button>
      </div>
    `;

    // Toggle custom size visibility
    document.querySelectorAll('input[name="mapsize"]').forEach(radio => {
      radio.addEventListener('change', () => {
        const customDiv = document.getElementById('wiz-custom-size')!;
        const val = (document.querySelector('input[name="mapsize"]:checked') as HTMLInputElement).value;
        customDiv.style.display = val === 'custom' ? 'block' : 'none';
      });
    });

    document.getElementById('wiz-cancel')!.addEventListener('click', this.onComplete);
    document.getElementById('wiz-next')!.addEventListener('click', () => {
      this.gameName = (document.getElementById('wiz-name') as HTMLInputElement).value;
      const selected = document.querySelector('input[name="mapsize"]:checked') as HTMLInputElement;
      if (selected) this.mapSize = selected.value;

      if (this.mapSize === 'custom') {
        const cx = parseInt((document.getElementById('wiz-custom-x') as HTMLInputElement).value) || 64;
        const cy = parseInt((document.getElementById('wiz-custom-y') as HTMLInputElement).value) || 64;
        const errorEl = document.getElementById('wiz-custom-error')!;

        if (cx < 24 || cy < 24) { errorEl.textContent = 'Minimum size is 24'; return; }
        if (cx % 8 !== 0 || cy % 8 !== 0) { errorEl.textContent = 'Must be divisible by 8'; return; }
        if (cx > 256 || cy > 256) { errorEl.textContent = 'Maximum size is 256'; return; }

        this.customX = cx;
        this.customY = cy;

        // Save as a named preset for future use
        const presetKey = `custom_${cx}x${cy}`;
        const presetLabel = cx === cy ? `Custom (${cx}×${cy})` : `Custom (${cx}×${cy})`;
        saveCustomPreset(presetKey, { x: cx, y: cy, label: presetLabel });
        this.mapSize = presetKey;
      }

      this.maxPlayers = parseInt((document.getElementById('wiz-max-players') as HTMLInputElement).value) || 10;
      this.minPlayers = parseInt((document.getElementById('wiz-min-players') as HTMLInputElement).value) || 2;
      this.step = 2;
      this.render();
    });
  }

  private renderStep2(): void {
    this.container.innerHTML = `
      <h1 style="color:#55ff55;text-shadow:0 0 10px #00aa00;">CREATE GAME</h1>
      <p style="color:#555;">Step 2/${this.totalSteps}: NPCs & Timer</p>

      <div style="max-width:500px;width:100%;margin:20px 0;">
        <div style="margin:10px 0;">
          <label style="color:#aaa;">NPC Nations: </label>
          <input id="wiz-npc" type="number" min="0" max="30" value="${this.npcCount}"
            style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:4px 8px;width:60px;">
        </div>

        <div style="margin:10px 0;">
          <label style="color:#aaa;">Monsters: </label>
          <input id="wiz-monsters" type="number" min="0" max="10" value="${this.monsterCount}"
            style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:4px 8px;width:60px;">
        </div>

        <div style="margin:10px 0;">
          <label style="color:#aaa;">Turn Timer (hours, 0=none): </label>
          <input id="wiz-timer" type="number" min="0" max="168" value="${this.turnTimerHours ?? 0}"
            style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:4px 8px;width:60px;">
        </div>

        <div style="margin:10px 0;">
          <label style="color:#aaa;">Seed: </label>
          <input id="wiz-seed" type="number" value="${this.seed}"
            style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:4px 8px;width:100px;">
          <button id="wiz-rand-seed" style="font-family:inherit;background:#222;color:#aaa;border:1px solid #333;padding:4px 8px;cursor:pointer;margin-left:4px;">🎲</button>
        </div>

        <div style="margin:10px 0;">
          <label style="color:#aaa;">Password (optional): </label>
          <input id="wiz-password" type="password" value="${this.password}" placeholder="Leave blank for no password"
            style="font-family:inherit;background:#111;color:#55ff55;border:1px solid #333;padding:4px 8px;width:200px;">
        </div>
      </div>

      <div style="display:flex;gap:10px;">
        <button id="wiz-back" style="font-family:inherit;background:#222;color:#aaa;border:1px solid #555;padding:8px 16px;cursor:pointer;">← Back</button>
        <button id="wiz-next" style="font-family:inherit;background:#222;color:#55ff55;border:1px solid #55ff55;padding:8px 16px;cursor:pointer;">Next →</button>
      </div>
    `;

    document.getElementById('wiz-back')!.addEventListener('click', () => { this.step = 1; this.render(); });
    document.getElementById('wiz-rand-seed')!.addEventListener('click', () => {
      (document.getElementById('wiz-seed') as HTMLInputElement).value = String(Math.floor(Math.random() * 100000));
    });
    document.getElementById('wiz-next')!.addEventListener('click', () => {
      this.npcCount = parseInt((document.getElementById('wiz-npc') as HTMLInputElement).value) || 0;
      this.monsterCount = parseInt((document.getElementById('wiz-monsters') as HTMLInputElement).value) || 0;
      const timer = parseInt((document.getElementById('wiz-timer') as HTMLInputElement).value);
      this.turnTimerHours = timer > 0 ? timer : null;
      this.seed = parseInt((document.getElementById('wiz-seed') as HTMLInputElement).value) || 42;
      this.password = (document.getElementById('wiz-password') as HTMLInputElement).value;
      this.step = 3;
      this.render();
    });
  }

  private renderStep3(): void {
    const presets = getMapPresets();
    const preset = presets[this.mapSize];
    this.container.innerHTML = `
      <h1 style="color:#55ff55;text-shadow:0 0 10px #00aa00;">CREATE GAME</h1>
      <p style="color:#555;">Step 3/${this.totalSteps}: Review & Create</p>

      <div style="max-width:500px;width:100%;margin:20px 0;border:1px solid #333;padding:16px;">
        <h2 style="color:#ffff55;margin-top:0;">Summary</h2>
        <table style="width:100%;">
          <tr><td style="color:#aaa;padding:3px 8px;">Name:</td><td style="color:#55ffff;">${this.gameName || '(unnamed)'}</td></tr>
          <tr><td style="color:#aaa;padding:3px 8px;">Map:</td><td>${preset.label}</td></tr>
          <tr><td style="color:#aaa;padding:3px 8px;">Players:</td><td>${this.minPlayers}-${this.maxPlayers}</td></tr>
          <tr><td style="color:#aaa;padding:3px 8px;">NPCs:</td><td>${this.npcCount}</td></tr>
          <tr><td style="color:#aaa;padding:3px 8px;">Monsters:</td><td>${this.monsterCount}</td></tr>
          <tr><td style="color:#aaa;padding:3px 8px;">Turn Timer:</td><td>${this.turnTimerHours ? this.turnTimerHours + 'h' : 'None'}</td></tr>
          <tr><td style="color:#aaa;padding:3px 8px;">Password:</td><td>${this.password ? '●●●●' : 'None'}</td></tr>
          <tr><td style="color:#aaa;padding:3px 8px;">Seed:</td><td>${this.seed}</td></tr>
        </table>

        <div style="margin:12px 0;">
          <label style="cursor:pointer;">
            <input type="checkbox" id="wiz-public" ${this.publicGame ? 'checked' : ''} style="accent-color:#55ff55;">
            <span>List in public game browser</span>
          </label>
        </div>
        <div style="margin:8px 0;">
          <label style="cursor:pointer;">
            <input type="checkbox" id="wiz-trade" ${this.tradeEnabled ? 'checked' : ''} style="accent-color:#55ff55;">
            <span>Trade enabled</span>
          </label>
        </div>
        <div style="margin:8px 0;">
          <label style="cursor:pointer;">
            <input type="checkbox" id="wiz-events" ${this.randomEvents ? 'checked' : ''} style="accent-color:#55ff55;">
            <span>Random events</span>
          </label>
        </div>
      </div>

      <div style="display:flex;gap:10px;">
        <button id="wiz-back" style="font-family:inherit;background:#222;color:#aaa;border:1px solid #555;padding:8px 16px;cursor:pointer;">← Back</button>
        <button id="wiz-create" style="font-family:inherit;background:#004400;color:#55ff55;border:1px solid #55ff55;padding:8px 20px;cursor:pointer;font-size:16px;">⚔ Create Game</button>
      </div>
      <div id="wiz-error" style="color:#ff5555;margin:8px 0;"></div>
    `;

    document.getElementById('wiz-back')!.addEventListener('click', () => { this.step = 2; this.render(); });
    document.getElementById('wiz-create')!.addEventListener('click', () => this.createGame());
  }

  private async createGame(): Promise<void> {
    const errorEl = document.getElementById('wiz-error')!;
    if (!this.gameName) { errorEl.textContent = 'Game name is required'; return; }

    this.publicGame = (document.getElementById('wiz-public') as HTMLInputElement).checked;
    this.tradeEnabled = (document.getElementById('wiz-trade') as HTMLInputElement).checked;
    this.randomEvents = (document.getElementById('wiz-events') as HTMLInputElement).checked;

    const presets = getMapPresets();
    const preset = presets[this.mapSize];
    const settings: Partial<GameSettings> = {
      map_x: preset.x,
      map_y: preset.y,
      max_players: this.maxPlayers,
      npc_count: this.npcCount,
      monster_count: this.monsterCount,
      seed: this.seed,
      turn_timer_hours: this.turnTimerHours,
      auto_advance: this.turnTimerHours !== null,
      min_players: this.minPlayers,
      password: this.password || undefined,
      public_game: this.publicGame,
      trade_enabled: this.tradeEnabled,
      random_events: this.randomEvents,
    };

    try {
      await this.client.createGame(this.gameName, settings);
      this.onComplete();
    } catch (e) {
      errorEl.textContent = `Failed: ${(e as Error).message}`;
    }
  }

  destroy(): void {
    this.container.remove();
  }
}
