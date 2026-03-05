// types.ts — Shared type definitions matching Rust server API responses

// ============================================================
// Game Constants (matching conquer-core/src/constants.rs)
// ============================================================

export const NTOTAL = 35;
export const MAXARM = 50;
export const MAXNAVY = 10;
export const MAPX = 32;
export const MAPY = 32;
export const LANDSEE = 2;
export const NAVYSEE = 1;
export const ARMYSEE = 2;

// ============================================================
// Enums (matching conquer-core/src/enums.rs)
// ============================================================

// Designation chars: "tcmfx$!&sC?lb+*g=u-P"
export const DES_CHARS = 'tcmfx$!&sC?lb+*g=u-P';
// Vegetation chars: "vdtblgwfjsi~"
export const VEG_CHARS = 'vdtblgwfjsi~';
// Altitude chars: "~#^%-"
export const ELE_CHARS = '~#^%-';

export enum DisplayMode {
  Vegetation = 1,
  Designation = 2,
  Contour = 3,
  Food = 4,
  Nation = 5,
  Race = 6,
  Move = 7,
  Defense = 8,
  People = 9,
  Gold = 10,
  Metal = 11,
  Items = 12,
}

export enum HighlightMode {
  Own = 0,
  Army = 1,
  None = 2,
  YourArmy = 3,
  Move = 4,
  Good = 5,
}

export const RACE_NAMES: Record<string, string> = {
  '-': 'GOD', 'O': 'ORC', 'E': 'ELF', 'D': 'DWARF',
  'L': 'LIZARD', 'H': 'HUMAN', 'P': 'PIRATE',
  'S': 'SAVAGE', 'N': 'NOMAD', '?': 'UNKNOWN',
};

export const CLASS_NAMES: string[] = [
  'monster', 'king', 'emperor', 'wizard', 'priest',
  'pirate', 'trader', 'warlord', 'demon', 'dragon', 'shadow',
];

export const DESIGNATION_NAMES: string[] = [
  'TOWN', 'CITY', 'MINE', 'FARM', 'DEVASTATED', 'GOLDMINE',
  'FORT', 'RUIN', 'STOCKADE', 'CAPITOL', 'SPECIAL', 'LUMBERYD',
  'BLKSMITH', 'ROAD', 'MILL', 'GRANARY', 'CHURCH', 'UNIVERSITY',
  'NODESIG', 'BASE CAMP',
];

export const VEGETATION_NAMES: string[] = [
  'VOLCANO', 'DESERT', 'TUNDRA', 'BARREN', 'LT VEG', 'GOOD',
  'WOOD', 'FOREST', 'JUNGLE', 'SWAMP', 'ICE', 'NONE',
];

export const ALTITUDE_NAMES: string[] = [
  'WATER', 'PEAK', 'MOUNTAIN', 'HILL', 'FLAT',
];

export const ARMY_STATUS_NAMES: string[] = [
  '', 'MARCH', 'SCOUT', 'GARRISON', 'TRADED', 'MILITIA',
  'FLYING', 'DEFEND', 'MAG_DEF', 'ATTACK', 'MAG_ATT',
  'GENERAL', 'SORTIE', 'SIEGE', 'BESIEGED', 'ON_BOARD', 'RULE',
];

export const DIPLO_NAMES: string[] = [
  'UNMET', 'TREATY', 'ALLIED', 'FRIENDLY', 'NEUTRAL',
  'HOSTILE', 'WAR', 'JIHAD',
];

export const SEASON_NAMES: string[] = ['Winter', 'Spring', 'Summer', 'Fall'];

// ============================================================
// API Response Types
// ============================================================

export interface Sector {
  designation: number;
  altitude: number;
  vegetation: number;
  owner: number;
  people: number;
  initial_people: number;
  jewels: number;
  fortress: number;
  metal: number;
  trade_good: number;
}

export interface ArmyInfo {
  index: number;
  unit_type: number;
  x: number;
  y: number;
  movement: number;
  soldiers: number;
  status: number;
}

export interface NavyInfo {
  index: number;
  warships: number;
  merchant: number;
  galleys: number;
  x: number;
  y: number;
  movement: number;
  crew: number;
  people: number;
}

export interface Nation {
  name: string;
  leader: string;
  race: string;
  mark: string;
  cap_x: number;
  cap_y: number;
  active: number;
  score: number;
  treasury_gold: number;
  jewels: number;
  total_mil: number;
  total_civ: number;
  metals: number;
  total_food: number;
  class: number;
  attack_plus: number;
  defense_plus: number;
  spell_points: number;
  total_sectors: number;
  total_ships: number;
  tax_rate: number;
  armies: ArmyInfo[];
  navies: NavyInfo[];
  diplomacy: number[];
  popularity: number;
  terror: number;
  reputation: number;
  powers: number;
  charity: number;
}

export interface PublicNationInfo {
  nation_id: number;
  name: string;
  race: string;
  class: number;
  mark: string;
  active: number;
  score: number;
}

export interface GameSettings {
  map_x: number;
  map_y: number;
  max_players: number;
  npc_count: number;
  monster_count: number;
  seed: number;
  turn_timer_hours: number | null;
  auto_advance: boolean;
}

export interface GameInfo {
  id: string;
  name: string;
  status: string;
  settings: GameSettings;
  created_at: string;
  updated_at: string;
  current_turn: number;
  player_count: number;
}

export interface MapResponse {
  map_x: number;
  map_y: number;
  sectors: (Sector | null)[][];
}

export interface ScoreEntry {
  nation_id: number;
  name: string;
  race: string;
  score: number;
}

export interface NewsEntry {
  id: string;
  turn: number;
  content: string;
  created_at: string;
}

export interface AuthResponse {
  token: string;
  user_id: string;
  username: string;
}

export interface JoinGameResponse {
  nation_id: number;
  game_id: string;
  nation_name: string;
}

// ============================================================
// WebSocket Message Types (matching ws.rs)
// ============================================================

export type ServerMessage =
  | { type: 'map_update'; data: { sectors?: unknown } }
  | { type: 'nation_update'; data: { nation_id: number; data?: unknown } }
  | { type: 'army_update'; data: { nation_id: number; army_id: number; data?: unknown } }
  | { type: 'news'; data: { turn: number; messages: string[] } }
  | { type: 'turn_start'; data: { turn: number; season: string } }
  | { type: 'turn_end'; data: { old_turn: number; new_turn: number } }
  | { type: 'player_joined'; data: { nation_id: number; nation_name: string; race: string } }
  | { type: 'player_done'; data: { nation_id: number; nation_name: string } }
  | { type: 'chat_message'; data: { sender_nation_id: number | null; channel: string; content: string; timestamp: string } }
  | { type: 'system_message'; data: { content: string } }
  | { type: 'pong'; data: null }
  | { type: 'error'; data: { message: string } };

export type ClientMessage =
  | { type: 'action'; data: { action: unknown } }
  | { type: 'chat_send'; data: { channel: string; content: string } }
  | { type: 'ping'; data: null };

// ============================================================
// Helper functions
// ============================================================

export function desChar(index: number): string {
  return DES_CHARS[index] ?? '?';
}

export function vegChar(index: number): string {
  return VEG_CHARS[index] ?? '?';
}

export function eleChar(index: number): string {
  return ELE_CHARS[index] ?? '?';
}

export function seasonFromTurn(turn: number): string {
  return SEASON_NAMES[turn % 4] ?? 'Unknown';
}

export function yearFromTurn(turn: number): number {
  return Math.floor((turn + 3) / 4);
}
