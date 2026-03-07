// gameScreen.ts — Main game screen: ties renderer, map, panels, input, network
// Orchestrates the full game UI

import { TerminalRenderer } from '../renderer/terminal';
import { GameClient } from '../network/client';
import { GameState, createInitialState, buildOccupied } from '../state/gameState';
import { renderMap, renderTilesetCursor, screenSize } from './mapView';
import { renderBottomPanel } from '../ui/sidePanel';
import { ChatPanel } from '../ui/chatPanel';
import { GameLayout } from '../ui/gameLayout';
import { CommandSidebar } from '../ui/commandSidebar';
import { StatsSidebar } from '../ui/statsSidebar';
import { InputHandler, GameAction } from './inputHandler';
import { MouseHandler } from './mouseHandler';
import {
  ServerMessage, DisplayMode, HighlightMode,
  DESIGNATION_NAMES, ARMY_STATUS_NAMES, DIPLO_NAMES,
  ALTITUDE_NAMES, VEGETATION_NAMES,
} from '../types';
import { getSector } from '../state/gameState';
import { CURSES_COLORS } from '../renderer/colors';
import { getTheme, ALL_THEMES } from '../renderer/themes';
import { applyUiThemeCss } from '../ui/uiThemes';
import { TilesetEditor, loadCustomTilesets } from '../ui/tilesetEditor';
import { KeybindingsManager, KeybindingsModal } from '../ui/keybindingsModal';
import { showConfirm, showAlert, showInput, showSelect, showForm } from '../ui/modalDialog';
import { registerTileset, getTileset as getTilesetById, preloadTilesetImages, getScaledCellSize } from '../renderer/tilesets';
import { renderCompositedMap, layersForMode, DEFAULT_LAYERS, LayerConfig } from '../renderer/compositor';
import { MapTooltip } from '../ui/mapTooltip';

export class GameScreen {
  private layout: GameLayout;
  private canvas: HTMLCanvasElement;
  private term: TerminalRenderer;
  private client: GameClient;
  private input: InputHandler;
  private chatPanel: ChatPanel;
  private cmdSidebar: CommandSidebar;
  private statsSidebar: StatsSidebar;
  private mouseHandler: MouseHandler;
  private tooltip: MapTooltip;
  private keybindingsManager: KeybindingsManager;
  private state: GameState;
  private animFrame: number = 0;
  private statusMessage: string = '';
  private statusTimeout: number = 0;

  constructor(parent: HTMLElement, client: GameClient, gameId: string, nationId: number) {
    this.client = client;
    this.keybindingsManager = new KeybindingsManager();
    this.state = createInitialState();
    this.state.token = client.getToken();
    this.state.gameId = gameId;
    this.state.nationId = nationId;

    // Three-column layout with CRT bezel
    this.layout = new GameLayout(parent);
    const savedUiTheme = localStorage.getItem('conquer_ui_theme');
    if (savedUiTheme) this.layout.uiThemeId = savedUiTheme;
    this.layout.initMobileState();

    this.canvas = this.layout.canvas;
    this.canvas.style.background = '#000';

    // Initialize renderer
    this.term = new TerminalRenderer(this.canvas);
    this.handleResize();
    window.addEventListener('resize', () => this.handleResize());

    // Input handler
    this.input = new InputHandler((action) => this.handleAction(action));

    // Load user's custom tilesets
    for (const ts of loadCustomTilesets()) {
      registerTileset(ts);
    }

    // Preload images for saved image-based tileset
    const savedTsId = localStorage.getItem('conquer_tileset');
    if (savedTsId) {
      const savedTs = getTilesetById(savedTsId);
      if (savedTs.tileType === 'image') {
        preloadTilesetImages(savedTs);
      }
    }

    // Mouse/touch pan & zoom
    this.mouseHandler = new MouseHandler(this.canvas, this.term, {
      getOffset: () => ({ x: this.state.xOffset, y: this.state.yOffset }),
      setOffset: (x, y) => {
        // Free panning — no limits. Negative offsets let you center/move the map anywhere.
        this.state.xOffset = x;
        this.state.yOffset = y;
      },
      getFontSize: () => this.term.fontSize,
      setFontSize: (size) => {
        this.term.setFontSize(size);
        localStorage.setItem('conquer_font_size', String(size));
        this.handleResize();
      },
      getTilesetId: () => this.state.tilesetId ?? 'ascii',
      onTapCell: (mapX, mapY) => {
        // Move cursor to tapped map cell
        const { screenX, screenY } = screenSize(this.term);
        const localX = mapX - this.state.xOffset;
        const localY = mapY - this.state.yOffset;
        if (localX >= 0 && localX < screenX && localY >= 0 && localY < screenY) {
          this.state.cursorX = localX;
          this.state.cursorY = localY;
        }
      },
    });

    // Map tooltip — provide actual cell dimensions from renderer
    this.tooltip = new MapTooltip(this.canvas);
    this.tooltip.setCellSizeProvider(() => {
      const tsId = this.state.tilesetId ?? 'ascii';
      const ts = getTilesetById(tsId);
      if (ts.tileType !== 'char') {
        return getScaledCellSize(ts, this.term.fontSize);
      }
      // Classic char mode: 2 terminal cells per map sector
      return { cw: this.term.cellW * 2, ch: this.term.cellH };
    });

    // Left sidebar: commands
    this.cmdSidebar = new CommandSidebar(this.layout.leftBar, (cmd) => this.handleCommand(cmd));
    this.cmdSidebar.themeId = this.layout.uiThemeId;

    // Right sidebar: stats (match font size with left)
    this.statsSidebar = new StatsSidebar(this.layout.rightBar);
    this.statsSidebar.themeId = this.layout.uiThemeId;
    this.statsSidebar.fontSize = this.cmdSidebar.fontSize;

    // Chat panel (Phase 5: T400)
    this.chatPanel = new ChatPanel(parent, this.client, this.state);

    // WebSocket
    this.client.onMessage((msg) => this.handleWsMessage(msg));
    this.client.onConnect(() => {
      this.state.connected = true;
      this.setStatus('Connected');
    });
    this.client.onDisconnect(() => {
      this.state.connected = false;
      this.setStatus('Disconnected — reconnecting...');
    });

    // Restore saved theme preference
    const savedTheme = localStorage.getItem('conquer_theme');
    if (savedTheme) {
      this.state.themeId = savedTheme;
      this.state.renderMode = savedTheme.startsWith('classic') ? 'classic' : 'enhanced';
    }

    // Load initial data and start
    this.loadGameData().then(() => {
      this.client.connectWebSocket(gameId);
      this.startRenderLoop();
    });
  }

  private handleResize(): void {
    this.layout.onResize();
    const rect = this.canvas.getBoundingClientRect();
    this.term.resize(rect.width, rect.height);
  }

  private async loadGameData(): Promise<void> {
    const gameId = this.state.gameId!;
    try {
      const [gameInfo, nation, mapData, armies, navies, nations] = await Promise.all([
        this.client.getGame(gameId),
        this.client.getNation(gameId),
        this.client.getMap(gameId),
        this.client.getArmies(gameId),
        this.client.getNavies(gameId),
        this.client.getNations(gameId),
      ]);

      this.state.gameInfo = gameInfo;
      this.state.nation = nation;
      this.state.mapData = mapData;
      this.state.armies = armies;
      this.state.navies = navies;
      this.state.publicNations = nations;

      buildOccupied(this.state);

      // Center on capitol
      if (nation.cap_x > 0 || nation.cap_y > 0) {
        this.centerOn(nation.cap_x, nation.cap_y);
      }

      this.setStatus(`Welcome, ${nation.name}! Turn ${gameInfo.current_turn} — Press T for chat`);

      // Load scores for sidebar
      this.client.getScores(gameId).then(s => { this.state.scores = s; }).catch(() => {});

      // Load initial chat data (Phase 5)
      this.loadChatData();
    } catch (e) {
      this.setStatus(`Error loading game: ${(e as Error).message}`);
    }
  }

  private async loadChatData(): Promise<void> {
    if (!this.state.gameId) return;
    try {
      // Load presence
      const online = await this.client.getPresence(this.state.gameId);
      this.chatPanel.setPresence(online);

      // Load channels
      const channels = await this.client.getChatChannels(this.state.gameId);
      this.state.chatChannels = channels.length > 0 ? channels : ['public'];

      // Load public chat history
      const history = await this.client.getChatHistory(this.state.gameId, 'public');
      if (history.messages.length > 0) {
        this.chatPanel.onChatHistory('public', history.messages);
      }

      this.chatPanel.updateNations();
    } catch {
      // Chat data is non-critical
    }
  }

  private centerOn(x: number, y: number): void {
    const { screenX, screenY } = screenSize(this.term);
    this.state.xOffset = Math.max(0, x - Math.floor(screenX / 2));
    this.state.yOffset = Math.max(0, y - Math.floor(screenY / 2));
    this.state.cursorX = x - this.state.xOffset;
    this.state.cursorY = y - this.state.yOffset;
  }

  private handleAction(action: GameAction): void {
    const { screenX, screenY } = screenSize(this.term);

    switch (action.type) {
      case 'move_cursor': {
        let nx = this.state.cursorX + action.dx;
        let ny = this.state.cursorY + action.dy;

        // Scroll if cursor goes off screen
        if (nx < 0) { this.state.xOffset = Math.max(0, this.state.xOffset - 1); nx = 0; }
        if (ny < 0) { this.state.yOffset = Math.max(0, this.state.yOffset - 1); ny = 0; }
        if (nx >= screenX) {
          const maxX = (this.state.mapData?.map_x ?? 32) - screenX;
          if (this.state.xOffset < maxX) { this.state.xOffset++; nx = screenX - 1; }
          else nx = screenX - 1;
        }
        if (ny >= screenY) {
          const maxY = (this.state.mapData?.map_y ?? 32) - screenY;
          if (this.state.yOffset < maxY) { this.state.yOffset++; ny = screenY - 1; }
          else ny = screenY - 1;
        }

        this.state.cursorX = Math.max(0, Math.min(nx, screenX - 1));
        this.state.cursorY = Math.max(0, Math.min(ny, screenY - 1));
        break;
      }

      case 'move_or_cursor':
        if (this.state.movementMode) {
          // In movement mode: arrows move the selected army
          this.handleAction({ type: 'move_army', dx: action.dx, dy: action.dy });
        } else {
          // Normal mode: arrows move cursor
          this.handleAction({ type: 'move_cursor', dx: action.dx, dy: action.dy });
        }
        break;

      case 'toggle_movement_mode': {
        // Select army at cursor position, enter movement mode
        const curMapX = this.state.cursorX + this.state.xOffset;
        const curMapY = this.state.cursorY + this.state.yOffset;
        const armyAtCursor = this.state.armies.findIndex(
          a => a.soldiers > 0 && a.x === curMapX && a.y === curMapY
        );
        if (armyAtCursor >= 0) {
          this.state.selectedArmy = armyAtCursor;
          this.state.movementMode = true;
          const army = this.state.armies[armyAtCursor];
          this.setStatus(`🚩 MOVEMENT MODE — Army ${army.index}: ${army.soldiers} soldiers, ${army.movement} moves left. Arrows=move, Space=done`);
        } else {
          this.setStatus('No army at cursor position. Move cursor to an army and press M.');
        }
        break;
      }

      case 'exit_movement_mode':
        if (this.state.movementMode) {
          this.state.movementMode = false;
          this.setStatus('Movement done.');
        }
        break;

      case 'center_map':
        this.centerOn(
          this.state.cursorX + this.state.xOffset,
          this.state.cursorY + this.state.yOffset,
        );
        break;

      case 'jump_capitol':
        if (this.state.nation) {
          this.centerOn(this.state.nation.cap_x, this.state.nation.cap_y);
          this.setStatus('Jumped to capitol');
        }
        break;

      case 'set_display':
        this.state.displayMode = action.mode;
        this.setStatus(`Display: ${DisplayMode[action.mode]}`);
        break;

      case 'set_highlight':
        this.state.highlightMode = action.mode;
        this.setStatus(`Highlight: ${HighlightMode[action.mode]}`);
        break;

      case 'select_next_army': {
        const active = this.state.armies.filter(a => a.soldiers > 0);
        if (active.length > 0) {
          this.state.selectedArmy = (this.state.selectedArmy + 1) % active.length;
          const army = active[this.state.selectedArmy];
          this.centerOn(army.x, army.y);
          this.state.movementMode = true;
          this.setStatus(`🚩 Army ${army.index}: ${army.soldiers} soldiers, ${army.movement} moves. Arrows=move, Space=done`);
        } else {
          this.setStatus('No armies available.');
        }
        break;
      }

      case 'select_prev_army': {
        const active = this.state.armies.filter(a => a.soldiers > 0);
        if (active.length > 0) {
          this.state.selectedArmy = (this.state.selectedArmy - 1 + active.length) % active.length;
          const army = active[this.state.selectedArmy];
          this.centerOn(army.x, army.y);
          this.state.movementMode = true;
          this.setStatus(`🚩 Army ${army.index}: ${army.soldiers} soldiers, ${army.movement} moves. Arrows=move, Space=done`);
        }
        break;
      }

      case 'toggle_army_navy':
        this.state.armyOrNavy = this.state.armyOrNavy === 'army' ? 'navy' : 'army';
        this.setStatus(`Showing: ${this.state.armyOrNavy === 'army' ? 'Armies' : 'Navies'}`);
        break;

      case 'move_army': {
        const active = this.state.armies.filter(a => a.soldiers > 0);
        if (this.state.selectedArmy >= 0 && this.state.selectedArmy < active.length) {
          const army = active[this.state.selectedArmy];
          const nx = army.x + action.dx;
          const ny = army.y + action.dy;
          // Submit move action to server
          this.submitAction({
            MoveArmy: { nation: this.state.nationId, army: army.index, x: nx, y: ny }
          });
          // Optimistically update local position and follow with cursor
          army.x = nx;
          army.y = ny;
          this.centerOn(nx, ny);
          this.setStatus(`🚩 Army ${army.index} → (${nx},${ny}). Arrows=move, Space=done`);
        } else {
          this.setStatus('No army selected. Press Tab to select, or move cursor to army and press M.');
        }
        break;
      }

      case 'end_turn':
        this.doEndTurn();
        break;

      case 'show_scores':
        this.showScores();
        break;

      case 'show_news':
        this.showNews();
        break;

      case 'show_budget':
        this.showBudget();
        break;

      case 'show_help':
        this.showHelp();
        break;

      case 'redesignate':
        this.doRedesignate();
        break;

      case 'draft':
        this.doDraft();
        break;

      case 'diplomacy':
        this.doDiplomacy();
        break;

      case 'magic':
      case 'cast_spell':
        this.doCastSpell();
        break;

      case 'buy_power':
        this.doBuyPower();
        break;

      // Army status commands (T1)
      case 'set_army_attack':
        this.setArmyStatus(9, 'ATTACK');
        break;
      case 'set_army_defend':
        this.setArmyStatus(7, 'DEFEND');
        break;
      case 'set_army_garrison':
        this.setArmyStatus(3, 'GARRISON');
        break;
      case 'set_army_scout':
        this.setArmyStatus(2, 'SCOUT');
        break;
      case 'set_army_rule':
        this.setArmyStatus(16, 'RULE');
        break;
      case 'set_army_march':
        this.setArmyStatus(1, 'MARCH');
        break;

      // Army split/combine/divide (T2-T4)
      case 'split_army':
        this.doSplitArmy();
        break;
      case 'combine_army':
        this.doCombineArmy();
        break;
      case 'divide_army':
        this.doDivideArmy();
        break;

      // Building (T7-T9)
      case 'build_fort':
        this.doBuildFort();
        break;
      case 'build_road':
        this.doBuildRoad();
        break;
      case 'build_ship':
        this.doBuildShip();
        break;

      // Navy (T10)
      case 'load_fleet':
        this.doLoadFleet();
        break;
      case 'unload_fleet':
        this.doUnloadFleet();
        break;

      // Trade (T14-T15)
      case 'propose_trade':
        this.doProposeTrade();
        break;

      // Misc commands (T16-T18)
      case 'hire_mercs':
        this.doHireMercs();
        break;
      case 'bribe':
        this.doBribe();
        break;
      case 'send_tribute':
        this.doSendTribute();
        break;

      case 'toggle_chat':
        this.chatPanel.toggle();
        break;

      case 'font_increase':
        this.zoomCentered(1);
        break;

      case 'font_decrease':
        this.zoomCentered(-1);
        break;
    }
  }

  /** Get the current pixel size of one map cell */
  private getCurrentCellPixelSize(): { w: number; h: number } {
    const tsId = this.state.tilesetId ?? 'ascii';
    const ts = getTilesetById(tsId);
    if (ts.tileType !== 'char') {
      const s = getScaledCellSize(ts, this.term.fontSize);
      return { w: s.cw, h: s.ch };
    }
    // Classic char mode: 2 terminal cells wide per map sector
    return { w: this.term.cellW * 2, h: this.term.cellH };
  }

  /** Adjust font size so the new tileset's cell pixel size matches a target */
  private matchCellSize(newTs: { tileType: string; cellWidth: number; cellHeight: number }, targetPx: { w: number; h: number }): void {
    if (newTs.tileType === 'char') {
      // For char tilesets, cell size = font-derived cellW*2 × cellH
      // cellH ≈ fontSize * 1.2, so fontSize ≈ targetH / 1.2
      // cellW ≈ fontSize * 0.6, so 2*cellW ≈ fontSize * 1.2 ≈ targetW
      // Use height as the primary match (more reliable)
      const newFontSize = Math.max(6, Math.min(32, Math.round(targetPx.h / 1.2)));
      this.term.setFontSize(newFontSize);
      localStorage.setItem('conquer_font_size', String(newFontSize));
      this.handleResize();
    } else {
      // For emoji/image tilesets: cellPx = baseCellSize * (fontSize / 14)
      // So fontSize = 14 * targetPx / baseCellSize
      const scaleW = targetPx.w / newTs.cellWidth;
      const scaleH = targetPx.h / newTs.cellHeight;
      const scale = (scaleW + scaleH) / 2; // average both dimensions
      const newFontSize = Math.max(6, Math.min(32, Math.round(14 * scale)));
      this.term.setFontSize(newFontSize);
      localStorage.setItem('conquer_font_size', String(newFontSize));
      this.handleResize();
    }
  }

  /** Zoom (change font size) while keeping the view centered */
  private zoomCentered(delta: number, anchorScreenX?: number, anchorScreenY?: number): void {
    const oldSize = screenSize(this.term);
    const newFontSize = Math.max(6, this.term.fontSize + delta);
    if (newFontSize === this.term.fontSize) return;

    // Anchor point in map coordinates (default: center of viewport)
    const ax = anchorScreenX ?? Math.floor(oldSize.screenX / 2);
    const ay = anchorScreenY ?? Math.floor(oldSize.screenY / 2);
    const anchorMapX = this.state.xOffset + ax;
    const anchorMapY = this.state.yOffset + ay;

    // Apply new font size
    this.term.setFontSize(newFontSize);
    localStorage.setItem('conquer_font_size', String(newFontSize));
    this.handleResize();

    // Recalculate: keep anchor point at same screen position
    const newSize = screenSize(this.term);
    const newAx = anchorScreenX ?? Math.floor(newSize.screenX / 2);
    const newAy = anchorScreenY ?? Math.floor(newSize.screenY / 2);
    this.state.xOffset = anchorMapX - newAx;
    this.state.yOffset = anchorMapY - newAy;
  }

  private async submitAction(action: unknown): Promise<void> {
    if (!this.state.gameId) return;
    try {
      await this.client.submitActions(this.state.gameId, [action]);
    } catch (e) {
      this.setStatus(`Action failed: ${(e as Error).message}`);
    }
  }

  private async doEndTurn(): Promise<void> {
    if (!this.state.gameId) return;
    // Exit movement mode first
    this.state.movementMode = false;
    // Confirm
    const yes = await showConfirm('End your turn? All army movements will be submitted.', {
      title: '⚔ End Turn', confirmText: 'End Turn', cancelText: 'Keep Playing',
    });
    if (!yes) return;
    try {
      await this.client.endTurn(this.state.gameId);
      this.state.isDone = true;
      this.setStatus('Turn ended — waiting for other players...');
    } catch (e) {
      this.setStatus(`End turn failed: ${(e as Error).message}`);
    }
  }

  private async showScores(): Promise<void> {
    if (!this.state.gameId) return;
    try {
      const scores = await this.client.getScores(this.state.gameId);
      this.state.scores = scores;
      const scoreText = scores.map(s => `${s.name}: ${s.score}`).join(' | ');
      this.setStatus(`Scores: ${scoreText}`);
    } catch (e) {
      this.setStatus(`Scores failed: ${(e as Error).message}`);
    }
  }

  private async showNews(): Promise<void> {
    if (!this.state.gameId) return;
    try {
      const news = await this.client.getNews(this.state.gameId);
      this.state.news = news;
      if (news.length === 0) {
        this.setStatus('No news this turn');
      } else {
        this.setStatus(`News: ${news[0].content}`);
      }
    } catch (e) {
      this.setStatus(`News failed: ${(e as Error).message}`);
    }
  }

  private async showHelp(): Promise<void> {
    this.input.enabled = false;
    await showAlert(
      'MOVEMENT: Arrow/hjkl=move  Tab=next army  m=move mode  Space=done\n' +
      'DISPLAY: d=designation v=vegetation c=contour f=food r=race n=nation\n' +
      '  M=move D=defense p=people J=gold i=items\n' +
      'HIGHLIGHT: o=own a=armies y=yours L=range s=trade x=none\n' +
      'ARMY: A=attack G=garrison -=split /=divide\n' +
      'SECTOR: R=redesignate P=draft F=fort W=road\n' +
      'GAME: E=end turn S=scores N=news B=budget X=diplomacy\n' +
      'MAGIC: Z=cast spell Q=buy power\n' +
      'TRADE: $=propose trade  NAVY: `=toggle\n' +
      'VIEW: +/-=font C=center g=capitol T=chat ?=help',
      '❓ Command Reference'
    );
    this.input.enabled = true;
  }

  // ============ T1: Army Status ============

  private setArmyStatus(status: number, name: string): void {
    const active = this.state.armies.filter(a => a.soldiers > 0);
    if (this.state.selectedArmy < 0 || this.state.selectedArmy >= active.length) {
      this.setStatus('No army selected. Press Tab to select an army first.');
      return;
    }
    const army = active[this.state.selectedArmy];
    this.submitAction({
      AdjustArmyStat: { nation: this.state.nationId, army: army.index, status }
    });
    army.status = status;
    this.setStatus(`Army ${army.index} set to ${name}`);
  }

  // ============ T2: Split Army ============

  private async doSplitArmy(): Promise<void> {
    const active = this.state.armies.filter(a => a.soldiers > 0);
    if (this.state.selectedArmy < 0 || this.state.selectedArmy >= active.length) {
      this.setStatus('No army selected. Press Tab to select an army first.');
      return;
    }
    const army = active[this.state.selectedArmy];
    if (army.soldiers < 2) {
      this.setStatus('Army too small to split.');
      return;
    }
    this.input.enabled = false;
    const half = Math.floor(army.soldiers / 2);
    const result = await showInput(
      `Army ${army.index} has ${army.soldiers} soldiers. Split how many?`,
      { title: '✂ Split Army', defaultValue: String(half), inputType: 'number', confirmText: 'Split' }
    );
    this.input.enabled = true;
    if (result === null) return;
    const count = parseInt(result);
    if (isNaN(count) || count < 1 || count >= army.soldiers) {
      this.setStatus('Invalid split count.');
      return;
    }
    this.submitAction({
      SplitArmy: { nation: this.state.nationId, army: army.index, soldiers: count }
    });
    this.setStatus(`Split ${count} soldiers from Army ${army.index}`);
  }

  // ============ T3: Combine Army ============

  private async doCombineArmy(): Promise<void> {
    const active = this.state.armies.filter(a => a.soldiers > 0);
    if (this.state.selectedArmy < 0 || this.state.selectedArmy >= active.length) {
      this.setStatus('No army selected. Press Tab to select an army first.');
      return;
    }
    const army = active[this.state.selectedArmy];
    const others = active.filter(a => a.index !== army.index && a.x === army.x && a.y === army.y);
    if (others.length === 0) {
      this.setStatus('No other armies at this location to combine with.');
      return;
    }
    let targetArmy: typeof others[0];
    if (others.length === 1) {
      targetArmy = others[0];
    } else {
      this.input.enabled = false;
      const idx = await showSelect(
        `Combine Army ${army.index} (${army.soldiers} soldiers) with:`,
        others.map(a => ({
          label: `Army ${a.index}: ${a.soldiers} soldiers (${ARMY_STATUS_NAMES[a.status] ?? '?'})`,
        })),
        { title: '⊕ Combine Armies' }
      );
      this.input.enabled = true;
      if (idx < 0) return;
      targetArmy = others[idx];
    }
    this.submitAction({
      CombineArmies: { nation: this.state.nationId, army1: army.index, army2: targetArmy.index }
    });
    this.setStatus(`Combining Army ${army.index} with Army ${targetArmy.index}`);
  }

  // ============ T4: Divide Army ============

  private doDivideArmy(): void {
    const active = this.state.armies.filter(a => a.soldiers > 0);
    if (this.state.selectedArmy < 0 || this.state.selectedArmy >= active.length) {
      this.setStatus('No army selected. Press Tab to select an army first.');
      return;
    }
    const army = active[this.state.selectedArmy];
    if (army.soldiers < 2) {
      this.setStatus('Army too small to divide.');
      return;
    }
    this.submitAction({
      DivideArmy: { nation: this.state.nationId, army: army.index }
    });
    this.setStatus(`Dividing Army ${army.index} in half`);
  }

  // ============ T5: Redesignate Sector ============

  private async doRedesignate(): Promise<void> {
    const absX = this.state.cursorX + this.state.xOffset;
    const absY = this.state.cursorY + this.state.yOffset;
    const sector = getSector(this.state, absX, absY);
    if (!sector) { this.setStatus('Cannot see this sector.'); return; }
    if (sector.owner !== this.state.nationId) { this.setStatus('You do not own this sector.'); return; }

    const designations = [
      { label: 'Town (t)', value: 't' },
      { label: 'City (c)', value: 'c' },
      { label: 'Mine (m)', value: 'm' },
      { label: 'Farm (f)', value: 'f' },
      { label: 'Gold Mine ($)', value: '$' },
      { label: 'Fort (!)', value: '!' },
      { label: 'Stockade (s)', value: 's' },
      { label: 'Capitol (C)', value: 'C' },
      { label: 'Lumberyard (l)', value: 'l' },
      { label: 'Blacksmith (b)', value: 'b' },
      { label: 'Road (+)', value: '+' },
      { label: 'Mill (*)', value: '*' },
      { label: 'Granary (g)', value: 'g' },
      { label: 'Church (=)', value: '=' },
      { label: 'University (u)', value: 'u' },
    ];

    this.input.enabled = false;
    const idx = await showSelect(
      `Redesignate sector at (${absX},${absY}) — currently ${DESIGNATION_NAMES[sector.designation] ?? '?'}`,
      designations.map(d => ({ label: d.label })),
      { title: '🏗 Redesignate Sector' }
    );
    this.input.enabled = true;
    if (idx < 0) return;

    this.submitAction({
      DesignateSector: { nation: this.state.nationId, x: absX, y: absY, designation: designations[idx].value }
    });
    this.setStatus(`Redesignated (${absX},${absY}) to ${designations[idx].label}`);
  }

  // ============ T6: Draft Troops ============

  private async doDraft(): Promise<void> {
    const absX = this.state.cursorX + this.state.xOffset;
    const absY = this.state.cursorY + this.state.yOffset;
    const sector = getSector(this.state, absX, absY);
    if (!sector) { this.setStatus('Cannot see this sector.'); return; }
    if (sector.owner !== this.state.nationId) { this.setStatus('You do not own this sector.'); return; }
    if (sector.people < 1) { this.setStatus('No population to draft from.'); return; }

    const unitTypes = [
      { label: 'Infantry (type 0)', value: '0' },
      { label: 'Cavalry (type 1)', value: '1' },
      { label: 'Archers (type 2)', value: '2' },
    ];

    this.input.enabled = false;
    const result = await showForm(
      [
        { id: 'unit_type', label: 'Unit Type', type: 'select', options: unitTypes, defaultValue: '0' },
        { id: 'count', label: `How many? (pop: ${sector.people})`, type: 'number', defaultValue: String(Math.min(100, Math.floor(sector.people / 2))) },
      ],
      { title: `👥 Draft Troops at (${absX},${absY})`, confirmText: 'Draft' }
    );
    this.input.enabled = true;
    if (!result) return;

    const count = parseInt(result.count);
    if (isNaN(count) || count < 1) { this.setStatus('Invalid count.'); return; }
    const unitType = parseInt(result.unit_type);

    this.submitAction({
      DraftUnit: { nation: this.state.nationId, x: absX, y: absY, unit_type: unitType, count }
    });
    this.setStatus(`Drafted ${count} soldiers at (${absX},${absY})`);
  }

  // ============ T7: Build Fort ============

  private async doBuildFort(): Promise<void> {
    const absX = this.state.cursorX + this.state.xOffset;
    const absY = this.state.cursorY + this.state.yOffset;
    const sector = getSector(this.state, absX, absY);
    if (!sector) { this.setStatus('Cannot see this sector.'); return; }
    if (sector.owner !== this.state.nationId) { this.setStatus('You do not own this sector.'); return; }

    this.input.enabled = false;
    const yes = await showConfirm(
      `Build fortification at (${absX},${absY})?\nCurrent fort level: ${sector.fortress}`,
      { title: '🏰 Build Fortification', confirmText: 'Build' }
    );
    this.input.enabled = true;
    if (!yes) return;

    this.submitAction({
      ConstructFort: { nation: this.state.nationId, x: absX, y: absY }
    });
    this.setStatus(`Building fortification at (${absX},${absY})`);
  }

  // ============ T8: Build Road ============

  private async doBuildRoad(): Promise<void> {
    const absX = this.state.cursorX + this.state.xOffset;
    const absY = this.state.cursorY + this.state.yOffset;
    const sector = getSector(this.state, absX, absY);
    if (!sector) { this.setStatus('Cannot see this sector.'); return; }
    if (sector.owner !== this.state.nationId) { this.setStatus('You do not own this sector.'); return; }

    this.input.enabled = false;
    const yes = await showConfirm(
      `Build road at (${absX},${absY})?`,
      { title: '🛤 Build Road', confirmText: 'Build' }
    );
    this.input.enabled = true;
    if (!yes) return;

    this.submitAction({
      BuildRoad: { nation: this.state.nationId, x: absX, y: absY }
    });
    this.setStatus(`Building road at (${absX},${absY})`);
  }

  // ============ T9: Build Ship ============

  private async doBuildShip(): Promise<void> {
    const absX = this.state.cursorX + this.state.xOffset;
    const absY = this.state.cursorY + this.state.yOffset;
    const sector = getSector(this.state, absX, absY);
    if (!sector) { this.setStatus('Cannot see this sector.'); return; }
    if (sector.owner !== this.state.nationId) { this.setStatus('You do not own this sector.'); return; }

    const shipTypes = [
      { label: 'Warship', value: '0' },
      { label: 'Merchant', value: '1' },
      { label: 'Galley', value: '2' },
    ];
    const shipSizes = [
      { label: 'Small', value: '0' },
      { label: 'Medium', value: '1' },
      { label: 'Large', value: '2' },
    ];

    this.input.enabled = false;
    const result = await showForm(
      [
        { id: 'ship_type', label: 'Ship Type', type: 'select', options: shipTypes, defaultValue: '0' },
        { id: 'ship_size', label: 'Ship Size', type: 'select', options: shipSizes, defaultValue: '1' },
        { id: 'count', label: 'How many?', type: 'number', defaultValue: '1' },
      ],
      { title: `🚢 Build Ships at (${absX},${absY})`, confirmText: 'Build' }
    );
    this.input.enabled = true;
    if (!result) return;

    this.submitAction({
      ConstructShip: {
        nation: this.state.nationId, x: absX, y: absY,
        ship_type: parseInt(result.ship_type), ship_size: parseInt(result.ship_size),
        count: parseInt(result.count),
      }
    });
    this.setStatus(`Building ships at (${absX},${absY})`);
  }

  // ============ T10: Navy Load/Unload ============

  private async doLoadFleet(): Promise<void> {
    if (this.state.navies.length === 0) { this.setStatus('No navies available.'); return; }
    const active = this.state.armies.filter(a => a.soldiers > 0);
    if (active.length === 0) { this.setStatus('No armies to load.'); return; }

    this.input.enabled = false;
    const fleetIdx = await showSelect(
      'Select fleet to load onto:',
      this.state.navies.map(n => ({
        label: `Fleet ${n.index}: ${n.warships}W ${n.merchant}M ${n.galleys}G at (${n.x},${n.y})`,
      })),
      { title: '📥 Load Fleet' }
    );
    if (fleetIdx < 0) { this.input.enabled = true; return; }
    const fleet = this.state.navies[fleetIdx];

    const armiesHere = active.filter(a => a.x === fleet.x && a.y === fleet.y);
    if (armiesHere.length === 0) { this.input.enabled = true; this.setStatus('No armies at fleet location.'); return; }

    const armyIdx = await showSelect(
      'Select army to load:',
      armiesHere.map(a => ({
        label: `Army ${a.index}: ${a.soldiers} soldiers`,
      })),
      { title: '📥 Load Army' }
    );
    this.input.enabled = true;
    if (armyIdx < 0) return;

    this.submitAction({
      LoadArmyOnFleet: { nation: this.state.nationId, army: armiesHere[armyIdx].index, fleet: fleet.index }
    });
    this.setStatus(`Loading Army ${armiesHere[armyIdx].index} onto Fleet ${fleet.index}`);
  }

  private async doUnloadFleet(): Promise<void> {
    if (this.state.navies.length === 0) { this.setStatus('No navies available.'); return; }

    this.input.enabled = false;
    const fleetIdx = await showSelect(
      'Select fleet to unload from:',
      this.state.navies.map(n => ({
        label: `Fleet ${n.index}: ${n.warships}W ${n.merchant}M ${n.galleys}G at (${n.x},${n.y})`,
      })),
      { title: '📤 Unload Fleet' }
    );
    this.input.enabled = true;
    if (fleetIdx < 0) return;

    const fleet = this.state.navies[fleetIdx];
    this.submitAction({
      UnloadArmyFromFleet: { nation: this.state.nationId, fleet: fleet.index }
    });
    this.setStatus(`Unloading from Fleet ${fleet.index}`);
  }

  // ============ T11: Cast Spell ============

  private async doCastSpell(): Promise<void> {
    if (!this.state.nation) return;
    const spells = [
      { label: '🐉 Summon Creature', detail: 'Summon a monster to fight for you', value: 0 },
      { label: '🦅 Flight', detail: 'Grant an army the ability to fly', value: 1 },
      { label: '⚔ Attack Enhancement', detail: 'Boost attack power', value: 2 },
      { label: '🛡 Defense Enhancement', detail: 'Boost defense power', value: 3 },
      { label: '💥 Destroy', detail: 'Destroy a target sector', value: 4 },
    ];

    this.input.enabled = false;
    const idx = await showSelect(
      `Spell Points: ${this.state.nation.spell_points}`,
      spells.map(s => ({ label: s.label, detail: s.detail })),
      { title: '✨ Cast Spell' }
    );
    this.input.enabled = true;
    if (idx < 0) return;

    const absX = this.state.cursorX + this.state.xOffset;
    const absY = this.state.cursorY + this.state.yOffset;

    this.submitAction({
      CastSpell: {
        nation: this.state.nationId,
        spell_type: spells[idx].value,
        target_x: absX, target_y: absY,
        target_nation: 0,
      }
    });
    this.setStatus(`Cast ${spells[idx].label} at (${absX},${absY})`);
  }

  // ============ T12: Buy Power ============

  private async doBuyPower(): Promise<void> {
    if (!this.state.nation) return;
    const powers = [
      { label: '🔥 Fire Magic', detail: 'Offensive fire spells', value: 0 },
      { label: '❄ Ice Magic', detail: 'Defensive ice barriers', value: 1 },
      { label: '⚡ Lightning', detail: 'Ranged attack spells', value: 2 },
      { label: '🌍 Earth Magic', detail: 'Fortification enhancement', value: 3 },
      { label: '💨 Wind Magic', detail: 'Movement and flight', value: 4 },
      { label: '🌑 Dark Magic', detail: 'Destruction and summoning', value: 5 },
    ];

    this.input.enabled = false;
    const idx = await showSelect(
      `Gold: ${this.state.nation.treasury_gold} — Select power to purchase:`,
      powers.map(p => ({ label: p.label, detail: p.detail })),
      { title: '📖 Buy Magic Power' }
    );
    this.input.enabled = true;
    if (idx < 0) return;

    this.submitAction({
      BuyMagicPower: { nation: this.state.nationId, power_type: powers[idx].value }
    });
    this.setStatus(`Purchased ${powers[idx].label}`);
  }

  // ============ T13: Diplomacy ============

  private async doDiplomacy(): Promise<void> {
    if (!this.state.nation || !this.state.publicNations.length) return;

    const nations = this.state.publicNations.filter(
      n => n.nation_id !== this.state.nationId && n.active > 0
    );
    if (nations.length === 0) { this.setStatus('No other nations to negotiate with.'); return; }

    const diplomacy = this.state.nation.diplomacy || [];
    const items = nations.map(n => {
      const diploStatus = diplomacy[n.nation_id] ?? 0;
      const statusName = DIPLO_NAMES[diploStatus] ?? 'UNMET';
      return {
        label: `${n.name} (${n.race}) — ${statusName}`,
        detail: `Score: ${n.score}`,
      };
    });

    this.input.enabled = false;
    const idx = await showSelect(
      'Select a nation to adjust diplomacy:',
      items,
      { title: '🤝 Diplomacy' }
    );
    if (idx < 0) { this.input.enabled = true; return; }

    const target = nations[idx];
    const currentStatus = diplomacy[target.nation_id] ?? 0;
    const statusOptions = DIPLO_NAMES.map((name, i) => ({
      label: name + (i === currentStatus ? ' (current)' : ''),
      disabled: i === currentStatus,
    }));

    const newStatus = await showSelect(
      `Set diplomacy with ${target.name}:`,
      statusOptions,
      { title: '🤝 Adjust Diplomacy' }
    );
    this.input.enabled = true;
    if (newStatus < 0) return;

    this.submitAction({
      AdjustDiplomacy: { nation_a: this.state.nationId, nation_b: target.nation_id, status: newStatus }
    });
    this.setStatus(`Diplomacy with ${target.name} set to ${DIPLO_NAMES[newStatus]}`);
  }

  // ============ T14: Propose Trade ============

  private async doProposeTrade(): Promise<void> {
    if (!this.state.publicNations.length) return;

    const nations = this.state.publicNations.filter(
      n => n.nation_id !== this.state.nationId && n.active > 0
    );
    if (nations.length === 0) { this.setStatus('No nations to trade with.'); return; }

    const tradeTypes = [
      { label: 'Gold', value: '0' },
      { label: 'Food', value: '1' },
      { label: 'Metal', value: '2' },
      { label: 'Jewels', value: '3' },
    ];

    this.input.enabled = false;
    const result = await showForm(
      [
        { id: 'target', label: 'Target Nation', type: 'select',
          options: nations.map(n => ({ label: `${n.name} (${n.race})`, value: String(n.nation_id) })),
          defaultValue: String(nations[0].nation_id),
        },
        { id: 'offer_type', label: 'Offer Type', type: 'select', options: tradeTypes, defaultValue: '0' },
        { id: 'offer_amount', label: 'Offer Amount', type: 'number', defaultValue: '100' },
        { id: 'request_type', label: 'Request Type', type: 'select', options: tradeTypes, defaultValue: '1' },
        { id: 'request_amount', label: 'Request Amount', type: 'number', defaultValue: '100' },
      ],
      { title: '💲 Propose Trade', confirmText: 'Propose' }
    );
    this.input.enabled = true;
    if (!result) return;

    this.submitAction({
      ProposeTrade: {
        nation: this.state.nationId,
        target_nation: parseInt(result.target),
        offer_type: parseInt(result.offer_type),
        offer_amount: parseInt(result.offer_amount),
        request_type: parseInt(result.request_type),
        request_amount: parseInt(result.request_amount),
      }
    });
    this.setStatus('Trade proposal sent!');
  }

  // ============ T15: Pending Trades ============

  private async showPendingTrades(): Promise<void> {
    this.input.enabled = false;
    await showAlert('No pending trade proposals at this time.', '📨 Pending Trades');
    this.input.enabled = true;
  }

  // ============ T16: Hire Mercs ============

  private async doHireMercs(): Promise<void> {
    if (!this.state.nation) return;

    this.input.enabled = false;
    const result = await showInput(
      `How many mercenaries to hire?\nYour gold: ${this.state.nation.treasury_gold}`,
      { title: '💂 Hire Mercenaries', defaultValue: '50', inputType: 'number', confirmText: 'Hire' }
    );
    this.input.enabled = true;
    if (result === null) return;

    const count = parseInt(result);
    if (isNaN(count) || count < 1) { this.setStatus('Invalid count.'); return; }

    this.submitAction({
      HireMercenaries: { nation: this.state.nationId, men: count }
    });
    this.setStatus(`Hiring ${count} mercenaries`);
  }

  // ============ T17: Bribe ============

  private async doBribe(): Promise<void> {
    if (!this.state.publicNations.length) return;

    const nations = this.state.publicNations.filter(
      n => n.nation_id !== this.state.nationId && n.active > 0
    );
    if (nations.length === 0) { this.setStatus('No nations to bribe.'); return; }

    this.input.enabled = false;
    const result = await showForm(
      [
        { id: 'target', label: 'Target Nation', type: 'select',
          options: nations.map(n => ({ label: `${n.name} (${n.race})`, value: String(n.nation_id) })),
          defaultValue: String(nations[0].nation_id),
        },
        { id: 'amount', label: `Gold to spend (have: ${this.state.nation?.treasury_gold ?? 0})`, type: 'number', defaultValue: '500' },
      ],
      { title: '💰 Bribe Nation', confirmText: 'Bribe' }
    );
    this.input.enabled = true;
    if (!result) return;

    this.submitAction({
      BribeNation: { nation: this.state.nationId, cost: parseInt(result.amount), target: parseInt(result.target) }
    });
    this.setStatus(`Bribe attempted on nation ${result.target}`);
  }

  // ============ T18: Send Tribute ============

  private async doSendTribute(): Promise<void> {
    if (!this.state.publicNations.length) return;

    const nations = this.state.publicNations.filter(
      n => n.nation_id !== this.state.nationId && n.active > 0
    );
    if (nations.length === 0) { this.setStatus('No nations to send tribute to.'); return; }

    this.input.enabled = false;
    const result = await showForm(
      [
        { id: 'target', label: 'Target Nation', type: 'select',
          options: nations.map(n => ({ label: `${n.name} (${n.race})`, value: String(n.nation_id) })),
          defaultValue: String(nations[0].nation_id),
        },
        { id: 'gold', label: 'Gold', type: 'number', defaultValue: '0' },
        { id: 'food', label: 'Food', type: 'number', defaultValue: '0' },
        { id: 'metal', label: 'Metal', type: 'number', defaultValue: '0' },
        { id: 'jewels', label: 'Jewels', type: 'number', defaultValue: '0' },
      ],
      { title: '🎁 Send Tribute', confirmText: 'Send' }
    );
    this.input.enabled = true;
    if (!result) return;

    this.submitAction({
      SendTribute: {
        nation: this.state.nationId,
        target: parseInt(result.target),
        gold: parseInt(result.gold) || 0,
        food: parseInt(result.food) || 0,
        metal: parseInt(result.metal) || 0,
        jewels: parseInt(result.jewels) || 0,
      }
    });
    this.setStatus(`Tribute sent to ${result.target}`);
  }

  // ============ T19: Budget ============

  private async showBudget(): Promise<void> {
    if (!this.state.nation) return;
    const n = this.state.nation;

    this.input.enabled = false;
    await showAlert(
      `═══ NATION BUDGET ═══\n` +
      `Gold: ${n.treasury_gold.toLocaleString()}   Food: ${n.total_food.toLocaleString()}\n` +
      `Metal: ${n.metals.toLocaleString()}   Jewels: ${n.jewels.toLocaleString()}\n` +
      `───────────────────────\n` +
      `Military: ${n.total_mil.toLocaleString()} soldiers\n` +
      `Civilians: ${n.total_civ.toLocaleString()} people\n` +
      `Sectors: ${n.total_sectors}   Ships: ${n.total_ships}\n` +
      `───────────────────────\n` +
      `Tax Rate: ${n.tax_rate}%   Charity: ${n.charity}\n` +
      `Popularity: ${n.popularity}  Terror: ${n.terror}\n` +
      `Reputation: ${n.reputation}\n` +
      `Attack+: ${n.attack_plus}  Defense+: ${n.defense_plus}\n` +
      `Spell Points: ${n.spell_points}  Score: ${n.score}`,
      '💰 Budget Report'
    );
    this.input.enabled = true;
  }

  private handleCommand(cmd: string): void {
    // Sidebar font sync
    if (cmd === '_sidebar_font_changed') {
      this.statsSidebar.fontSize = this.cmdSidebar.fontSize;
      return;
    }

    // Layer toggle commands
    if (cmd.startsWith('layer_')) {
      const layerName = cmd.substring(6);
      if (layerName === 'all') {
        this.state.layerOverrides = { terrain: true, vegetation: true, designation: true, resources: true, ownership: true, units: true, cursor: true };
        this.setStatus('All layers ON');
      } else if (layerName === 'mode_default') {
        this.state.layerOverrides = null;
        this.setStatus('Layers: mode default');
      } else {
        // Toggle individual layer
        if (!this.state.layerOverrides) {
          this.state.layerOverrides = { ...layersForMode(this.state.displayMode) };
        }
        const current = (this.state.layerOverrides as any)[layerName] ?? false;
        (this.state.layerOverrides as any)[layerName] = !current;
        this.setStatus(`${layerName}: ${!current ? 'ON' : 'OFF'}`);
      }
      return;
    }

    // UI theme commands
    if (cmd.startsWith('uitheme_')) {
      const uiId = cmd.substring(8);
      this.layout.uiThemeId = uiId;
      this.cmdSidebar.themeId = uiId;
      this.statsSidebar.themeId = uiId;
      applyUiThemeCss(uiId);
      this.setStatus(`UI Theme: ${uiId}`);
      return;
    }

    // Tileset editor
    if (cmd === 'tileset_editor') {
      this.input.enabled = false;
      const oldCellPx = this.getCurrentCellPixelSize();
      const canvasW = this.canvas.width;
      const canvasH = this.canvas.height;
      const centerMapX = this.state.xOffset + (canvasW / 2) / oldCellPx.w;
      const centerMapY = this.state.yOffset + (canvasH / 2) / oldCellPx.h;
      new TilesetEditor(
        document.body,
        this.state.tilesetId ?? 'ascii',
        (ts) => {
          registerTileset(ts);
          this.state.tilesetId = ts.id;
          localStorage.setItem('conquer_tileset', ts.id);
          this.matchCellSize(ts, oldCellPx);
          const newCellPx = this.getCurrentCellPixelSize();
          this.state.xOffset = Math.round(centerMapX - (canvasW / 2) / newCellPx.w);
          this.state.yOffset = Math.round(centerMapY - (canvasH / 2) / newCellPx.h);
          this.setStatus(`Tileset saved: ${ts.name}`);
        },
        () => { this.input.enabled = true; },
      );
      return;
    }

    // Tileset commands
    if (cmd.startsWith('tileset_')) {
      const tsId = cmd.substring(8);
      const ts = getTilesetById(tsId);

      // Capture current cell pixel size and canvas center in map coords
      const oldCellPx = this.getCurrentCellPixelSize();
      const canvasW = this.canvas.width;
      const canvasH = this.canvas.height;
      // Map cell at pixel center of canvas
      const centerMapX = this.state.xOffset + (canvasW / 2) / oldCellPx.w;
      const centerMapY = this.state.yOffset + (canvasH / 2) / oldCellPx.h;

      this.state.tilesetId = tsId;
      localStorage.setItem('conquer_tileset', tsId);

      // Adjust font size so new tileset's cell pixel size matches the old one
      this.matchCellSize(ts, oldCellPx);

      const reanchor = () => {
        // Get the NEW cell pixel size after matchCellSize
        const newCellPx = this.getCurrentCellPixelSize();
        // Offset so the same map cell is at the pixel center
        this.state.xOffset = Math.round(centerMapX - (canvasW / 2) / newCellPx.w);
        this.state.yOffset = Math.round(centerMapY - (canvasH / 2) / newCellPx.h);
      };

      // Preload images for image-based tilesets
      if (ts.tileType === 'image') {
        this.setStatus(`Loading tileset: ${ts.name}...`);
        preloadTilesetImages(ts).then(() => {
          reanchor();
          this.setStatus(`Tileset: ${ts.name}`);
          this.renderFrame();
        });
      } else {
        reanchor();
        this.setStatus(`Tileset: ${ts.name}`);
      }
      return;
    }

    // Theme commands
    if (cmd.startsWith('theme_')) {
      const themeId = cmd.slice(6);
      const theme = getTheme(themeId);
      this.state.themeId = themeId;
      this.state.renderMode = themeId.startsWith('classic') ? 'classic' : 'enhanced';
      this.setStatus(`Theme: ${theme.name} — ${theme.description}`);
      // Save preference
      localStorage.setItem('conquer_theme', themeId);
      return;
    }

    const displayMap: Record<string, DisplayMode> = {
      disp_veg: DisplayMode.Vegetation, disp_des: DisplayMode.Designation,
      disp_cnt: DisplayMode.Contour, disp_food: DisplayMode.Food,
      disp_race: DisplayMode.Race, disp_ntn: DisplayMode.Nation,
      disp_move: DisplayMode.Move, disp_def: DisplayMode.Defense,
      disp_pop: DisplayMode.People, disp_gold: DisplayMode.Gold,
      disp_mtl: DisplayMode.Metal, disp_itm: DisplayMode.Items,
    };
    const hlMap: Record<string, HighlightMode> = {
      hl_own: HighlightMode.Own, hl_army: HighlightMode.Army,
      hl_yours: HighlightMode.YourArmy, hl_move: HighlightMode.Move,
      hl_trade: HighlightMode.Good, hl_none: HighlightMode.None,
    };

    if (displayMap[cmd] !== undefined) {
      this.handleAction({ type: 'set_display', mode: displayMap[cmd] });
    } else if (hlMap[cmd] !== undefined) {
      this.handleAction({ type: 'set_highlight', mode: hlMap[cmd] });
    } else {
      switch (cmd) {
        case 'move_up': this.handleAction({ type: 'move_cursor', dx: 0, dy: -1 }); break;
        case 'move_down': this.handleAction({ type: 'move_cursor', dx: 0, dy: 1 }); break;
        case 'move_left': this.handleAction({ type: 'move_cursor', dx: -1, dy: 0 }); break;
        case 'move_right': this.handleAction({ type: 'move_cursor', dx: 1, dy: 0 }); break;
        case 'next_army': this.handleAction({ type: 'select_next_army' }); break;
        case 'prev_army': this.handleAction({ type: 'select_prev_army' }); break;
        case 'army_move': this.handleAction({ type: 'move_army', dx: 0, dy: 0 }); break;
        case 'toggle_navy': this.handleAction({ type: 'toggle_army_navy' }); break;
        case 'jump_capitol': this.handleAction({ type: 'jump_capitol' }); break;
        case 'show_scores': this.handleAction({ type: 'show_scores' }); break;
        case 'show_news': this.handleAction({ type: 'show_news' }); break;
        case 'show_budget': this.handleAction({ type: 'show_budget' }); break;
        case 'toggle_chat': this.handleAction({ type: 'toggle_chat' }); break;
        case 'end_turn': this.handleAction({ type: 'end_turn' }); break;
        case 'redesignate': this.handleAction({ type: 'redesignate' }); break;
        case 'draft': this.handleAction({ type: 'draft' }); break;
        case 'build_fort': this.handleAction({ type: 'build_fort' }); break;
        case 'build_road': this.handleAction({ type: 'build_road' }); break;
        case 'build_ship': this.handleAction({ type: 'build_ship' }); break;
        case 'diplomacy': this.handleAction({ type: 'diplomacy' }); break;
        case 'cast_spell': this.handleAction({ type: 'cast_spell' }); break;
        case 'buy_power': this.handleAction({ type: 'buy_power' }); break;
        case 'propose_trade': this.handleAction({ type: 'propose_trade' }); break;
        case 'pending_trades': this.showPendingTrades(); break;
        case 'hire_mercs': this.handleAction({ type: 'hire_mercs' }); break;
        case 'bribe': this.handleAction({ type: 'bribe' }); break;
        case 'send_tribute': this.handleAction({ type: 'send_tribute' }); break;
        case 'set_army_attack': this.handleAction({ type: 'set_army_attack' }); break;
        case 'set_army_defend': this.handleAction({ type: 'set_army_defend' }); break;
        case 'set_army_garrison': this.handleAction({ type: 'set_army_garrison' }); break;
        case 'set_army_scout': this.handleAction({ type: 'set_army_scout' }); break;
        case 'set_army_rule': this.handleAction({ type: 'set_army_rule' }); break;
        case 'set_army_march': this.handleAction({ type: 'set_army_march' }); break;
        case 'split_army': this.handleAction({ type: 'split_army' }); break;
        case 'combine_army': this.handleAction({ type: 'combine_army' }); break;
        case 'divide_army': this.handleAction({ type: 'divide_army' }); break;
        case 'load_fleet': this.handleAction({ type: 'load_fleet' }); break;
        case 'unload_fleet': this.handleAction({ type: 'unload_fleet' }); break;
        case 'refresh': this.loadGameData(); this.setStatus('Refreshing...'); break;
        case 'font_up':
          this.zoomCentered(2);
          break;
        case 'font_down':
          this.zoomCentered(-2);
          break;
        case 'center_map':
          this.handleAction({ type: 'center_map' });
          break;
        case 'toggle_sidebars':
          this.layout.toggleLeft();
          this.layout.toggleRight();
          this.handleResize();
          this.setStatus(this.layout.leftBar.style.display === 'none' ? 'Focus mode' : 'Sidebars visible');
          break;
        case 'keybindings':
          this.input.enabled = false;
          new KeybindingsModal(document.body, this.keybindingsManager, () => {
            this.input.enabled = true;
          });
          break;
        case 'back_to_lobby':
          localStorage.removeItem('conquer_game_id');
          localStorage.removeItem('conquer_nation_id');
          window.location.reload();
          break;
      }
    }
  }

  private handleWsMessage(msg: ServerMessage): void {
    switch (msg.type) {
      case 'turn_end':
        this.state.isDone = false;
        this.addNotification(`Turn ${msg.data.old_turn} ended — now turn ${msg.data.new_turn}`);
        // Reload all data
        this.loadGameData();
        break;

      case 'turn_start':
        this.addNotification(`Turn ${msg.data.turn} (${msg.data.season})`);
        break;

      case 'player_joined':
        this.addNotification(`${msg.data.nation_name} (${msg.data.race}) has joined!`);
        // Refresh nations list for chat panel
        if (this.state.gameId) {
          this.client.getNations(this.state.gameId).then(nations => {
            this.state.publicNations = nations;
            this.chatPanel.updateNations();
          });
        }
        break;

      case 'player_done':
        this.addNotification(`${msg.data.nation_name} has ended their turn`);
        break;

      case 'chat_message':
        // Route to chat panel (T388)
        this.chatPanel.onChatMessage(msg.data);
        break;

      case 'chat_history':
        // Route history to chat panel
        this.chatPanel.onChatHistory(msg.data.channel, msg.data.messages);
        break;

      case 'presence_update':
        // Route to chat panel (T405)
        this.chatPanel.onPresenceUpdate(msg.data.nation_id, msg.data.status);
        break;

      case 'system_message':
        this.addNotification(msg.data.content);
        break;

      case 'map_update':
        // Refresh map from server
        if (this.state.gameId) {
          this.client.getMap(this.state.gameId).then(m => {
            this.state.mapData = m;
            buildOccupied(this.state);
          });
        }
        break;

      case 'nation_update':
        if (msg.data.nation_id === this.state.nationId && this.state.gameId) {
          this.client.getNation(this.state.gameId).then(n => {
            this.state.nation = n;
          });
        }
        break;

      default:
        break;
    }
  }

  private setStatus(msg: string): void {
    this.statusMessage = msg;
    if (this.statusTimeout) clearTimeout(this.statusTimeout);
    this.statusTimeout = window.setTimeout(() => {
      this.statusMessage = '';
    }, 8000);
  }

  private addNotification(msg: string): void {
    this.state.notifications.push(msg);
    // Keep last 10
    if (this.state.notifications.length > 10) {
      this.state.notifications.shift();
    }
    this.setStatus(msg);
  }

  private startRenderLoop(): void {
    const loop = () => {
      this.renderFrame();
      this.animFrame = requestAnimationFrame(loop);
    };
    this.animFrame = requestAnimationFrame(loop);
  }

  private renderFrame(): void {
    const tsId = this.state.tilesetId ?? 'ascii';
    const isDirectCanvas = tsId !== 'ascii' && tsId !== 'unicode';

    if (isDirectCanvas) {
      const ctx = this.term.getContext();
      ctx.fillStyle = '#000';
      ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);

      const ts = getTilesetById(tsId);

      if (ts.tileType === 'image') {
        // Image/sprite tilesets: multi-layer compositor with z-order compositing
        const layers: LayerConfig = this.state.layerOverrides
          ? { ...DEFAULT_LAYERS, ...this.state.layerOverrides, cursor: true } as LayerConfig
          : layersForMode(this.state.displayMode);
        renderCompositedMap(ctx, this.state, ts, this.term.fontSize, ctx.canvas.width, ctx.canvas.height, layers);
      } else {
        // Emoji tilesets: single-layer per-mode mapping (no compositing)
        renderMap(this.term, this.state);
      }

      // Blinking cursor overlay
      renderTilesetCursor(ctx, this.state, ts, this.term.fontSize);

      // Render bottom panel as text overlay on canvas
      const fontSize = this.term.fontSize;
      ctx.font = `${fontSize}px "Courier New", monospace`;
      ctx.textBaseline = 'top';
      const panelY = ctx.canvas.height - fontSize * 3.5;
      ctx.fillStyle = 'rgba(0,0,0,0.85)';
      ctx.fillRect(0, panelY, ctx.canvas.width, fontSize * 3.5);
      ctx.fillStyle = '#55ff55';
      // Use the term bottom panel for text content
      this.term.clear();
      renderBottomPanel(this.term, this.state, this.statusMessage);
      // Draw just the bottom panel rows from term buffer
      this.term.renderPartial(panelY);
    } else {
      // Classic char rendering: full terminal path
      this.term.clear();
      renderMap(this.term, this.state);
      renderBottomPanel(this.term, this.state, this.statusMessage);
      this.term.setCursor(this.state.cursorX * 2, this.state.cursorY);
      this.term.render();
    }

    // Update right sidebar stats
    this.statsSidebar.update(this.state);

    // Update tooltip state
    this.tooltip.setState(this.state);
  }

  destroy(): void {
    if (this.animFrame) cancelAnimationFrame(this.animFrame);
    this.input.destroy();
    this.mouseHandler.destroy();
    this.tooltip.destroy();
    this.chatPanel.destroy();
    this.cmdSidebar.destroy();
    this.statsSidebar.destroy();
    this.term.destroy();
    this.client.disconnectWebSocket();
    this.layout.destroy();
  }
}
