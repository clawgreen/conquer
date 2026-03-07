// movementCost.ts — Client-side movement cost calculation
// Mirrors conquer-engine/src/movement.rs update_move_costs() and the C misc.c updmove()

import { Sector } from '../types';

// Altitude enum values (matching Rust/C)
export const ALT_WATER = 0;
export const ALT_PEAK = 1;
export const ALT_MOUNTAIN = 2;
export const ALT_HILL = 3;
export const ALT_CLEAR = 4;

// Vegetation enum values
export const VEG_VOLCANO = 0;
export const VEG_DESERT = 1;
export const VEG_TUNDRA = 2;
export const VEG_ICE = 10;

// Designation: Road = 13
export const DESIG_ROAD = 13;

// Army statuses (matching Rust ArmyStatus enum)
export const STATUS_MARCH = 1;
export const STATUS_SCOUT = 2;
export const STATUS_GARRISON = 3;
export const STATUS_TRADED = 4;
export const STATUS_MILITIA = 5;
export const STATUS_FLIGHT = 6;
export const STATUS_DEFEND = 7;
export const STATUS_ONBOARD = 15;
export const STATUS_RULE = 16;

// Cost tables by race, indexed by altitude/vegetation enum value
// '/' means impassable (-1)
// These match conquer-core/src/tables.rs exactly
const PARSE = (s: string): number[] =>
  [...s].map(c => c === '/' ? -1 : c.charCodeAt(0) - 48);

const H_ELE_COST = PARSE('//521');
const H_VEG_COST = PARSE('63210001332//');
const O_ELE_COST = PARSE('//222');
const O_VEG_COST = PARSE('43100022527//');
const E_ELE_COST = PARSE('//631');
const E_VEG_COST = PARSE('86221000027//');
const D_ELE_COST = PARSE('//311');
const D_VEG_COST = PARSE('47100013577//');
const F_ELE_COST = PARSE('//100');
const F_VEG_COST = PARSE('410000001000/');

function getEleCost(race: string): number[] {
  switch (race) {
    case 'O': return O_ELE_COST;
    case 'E': return E_ELE_COST;
    case 'D': return D_ELE_COST;
    default:  return H_ELE_COST;
  }
}

function getVegCost(race: string): number[] {
  switch (race) {
    case 'O': return O_VEG_COST;
    case 'E': return E_VEG_COST;
    case 'D': return D_VEG_COST;
    default:  return H_VEG_COST;
  }
}

/**
 * Calculate movement cost for a sector.
 * Returns -1 for water (coast), -3 for deep water, -2 for impassable land, or positive cost.
 */
export function sectorMoveCost(
  sector: Sector,
  race: string,
  sectors: (Sector | null)[][],
  sx: number, sy: number,
  mapX: number, mapY: number,
): number {
  const alt = sector.altitude;
  const veg = sector.vegetation;

  // Water
  if (alt === ALT_WATER) {
    // Check if coastal (adjacent to land)
    for (let dx = -1; dx <= 1; dx++) {
      for (let dy = -1; dy <= 1; dy++) {
        const nx = sx + dx;
        const ny = sy + dy;
        if (nx >= 0 && nx < mapX && ny >= 0 && ny < mapY) {
          const adj = sectors[nx]?.[ny];
          if (adj && adj.altitude !== ALT_WATER) return -1; // coastal
        }
      }
    }
    return -3; // deep water
  }

  const eleCosts = getEleCost(race);
  const vegCosts = getVegCost(race);

  const eleC = alt < eleCosts.length ? eleCosts[alt] : -1;
  const vegC = veg < vegCosts.length ? vegCosts[veg] : -1;

  if (eleC < 0 || vegC < 0) return -2; // impassable land

  let cost = eleC + vegC;

  // Road reduces cost
  if (sector.designation === DESIG_ROAD && cost > 1) {
    cost = 1;
  }

  // Minimum cost of 1 for habitable land
  if (cost < 1) cost = 1;

  return cost;
}

/**
 * Calculate flight cost for a sector.
 * Matches C flightcost() / Rust utils::flightcost().
 */
export function flightMoveCost(sector: Sector): number {
  const alt = sector.altitude;
  const veg = sector.vegetation;
  const eleC = alt < F_ELE_COST.length ? F_ELE_COST[alt] : -1;
  const vegC = veg < F_VEG_COST.length ? F_VEG_COST[veg] : -1;
  if (eleC < 0 || vegC < 0) return -1;
  return eleC + vegC;
}

/**
 * Check if an army can move at all (status check).
 */
export function canArmyMove(status: number): boolean {
  return status !== STATUS_GARRISON
    && status !== STATUS_RULE
    && status !== STATUS_MILITIA
    && status !== STATUS_ONBOARD
    && status !== STATUS_TRADED;
}

/**
 * Compute effective movement cost for an army moving to a given sector.
 * Returns -1 if impassable, or the cost in movement points.
 */
export function effectiveMoveCost(
  sector: Sector,
  armyStatus: number,
  race: string,
  sectors: (Sector | null)[][],
  tx: number, ty: number,
  mapX: number, mapY: number,
): number {
  if (armyStatus === STATUS_FLIGHT) {
    const fc = flightMoveCost(sector);
    const mc = sectorMoveCost(sector, race, sectors, tx, ty, mapX, mapY);
    // Use land cost if cheaper and passable
    if (mc > 0 && mc < fc) return mc;
    return fc;
  }
  if (armyStatus === STATUS_SCOUT) {
    const mc = sectorMoveCost(sector, race, sectors, tx, ty, mapX, mapY);
    if (mc < 0) return -1; // impassable even for scouts
    return Math.min(1, mc); // scouts always cost 1
  }
  return sectorMoveCost(sector, race, sectors, tx, ty, mapX, mapY);
}

/**
 * Compute reachable tiles from (cx, cy) with given movement points.
 * Returns a Map from "x,y" to remaining movement after reaching that tile.
 * Uses BFS/Dijkstra-style flood fill.
 */
export function computeReachable(
  cx: number, cy: number,
  movePoints: number,
  armyStatus: number,
  race: string,
  sectors: (Sector | null)[][],
  mapX: number, mapY: number,
): Map<string, number> {
  const reachable = new Map<string, number>();
  if (movePoints <= 0) return reachable;

  // BFS with cost tracking (simple Dijkstra with priority queue)
  const remaining = new Map<string, number>();
  const key = (x: number, y: number) => `${x},${y}`;
  remaining.set(key(cx, cy), movePoints);

  // Use a simple queue (tiles to explore), sorted by remaining desc
  const queue: [number, number][] = [[cx, cy]];

  while (queue.length > 0) {
    const [x, y] = queue.shift()!;
    const rem = remaining.get(key(x, y))!;

    for (let dx = -1; dx <= 1; dx++) {
      for (let dy = -1; dy <= 1; dy++) {
        if (dx === 0 && dy === 0) continue;
        const nx = x + dx;
        const ny = y + dy;
        if (nx < 0 || nx >= mapX || ny < 0 || ny >= mapY) continue;
        const sector = sectors[nx]?.[ny];
        if (!sector) continue;

        const cost = effectiveMoveCost(sector, armyStatus, race, sectors, nx, ny, mapX, mapY);
        if (cost < 0) continue; // impassable

        const newRem = rem - cost;
        if (newRem < 0) continue; // not enough movement

        const k = key(nx, ny);
        if (!remaining.has(k) || remaining.get(k)! < newRem) {
          remaining.set(k, newRem);
          reachable.set(k, newRem);
          queue.push([nx, ny]);
        }
      }
    }
  }

  return reachable;
}
