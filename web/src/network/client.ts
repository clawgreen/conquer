// client.ts — GameClient: REST API + WebSocket connection manager
// T373-T380

import {
  AuthResponse, GameInfo, MapResponse, Nation, ArmyInfo, NavyInfo,
  PublicNationInfo, ScoreEntry, NewsEntry, JoinGameResponse,
  ServerMessage, ClientMessage,
} from '../types';

const API_BASE = '/api';

export class GameClient {
  private token: string | null = null;
  private ws: WebSocket | null = null;
  private reconnectTimer: number = 0;
  private pingTimer: number = 0;
  private messageHandlers: ((msg: ServerMessage) => void)[] = [];
  private disconnectHandlers: (() => void)[] = [];
  private connectHandlers: (() => void)[] = [];

  setToken(token: string): void {
    this.token = token;
    localStorage.setItem('conquer_token', token);
  }

  getToken(): string | null {
    return this.token;
  }

  private async request<T>(method: string, path: string, body?: unknown): Promise<T> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    if (this.token) {
      headers['Authorization'] = `Bearer ${this.token}`;
    }
    const resp = await fetch(`${API_BASE}${path}`, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
    });
    if (!resp.ok) {
      const text = await resp.text();
      throw new Error(`API ${method} ${path}: ${resp.status} ${text}`);
    }
    return resp.json() as Promise<T>;
  }

  // ============ Auth ============

  async register(username: string, email: string, password: string): Promise<AuthResponse> {
    const resp = await this.request<AuthResponse>('POST', '/auth/register', { username, email, password });
    this.setToken(resp.token);
    localStorage.setItem('conquer_user_id', resp.user_id);
    localStorage.setItem('conquer_username', resp.username);
    return resp;
  }

  async login(username: string, password: string): Promise<AuthResponse> {
    const resp = await this.request<AuthResponse>('POST', '/auth/login', { username, password });
    this.setToken(resp.token);
    localStorage.setItem('conquer_user_id', resp.user_id);
    localStorage.setItem('conquer_username', resp.username);
    return resp;
  }

  // ============ Games ============

  async createGame(name: string, settings?: Partial<unknown>): Promise<GameInfo> {
    return this.request<GameInfo>('POST', '/games', { name, settings });
  }

  async listGames(status?: string): Promise<GameInfo[]> {
    const qs = status ? `?status=${status}` : '';
    return this.request<GameInfo[]>('GET', `/games${qs}`);
  }

  async getGame(gameId: string): Promise<GameInfo> {
    return this.request<GameInfo>('GET', `/games/${gameId}`);
  }

  async joinGame(gameId: string, nationName: string, leaderName: string, race: string, classId: number, mark: string): Promise<JoinGameResponse> {
    return this.request<JoinGameResponse>('POST', `/games/${gameId}/join`, {
      nation_name: nationName,
      leader_name: leaderName,
      race,
      class: classId,
      mark,
    });
  }

  // ============ Game State ============

  async getMap(gameId: string): Promise<MapResponse> {
    return this.request<MapResponse>('GET', `/games/${gameId}/map`);
  }

  async getNation(gameId: string): Promise<Nation> {
    return this.request<Nation>('GET', `/games/${gameId}/nation`);
  }

  async getNations(gameId: string): Promise<PublicNationInfo[]> {
    return this.request<PublicNationInfo[]>('GET', `/games/${gameId}/nations`);
  }

  async getArmies(gameId: string): Promise<ArmyInfo[]> {
    return this.request<ArmyInfo[]>('GET', `/games/${gameId}/armies`);
  }

  async getNavies(gameId: string): Promise<NavyInfo[]> {
    return this.request<NavyInfo[]>('GET', `/games/${gameId}/navies`);
  }

  async getScores(gameId: string): Promise<ScoreEntry[]> {
    return this.request<ScoreEntry[]>('GET', `/games/${gameId}/scores`);
  }

  async getNews(gameId: string): Promise<NewsEntry[]> {
    return this.request<NewsEntry[]>('GET', `/games/${gameId}/news`);
  }

  // ============ Actions ============

  async submitActions(gameId: string, actions: unknown[]): Promise<unknown[]> {
    return this.request<unknown[]>('POST', `/games/${gameId}/actions`, { actions });
  }

  async getActions(gameId: string): Promise<unknown[]> {
    return this.request<unknown[]>('GET', `/games/${gameId}/actions`);
  }

  async endTurn(gameId: string): Promise<unknown> {
    return this.request<unknown>('POST', `/games/${gameId}/end-turn`);
  }

  async runTurn(gameId: string): Promise<unknown> {
    return this.request<unknown>('POST', `/games/${gameId}/run-turn`);
  }

  // ============ WebSocket ============

  onMessage(handler: (msg: ServerMessage) => void): void {
    this.messageHandlers.push(handler);
  }

  onDisconnect(handler: () => void): void {
    this.disconnectHandlers.push(handler);
  }

  onConnect(handler: () => void): void {
    this.connectHandlers.push(handler);
  }

  connectWebSocket(gameId: string): void {
    if (this.ws) {
      this.ws.close();
    }

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}${API_BASE}/games/${gameId}/ws?token=${this.token}`;

    this.ws = new WebSocket(wsUrl);

    this.ws.onopen = () => {
      console.log('[WS] Connected');
      this.connectHandlers.forEach(h => h());
      // Start ping heartbeat
      this.pingTimer = window.setInterval(() => {
        this.sendWs({ type: 'ping', data: null });
      }, 30000);
    };

    this.ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data) as ServerMessage;
        this.messageHandlers.forEach(h => h(msg));
      } catch (e) {
        console.warn('[WS] Bad message:', event.data, e);
      }
    };

    this.ws.onclose = () => {
      console.log('[WS] Disconnected');
      this.disconnectHandlers.forEach(h => h());
      if (this.pingTimer) clearInterval(this.pingTimer);
      // Auto-reconnect after 3s
      this.reconnectTimer = window.setTimeout(() => {
        console.log('[WS] Reconnecting...');
        this.connectWebSocket(gameId);
      }, 3000);
    };

    this.ws.onerror = (e) => {
      console.error('[WS] Error:', e);
    };
  }

  private sendWs(msg: ClientMessage): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(msg));
    }
  }

  disconnectWebSocket(): void {
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    if (this.pingTimer) clearInterval(this.pingTimer);
    if (this.ws) {
      this.ws.onclose = null; // prevent auto-reconnect
      this.ws.close();
      this.ws = null;
    }
  }
}
