// gameScreen.ts — Main game screen: ties renderer, map, panels, input, network
// Orchestrates the full game UI

import { TerminalRenderer } from '../renderer/terminal';
import { GameClient } from '../network/client';
import { GameState, createInitialState, buildOccupied } from '../state/gameState';
import { renderMap, screenSize } from './mapView';
import { renderBottomPanel } from '../ui/sidePanel';
import { ChatPanel } from '../ui/chatPanel';
import { HudOverlay } from '../ui/hudOverlay';
import { CommandBar } from '../ui/commandBar';
import { InputHandler, GameAction } from './inputHandler';
import { ServerMessage, DisplayMode, HighlightMode } from '../types';
import { CURSES_COLORS } from '../renderer/colors';
import { getTheme, ALL_THEMES } from '../renderer/themes';

export class GameScreen {
  private canvas: HTMLCanvasElement;
  private term: TerminalRenderer;
  private client: GameClient;
  private input: InputHandler;
  private chatPanel: ChatPanel;
  private hud: HudOverlay;
  private commandBar: CommandBar;
  private _backBtn: HTMLElement | null = null;
  private state: GameState;
  private animFrame: number = 0;
  private statusMessage: string = '';
  private statusTimeout: number = 0;

  constructor(parent: HTMLElement, client: GameClient, gameId: string, nationId: number) {
    this.client = client;
    this.state = createInitialState();
    this.state.token = client.getToken();
    this.state.gameId = gameId;
    this.state.nationId = nationId;

    // Create canvas
    this.canvas = document.createElement('canvas');
    this.canvas.style.display = 'block';
    this.canvas.style.background = '#000';
    parent.appendChild(this.canvas);

    // Initialize renderer
    this.term = new TerminalRenderer(this.canvas);
    this.handleResize();
    window.addEventListener('resize', () => this.handleResize());

    // Input handler
    this.input = new InputHandler((action) => this.handleAction(action));

    // Back button — always visible, top-left
    const backBtn = document.createElement('button');
    backBtn.id = 'btn-back-to-lobby';
    backBtn.textContent = '← Lobby';
    backBtn.style.cssText = `
      position: fixed; top: calc(env(safe-area-inset-top, 0px) + 4px); left: 4px; z-index: 60;
      background: rgba(0,17,0,0.9); color: #55ff55; border: 1px solid #338833;
      border-radius: 4px; padding: 6px 10px; font-family: inherit; font-size: 12px;
      cursor: pointer; backdrop-filter: blur(4px); -webkit-backdrop-filter: blur(4px);
    `;
    backBtn.addEventListener('click', () => {
      localStorage.removeItem('conquer_game_id');
      localStorage.removeItem('conquer_nation_id');
      window.location.reload();
    });
    parent.appendChild(backBtn);
    this._backBtn = backBtn;

    // HTML HUD overlay (replaces canvas-rendered side panel)
    this.hud = new HudOverlay(parent);

    // Command button bar
    this.commandBar = new CommandBar(parent, (cmd) => this.handleCommand(cmd));

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
    const barH = this.commandBar ? this.commandBar.getHeight() : 80;
    this.canvas.width = window.innerWidth;
    this.canvas.height = window.innerHeight - barH;
    this.term.resize(window.innerWidth, window.innerHeight - barH);
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
          this.setStatus(`Army ${army.index}: ${army.soldiers} soldiers`);
        }
        break;
      }

      case 'select_prev_army': {
        const active = this.state.armies.filter(a => a.soldiers > 0);
        if (active.length > 0) {
          this.state.selectedArmy = (this.state.selectedArmy - 1 + active.length) % active.length;
          const army = active[this.state.selectedArmy];
          this.centerOn(army.x, army.y);
          this.setStatus(`Army ${army.index}: ${army.soldiers} soldiers`);
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
          // Submit move action
          this.submitAction({
            MoveArmy: { nation: this.state.nationId, army: army.index, x: nx, y: ny }
          });
          this.setStatus(`Moving army ${army.index} to (${nx},${ny})`);
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
        this.setStatus('Budget view — press any key to return');
        break;

      case 'show_help':
        this.showHelp();
        break;

      case 'redesignate':
        this.setStatus('Redesignate: select sector, then press designation key');
        break;

      case 'draft':
        this.setStatus('Draft: not yet implemented in frontend');
        break;

      case 'diplomacy':
        this.setStatus('Diplomacy: not yet implemented in frontend');
        break;

      case 'magic':
        this.setStatus('Magic: not yet implemented in frontend');
        break;

      case 'toggle_chat':
        this.chatPanel.toggle();
        break;

      case 'font_increase':
        this.term.setFontSize(this.term.fontSize + 1);
        this.handleResize();
        break;

      case 'font_decrease':
        this.term.setFontSize(this.term.fontSize - 1);
        this.handleResize();
        break;
    }
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

  private showHelp(): void {
    this.setStatus(
      'Keys: arrows/hjkl=move  Tab=army  d/v/c/f/r/m/M/D/p/J/i/n=display  o/a/y/L/s/x=hl  T=chat  E=end turn  ?=help  +/-=font'
    );
  }

  private handleCommand(cmd: string): void {
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
        case 'toggle_chat': this.handleAction({ type: 'toggle_chat' }); break;
        case 'end_turn': this.handleAction({ type: 'end_turn' }); break;
        case 'refresh': this.loadGameData(); this.setStatus('Refreshing...'); break;
        case 'back_to_lobby':
          localStorage.removeItem('conquer_game_id');
          localStorage.removeItem('conquer_nation_id');
          window.location.reload();
          break;
        case 'font_up':
          this.term.setFontSize(this.term.fontSize + 2);
          this.handleResize();
          break;
        case 'font_down':
          this.term.setFontSize(this.term.fontSize - 2);
          this.handleResize();
          break;
        case 'center_map':
          this.handleAction({ type: 'center_map' });
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
    this.term.clear();
    renderMap(this.term, this.state);
    renderBottomPanel(this.term, this.state, this.statusMessage);

    // Set cursor position on map
    this.term.setCursor(this.state.cursorX * 2, this.state.cursorY);

    this.term.render();

    // Update HTML HUD overlay
    this.hud.update(this.state);
  }

  destroy(): void {
    if (this.animFrame) cancelAnimationFrame(this.animFrame);
    this.input.destroy();
    this.chatPanel.destroy();
    this.hud.destroy();
    this.commandBar.destroy();
    this.term.destroy();
    this.client.disconnectWebSocket();
    this._backBtn?.remove();
    this.canvas.remove();
  }
}
