// client.ts — GameClient: REST API + WebSocket connection manager
// T373-T380

import {
  AuthResponse, GameInfo, MapResponse, Nation, ArmyInfo, NavyInfo,
  PublicNationInfo, ScoreEntry, NewsEntry, JoinGameResponse,
  ServerMessage, ClientMessage, ChatMessageData,
  UserProfile, GameHistoryEntry, Notification, NotificationPreferences,
  AdminPlayerInfo, TurnSnapshotInfo, ServerStats, InviteInfo, GameSettings,
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

  async endTurn(gameId: string): Promise<{ status: string; new_turn?: number }> {
    return this.request<{ status: string; new_turn?: number }>('POST', `/games/${gameId}/end-turn`);
  }

  async runTurn(gameId: string): Promise<unknown> {
    return this.request<unknown>('POST', `/games/${gameId}/run-turn`);
  }

  // ============ Chat (T392) ============

  async getChatHistory(gameId: string, channel: string = 'public', before?: string, limit: number = 50): Promise<{ channel: string; messages: ChatMessageData[] }> {
    const params = new URLSearchParams({ channel, limit: String(limit) });
    if (before) params.set('before', before);
    return this.request<{ channel: string; messages: ChatMessageData[] }>('GET', `/games/${gameId}/chat?${params}`);
  }

  async getChatChannels(gameId: string): Promise<string[]> {
    return this.request<string[]>('GET', `/games/${gameId}/chat/channels`);
  }

  async getPresence(gameId: string): Promise<number[]> {
    return this.request<number[]>('GET', `/games/${gameId}/presence`);
  }

  sendChatMessage(channel: string, content: string): void {
    this.sendWs({ type: 'chat_send', data: { channel, content } });
  }

  requestChatHistory(channel: string, before?: string, limit: number = 50): void {
    this.sendWs({ type: 'chat_history_request', data: { channel, before, limit } });
  }

  // ============ User Profile (T409-T411) ============

  async getProfile(): Promise<UserProfile> {
    return this.request<UserProfile>('GET', '/users/me');
  }

  async updateProfile(data: { display_name?: string; email?: string }): Promise<unknown> {
    return this.request('PUT', '/users/me', data);
  }

  async changePassword(oldPassword: string, newPassword: string): Promise<unknown> {
    return this.request('PUT', '/users/me/password', { old_password: oldPassword, new_password: newPassword });
  }

  async getGameHistory(): Promise<GameHistoryEntry[]> {
    return this.request<GameHistoryEntry[]>('GET', '/users/me/history');
  }

  // ============ Game Settings (T415-T418) ============

  async updateGameSettings(gameId: string, settings: Partial<GameSettings>): Promise<GameInfo> {
    return this.request<GameInfo>('PUT', `/games/${gameId}/settings`, settings);
  }

  // ============ Invites (T419-T422) ============

  async createInvite(gameId: string, maxUses?: number, expiresHours?: number): Promise<InviteInfo> {
    return this.request<InviteInfo>('POST', `/games/${gameId}/invites`, { max_uses: maxUses, expires_hours: expiresHours });
  }

  async listInvites(gameId: string): Promise<InviteInfo[]> {
    return this.request<InviteInfo[]>('GET', `/games/${gameId}/invites`);
  }

  async revokeInvite(gameId: string, inviteId: string): Promise<unknown> {
    return this.request('DELETE', `/games/${gameId}/invites/${inviteId}`);
  }

  async getInvite(code: string): Promise<InviteInfo> {
    return this.request<InviteInfo>('GET', `/invites/${code}`);
  }

  async acceptInvite(code: string, nationName: string, leaderName: string, race: string, classId: number, mark: string): Promise<JoinGameResponse> {
    return this.request<JoinGameResponse>('POST', `/invites/${code}/accept`, {
      nation_name: nationName, leader_name: leaderName, race, class: classId, mark,
    });
  }

  // ============ Admin (T423-T427) ============

  async adminListPlayers(gameId: string): Promise<AdminPlayerInfo[]> {
    return this.request<AdminPlayerInfo[]>('GET', `/games/${gameId}/admin/players`);
  }

  async adminKickPlayer(gameId: string, nationId: number): Promise<unknown> {
    return this.request('POST', `/games/${gameId}/admin/kick`, { nation_id: nationId });
  }

  async adminSetStatus(gameId: string, status: string): Promise<GameInfo> {
    return this.request<GameInfo>('POST', `/games/${gameId}/admin/status`, { status });
  }

  async adminAdvanceTurn(gameId: string): Promise<unknown> {
    return this.request('POST', `/games/${gameId}/admin/advance-turn`);
  }

  async adminListSnapshots(gameId: string): Promise<TurnSnapshotInfo[]> {
    return this.request<TurnSnapshotInfo[]>('GET', `/games/${gameId}/admin/snapshots`);
  }

  async adminRollback(gameId: string, targetTurn: number): Promise<unknown> {
    return this.request('POST', `/games/${gameId}/admin/rollback`, { target_turn: targetTurn });
  }

  async adminDeleteGame(gameId: string): Promise<unknown> {
    return this.request('DELETE', `/games/${gameId}`);
  }

  async getServerStats(): Promise<ServerStats> {
    return this.request<ServerStats>('GET', '/admin/stats');
  }

  // ============ Spectator (T428-T431) ============

  async joinSpectator(gameId: string): Promise<unknown> {
    return this.request('POST', `/games/${gameId}/spectate`);
  }

  async leaveSpectator(gameId: string): Promise<unknown> {
    return this.request('DELETE', `/games/${gameId}/spectate`);
  }

  async getSpectatorMap(gameId: string): Promise<MapResponse> {
    return this.request<MapResponse>('GET', `/games/${gameId}/spectate/map`);
  }

  // ============ Notifications (T432-T434) ============

  async getNotifications(unreadOnly: boolean = false): Promise<Notification[]> {
    const qs = unreadOnly ? '?unread_only=true' : '';
    return this.request<Notification[]>('GET', `/notifications${qs}`);
  }

  async markNotificationRead(id: string): Promise<unknown> {
    return this.request('POST', `/notifications/${id}/read`);
  }

  async markAllNotificationsRead(): Promise<unknown> {
    return this.request('POST', '/notifications/read-all');
  }

  async getNotificationPreferences(): Promise<NotificationPreferences> {
    return this.request<NotificationPreferences>('GET', '/notifications/preferences');
  }

  async setNotificationPreferences(prefs: NotificationPreferences): Promise<NotificationPreferences> {
    return this.request<NotificationPreferences>('PUT', '/notifications/preferences', prefs);
  }

  // ============ Game Browser (T422) ============

  async listPublicGames(): Promise<GameInfo[]> {
    return this.request<GameInfo[]>('GET', '/games/public');
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
