// lobby.ts — Game lobby screen: login, register, list/create/join games
// T374, T381-T382: Login/register flow, game lobby, nation creation

import { GameClient } from '../network/client';
import { GameInfo, RACE_NAMES, CLASS_NAMES } from '../types';
import { ProfilePage } from './profilePage';
import { GameCreationWizard } from './gameCreationWizard';
import { AdminPanel } from './adminPanel';
import { InviteManager } from './invitePage';
import { NotificationBell } from './notifications';

// Terminal-retro styles for the lobby
const LOBBY_CSS = `
  .lobby {
    font-family: "Courier New", "Consolas", "Liberation Mono", monospace;
    background: #000;
    color: #aaa;
    min-height: 100vh;
    padding: 20px;
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  .lobby h1 {
    color: #55ff55;
    text-shadow: 0 0 10px #00aa00;
    font-size: 2em;
    margin-bottom: 10px;
  }
  .lobby h2 {
    color: #ffff55;
    font-size: 1.2em;
    margin: 15px 0 5px;
  }
  .lobby input, .lobby select {
    font-family: inherit;
    background: #111;
    color: #55ff55;
    border: 1px solid #333;
    padding: 6px 10px;
    font-size: 14px;
    outline: none;
    width: 220px;
  }
  .lobby input:focus, .lobby select:focus {
    border-color: #55ff55;
  }
  .lobby button {
    font-family: inherit;
    background: #222;
    color: #55ff55;
    border: 1px solid #55ff55;
    padding: 8px 16px;
    cursor: pointer;
    font-size: 14px;
    margin: 4px;
  }
  .lobby button:hover {
    background: #55ff55;
    color: #000;
  }
  .lobby .error {
    color: #ff5555;
    margin: 5px 0;
  }
  .lobby .game-list {
    margin: 10px 0;
    max-width: 600px;
    width: 100%;
  }
  .lobby .game-item {
    border: 1px solid #333;
    padding: 8px;
    margin: 4px 0;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .lobby .game-item:hover {
    border-color: #55ff55;
  }
  .lobby .game-name { color: #55ffff; }
  .lobby .game-status { color: #aaa; font-size: 0.9em; }
  .lobby .form-row {
    margin: 6px 0;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .lobby .form-row label {
    width: 80px;
    text-align: right;
    color: #aaa;
  }
  .lobby .section { margin: 20px 0; max-width: 600px; width: 100%; }
  .lobby .ascii-art { color: #00aa00; font-size: 12px; white-space: pre; line-height: 1.1; text-align: center; margin: 10px 0; }
`;

export class LobbyScreen {
  private container: HTMLDivElement;
  private client: GameClient;
  private onGameStart: (gameId: string, nationId: number) => void;
  private isLoggedIn = false;
  private parent: HTMLElement;
  private subScreen: ProfilePage | GameCreationWizard | AdminPanel | InviteManager | null = null;
  private notifBell: NotificationBell | null = null;

  constructor(
    parent: HTMLElement,
    client: GameClient,
    onGameStart: (gameId: string, nationId: number) => void,
  ) {
    this.client = client;
    this.onGameStart = onGameStart;
    this.parent = parent;

    // Inject CSS
    const style = document.createElement('style');
    style.textContent = LOBBY_CSS;
    document.head.appendChild(style);

    this.container = document.createElement('div');
    this.container.className = 'lobby';
    parent.appendChild(this.container);

    this.isLoggedIn = !!client.getToken();
    this.render();
  }

  private render(): void {
    if (this.isLoggedIn) {
      this.renderGameList();
    } else {
      this.renderAuth();
    }
  }

  private renderAuth(): void {
    this.container.innerHTML = `
      <div class="ascii-art">
   ██████╗ ██████╗ ███╗   ██╗ ██████╗ ██╗   ██╗███████╗██████╗
  ██╔════╝██╔═══██╗████╗  ██║██╔═══██╗██║   ██║██╔════╝██╔══██╗
  ██║     ██║   ██║██╔██╗ ██║██║   ██║██║   ██║█████╗  ██████╔╝
  ██║     ██║   ██║██║╚██╗██║██║▄▄ ██║██║   ██║██╔══╝  ██╔══██╗
  ╚██████╗╚██████╔╝██║ ╚████║╚██████╔╝╚██████╔╝███████╗██║  ██║
   ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝ ╚══▀▀═╝  ╚═════╝ ╚══════╝╚═╝  ╚═╝
      </div>
      <h1>CONQUER</h1>
      <p style="color:#aaa;margin-bottom:20px;">A Strategy Game of Strife and Diplomacy — Since 1988</p>

      <div class="section">
        <h2>Login</h2>
        <div class="form-row">
          <label>User:</label>
          <input id="login-user" type="text" placeholder="username">
        </div>
        <div class="form-row">
          <label>Pass:</label>
          <input id="login-pass" type="password" placeholder="password">
        </div>
        <button id="btn-login">Login</button>
        <div id="login-error" class="error"></div>
      </div>

      <div class="section">
        <h2>Register</h2>
        <div class="form-row">
          <label>User:</label>
          <input id="reg-user" type="text" placeholder="username">
        </div>
        <div class="form-row">
          <label>Email:</label>
          <input id="reg-email" type="text" placeholder="email">
        </div>
        <div class="form-row">
          <label>Pass:</label>
          <input id="reg-pass" type="password" placeholder="password">
        </div>
        <button id="btn-register">Register</button>
        <div id="reg-error" class="error"></div>
      </div>
    `;

    document.getElementById('btn-login')!.addEventListener('click', () => this.doLogin());
    document.getElementById('btn-register')!.addEventListener('click', () => this.doRegister());

    // Enter key in login fields
    document.getElementById('login-pass')!.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') this.doLogin();
    });
    document.getElementById('reg-pass')!.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') this.doRegister();
    });
  }

  private async doLogin(): Promise<void> {
    const user = (document.getElementById('login-user') as HTMLInputElement).value;
    const pass = (document.getElementById('login-pass') as HTMLInputElement).value;
    const errorEl = document.getElementById('login-error')!;
    try {
      await this.client.login(user, pass);
      this.isLoggedIn = true;
      this.render();
    } catch (e) {
      errorEl.textContent = `Login failed: ${(e as Error).message}`;
    }
  }

  private async doRegister(): Promise<void> {
    const user = (document.getElementById('reg-user') as HTMLInputElement).value;
    const email = (document.getElementById('reg-email') as HTMLInputElement).value;
    const pass = (document.getElementById('reg-pass') as HTMLInputElement).value;
    const errorEl = document.getElementById('reg-error')!;
    try {
      await this.client.register(user, email, pass);
      this.isLoggedIn = true;
      this.render();
    } catch (e) {
      errorEl.textContent = `Registration failed: ${(e as Error).message}`;
    }
  }

  private returnFromSub(): void {
    if (this.subScreen) {
      if ('destroy' in this.subScreen) (this.subScreen as any).destroy();
      this.subScreen = null;
    }
    this.parent.innerHTML = '';
    this.container = document.createElement('div');
    this.container.className = 'lobby';
    this.parent.appendChild(this.container);
    this.render();
  }

  private async renderGameList(): Promise<void> {
    // Show notification bell (T432)
    if (!this.notifBell) {
      this.notifBell = new NotificationBell(document.body, this.client);
    }

    this.container.innerHTML = `
      <h1>CONQUER</h1>
      <p style="color:#55ff55;">Welcome, ${localStorage.getItem('conquer_username') ?? 'player'}!</p>
      <div style="position:absolute;top:10px;right:10px;display:flex;gap:8px;">
        <button id="btn-profile" style="font-family:inherit;background:#222;color:#55ffff;border:1px solid #55ffff;padding:6px 12px;cursor:pointer;">⚡ Profile</button>
        <button id="btn-logout" style="font-family:inherit;background:#222;color:#ff5555;border:1px solid #ff5555;padding:6px 12px;cursor:pointer;">Logout</button>
      </div>

      <div class="section">
        <h2>Create New Game</h2>
        <button id="btn-create-wizard" style="font-family:inherit;background:#004400;color:#55ff55;border:1px solid #55ff55;padding:10px 20px;cursor:pointer;font-size:14px;">⚔ New Game Wizard</button>
      </div>

      <div class="section">
        <h2>Available Games</h2>
        <button id="btn-refresh">Refresh</button>
        <div id="game-list" class="game-list">Loading...</div>
      </div>

      <div id="join-section" class="section" style="display:none;">
        <h2>Join Game</h2>
        <div id="join-game-name" style="color:#55ffff;margin-bottom:10px;"></div>
        <div class="form-row">
          <label>Nation:</label>
          <input id="join-nation" type="text" placeholder="nation name" maxlength="9">
        </div>
        <div class="form-row">
          <label>Leader:</label>
          <input id="join-leader" type="text" placeholder="leader name" maxlength="9">
        </div>
        <div class="form-row">
          <label>Race:</label>
          <select id="join-race">
            <option value="H">Human</option>
            <option value="E">Elf</option>
            <option value="D">Dwarf</option>
            <option value="O">Orc</option>
          </select>
        </div>
        <div class="form-row">
          <label>Class:</label>
          <select id="join-class">
            <option value="1">King</option>
            <option value="2">Emperor</option>
            <option value="3">Wizard</option>
            <option value="4">Priest</option>
            <option value="6">Trader</option>
            <option value="7">Warlord</option>
          </select>
        </div>
        <div class="form-row">
          <label>Mark:</label>
          <input id="join-mark" type="text" placeholder="*" maxlength="1" value="*" style="width:40px;">
        </div>
        <button id="btn-join">Join</button>
        <div id="join-error" class="error"></div>
      </div>
    `;

    document.getElementById('btn-logout')!.addEventListener('click', () => {
      localStorage.removeItem('conquer_token');
      localStorage.removeItem('conquer_user_id');
      localStorage.removeItem('conquer_username');
      if (this.notifBell) { this.notifBell.destroy(); this.notifBell = null; }
      this.isLoggedIn = false;
      this.render();
    });
    document.getElementById('btn-profile')!.addEventListener('click', () => {
      this.container.remove();
      this.subScreen = new ProfilePage(this.parent, this.client, () => this.returnFromSub());
    });
    document.getElementById('btn-create-wizard')!.addEventListener('click', () => {
      this.container.remove();
      this.subScreen = new GameCreationWizard(this.parent, this.client, () => this.returnFromSub());
    });
    document.getElementById('btn-refresh')!.addEventListener('click', () => this.loadGames());
    document.getElementById('btn-join')!.addEventListener('click', () => this.joinGame());

    this.loadGames();
  }

  private joinGameId: string = '';

  private async loadGames(): Promise<void> {
    const listEl = document.getElementById('game-list')!;
    try {
      const games = await this.client.listGames();
      if (games.length === 0) {
        listEl.innerHTML = '<p style="color:#555;">No games yet. Create one!</p>';
        return;
      }
      listEl.innerHTML = games.map(g => `
        <div class="game-item" data-id="${g.id}">
          <div>
            <span class="game-name">${g.name}</span>
            <span class="game-status">[${g.status}] Turn ${g.current_turn} | ${g.player_count} players</span>
          </div>
          <div style="display:flex;gap:4px;">
            <button class="btn-join-game" data-id="${g.id}" data-name="${g.name}">Join</button>
            <button class="btn-spectate" data-id="${g.id}" style="font-family:inherit;background:#111;color:#aaa;border:1px solid #555;padding:4px 8px;cursor:pointer;font-size:12px;">👁 Watch</button>
            <button class="btn-admin" data-id="${g.id}" style="font-family:inherit;background:#111;color:#ffff55;border:1px solid #555;padding:4px 8px;cursor:pointer;font-size:12px;">⚙</button>
            <button class="btn-invites" data-id="${g.id}" style="font-family:inherit;background:#111;color:#55ffff;border:1px solid #555;padding:4px 8px;cursor:pointer;font-size:12px;">📨</button>
          </div>
        </div>
      `).join('');

      listEl.querySelectorAll('.btn-join-game').forEach(btn => {
        btn.addEventListener('click', (e) => {
          const el = e.target as HTMLElement;
          this.joinGameId = el.dataset.id!;
          document.getElementById('join-game-name')!.textContent = el.dataset.name ?? '';
          document.getElementById('join-section')!.style.display = 'block';
        });
      });

      // Spectate buttons (T428)
      listEl.querySelectorAll('.btn-spectate').forEach(btn => {
        btn.addEventListener('click', async (e) => {
          const gameId = (e.target as HTMLElement).dataset.id!;
          try {
            await this.client.joinSpectator(gameId);
            // Connect as spectator — for now just go to game with nation -1
            this.onGameStart(gameId, -1);
          } catch (err) {
            alert(`Spectate failed: ${(err as Error).message}`);
          }
        });
      });

      // Admin buttons (T423)
      listEl.querySelectorAll('.btn-admin').forEach(btn => {
        btn.addEventListener('click', (e) => {
          const gameId = (e.target as HTMLElement).dataset.id!;
          this.container.remove();
          this.subScreen = new AdminPanel(this.parent, this.client, gameId, () => this.returnFromSub());
        });
      });

      // Invite buttons (T419)
      listEl.querySelectorAll('.btn-invites').forEach(btn => {
        btn.addEventListener('click', (e) => {
          const gameId = (e.target as HTMLElement).dataset.id!;
          this.container.remove();
          this.subScreen = new InviteManager(this.parent, this.client, gameId, () => this.returnFromSub());
        });
      });
    } catch (e) {
      listEl.innerHTML = `<p class="error">Failed to load games: ${(e as Error).message}</p>`;
    }
  }

  private async joinGame(): Promise<void> {
    const nationName = (document.getElementById('join-nation') as HTMLInputElement).value;
    const leaderName = (document.getElementById('join-leader') as HTMLInputElement).value;
    const race = (document.getElementById('join-race') as HTMLSelectElement).value;
    const classId = parseInt((document.getElementById('join-class') as HTMLSelectElement).value);
    const mark = (document.getElementById('join-mark') as HTMLInputElement).value || '*';
    const errorEl = document.getElementById('join-error')!;

    if (!nationName || !leaderName) {
      errorEl.textContent = 'Enter nation and leader names';
      return;
    }

    try {
      const resp = await this.client.joinGame(this.joinGameId, nationName, leaderName, race, classId, mark);
      // Successfully joined — start the game
      this.container.remove();
      this.onGameStart(resp.game_id, resp.nation_id);
    } catch (e) {
      errorEl.textContent = `Failed: ${(e as Error).message}`;
    }
  }

  destroy(): void {
    if (this.subScreen && 'destroy' in this.subScreen) (this.subScreen as any).destroy();
    if (this.notifBell) { this.notifBell.destroy(); this.notifBell = null; }
    this.container.remove();
  }
}
