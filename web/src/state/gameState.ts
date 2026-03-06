// gameState.ts — Client-side game state cache
// Holds the current game data fetched from the server

import {
  GameInfo, Nation, MapResponse, Sector, ArmyInfo, NavyInfo,
  PublicNationInfo, ScoreEntry, NewsEntry, ChatMessageData,
  DisplayMode, HighlightMode,
} from '../types';

export interface GameState {
  // Auth
  token: string | null;
  userId: string | null;
  username: string | null;

  // Current game
  gameId: string | null;
  gameInfo: GameInfo | null;
  nationId: number | null;

  // Game data
  nation: Nation | null;
  mapData: MapResponse | null;
  armies: ArmyInfo[];
  navies: NavyInfo[];
  publicNations: PublicNationInfo[];
  scores: ScoreEntry[];
  news: NewsEntry[];

  // Occupied grid (nation ids of armies at each position)
  occupied: number[][];

  // UI state
  displayMode: DisplayMode;
  highlightMode: HighlightMode;
  cursorX: number;
  cursorY: number;
  xOffset: number;
  yOffset: number;
  selectedArmy: number;  // index into armies array, -1 = none
  selectedNavy: number;
  armyOrNavy: 'army' | 'navy';
  notifications: string[];
  waitingForPlayers: boolean;
  isDone: boolean;

  // Chat state (Phase 5)
  chatMessages: Record<string, ChatMessageData[]>; // channel -> messages
  chatChannel: string;           // current active channel
  chatChannels: string[];        // available channels
  chatInput: string;             // current input text
  chatOpen: boolean;             // panel visible?
  chatUnread: Record<string, number>; // channel -> unread count
  onlineNations: Set<number>;    // nation IDs currently connected

  // Connection
  connected: boolean;

  // Render theme
  renderMode: 'classic' | 'enhanced';  // kept for compat
  themeId: string;
}

export function createInitialState(): GameState {
  return {
    token: localStorage.getItem('conquer_token'),
    userId: localStorage.getItem('conquer_user_id'),
    username: localStorage.getItem('conquer_username'),
    gameId: null,
    gameInfo: null,
    nationId: null,
    nation: null,
    mapData: null,
    armies: [],
    navies: [],
    publicNations: [],
    scores: [],
    news: [],
    occupied: [],
    displayMode: DisplayMode.Designation,
    highlightMode: HighlightMode.Own,
    cursorX: 0,
    cursorY: 0,
    xOffset: 0,
    yOffset: 0,
    selectedArmy: -1,
    selectedNavy: -1,
    armyOrNavy: 'army',
    notifications: [],
    waitingForPlayers: false,
    isDone: false,
    chatMessages: { public: [] },
    chatChannel: 'public',
    chatChannels: ['public'],
    chatInput: '',
    chatOpen: false,
    chatUnread: {},
    onlineNations: new Set(),
    connected: false,
    renderMode: 'classic',
    themeId: 'classic-green',
  };
}

/** Build occupied grid from nation data */
export function buildOccupied(state: GameState): void {
  if (!state.mapData) return;
  const mx = state.mapData.map_x;
  const my = state.mapData.map_y;
  state.occupied = [];
  for (let x = 0; x < mx; x++) {
    state.occupied[x] = new Array(my).fill(0);
  }
  // Mark own armies
  for (const army of state.armies) {
    if (army.soldiers > 0 && army.x < mx && army.y < my) {
      state.occupied[army.x][army.y] = state.nationId ?? 1;
    }
  }
}

/** Get sector at absolute coords, or null if fog */
export function getSector(state: GameState, x: number, y: number): Sector | null {
  if (!state.mapData) return null;
  if (x < 0 || y < 0 || x >= state.mapData.map_x || y >= state.mapData.map_y) return null;
  return state.mapData.sectors[x]?.[y] ?? null;
}
