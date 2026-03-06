// compositor.ts — Multi-layer tile compositor
// Draws terrain → vegetation → designation → resources → ownership → units → cursor
// Each layer can be toggled. Supports both emoji and image tile types.

import { GameState, getSector } from '../state/gameState';
import { Sector, DisplayMode } from '../types';
import { TileSet, TileDef, getScaledCellSize, getCachedImage } from './tilesets';

/** Which layers are currently enabled for rendering */
export interface LayerConfig {
  terrain: boolean;
  vegetation: boolean;
  designation: boolean;
  resources: boolean;
  ownership: boolean;
  units: boolean;
  cursor: boolean;
}

export const DEFAULT_LAYERS: LayerConfig = {
  terrain: true,
  vegetation: true,
  designation: true,
  resources: false,
  ownership: true,
  units: true,
  cursor: true,
};

/** Per-mode default layer visibility */
export function layersForMode(mode: DisplayMode): LayerConfig {
  switch (mode) {
    case DisplayMode.Vegetation:
      return { ...DEFAULT_LAYERS, designation: false, resources: false, ownership: false };
    case DisplayMode.Designation:
      return { ...DEFAULT_LAYERS, resources: false };
    case DisplayMode.Contour:
      return { ...DEFAULT_LAYERS, vegetation: false, designation: false, resources: false, ownership: false };
    case DisplayMode.Food:
    case DisplayMode.Move:
    case DisplayMode.Defense:
    case DisplayMode.People:
    case DisplayMode.Gold:
    case DisplayMode.Metal:
    case DisplayMode.Items:
      // Data overlay modes — show terrain as base, units on top
      return { terrain: true, vegetation: false, designation: false, resources: false, ownership: false, units: true, cursor: true };
    case DisplayMode.Nation:
      return { terrain: true, vegetation: false, designation: false, resources: false, ownership: true, units: true, cursor: true };
    case DisplayMode.Race:
      return { terrain: true, vegetation: false, designation: false, resources: false, ownership: true, units: true, cursor: true };
    default:
      return DEFAULT_LAYERS;
  }
}

/** Draw a single tile (emoji or image) at position */
function drawTile(
  ctx: CanvasRenderingContext2D,
  tile: TileDef,
  px: number,
  py: number,
  cw: number,
  ch: number,
): void {
  if (tile.bg) {
    ctx.fillStyle = tile.bg;
    ctx.fillRect(px, py, cw, ch);
  }

  if (tile.type === 'emoji') {
    ctx.font = `${Math.max(10, ch - 4)}px serif`;
    ctx.textBaseline = 'middle';
    ctx.textAlign = 'center';
    ctx.fillText(tile.value, px + cw / 2, py + ch / 2);
  } else if (tile.type === 'image') {
    const img = getCachedImage(tile.value);
    if (img) {
      // Pixel art: disable smoothing for crisp nearest-neighbor scaling
      ctx.imageSmoothingEnabled = false;
      ctx.drawImage(img, px, py, cw, ch);
    }
  } else if (tile.type === 'char') {
    ctx.font = `${Math.max(10, ch - 2)}px "Courier New", monospace`;
    ctx.textBaseline = 'middle';
    ctx.textAlign = 'center';
    ctx.fillStyle = tile.fg ?? '#aaa';
    ctx.fillText(tile.value, px + cw / 2, py + ch / 2);
  }
}

/** Composite render a single sector with multiple layers */
export function compositeRenderSector(
  ctx: CanvasRenderingContext2D,
  ts: TileSet,
  sector: Sector,
  state: GameState,
  absX: number,
  absY: number,
  px: number,
  py: number,
  cw: number,
  ch: number,
  layers: LayerConfig,
): void {
  const nationId = state.nationId ?? 0;

  // Layer 0: Terrain (elevation base)
  if (layers.terrain) {
    const terrainTile = ts.elevation[sector.altitude];
    if (terrainTile) drawTile(ctx, terrainTile, px, py, cw, ch);
  }

  // Layer 1: Vegetation (skip for water/peak)
  if (layers.vegetation && sector.altitude >= 2 && sector.vegetation < 11) {
    const vegTile = ts.vegetation[sector.vegetation];
    if (vegTile) {
      if (ts.tileType === 'image') {
        // Image tilesets: vegetation IS the base tile, draw full-cell opaque
        drawTile(ctx, vegTile, px, py, cw, ch);
      } else {
        // Emoji/char: semi-transparent overlay
        ctx.globalAlpha = 0.85;
        drawTile(ctx, vegTile, px, py, cw, ch);
        ctx.globalAlpha = 1.0;
      }
    }
  }

  // Layer 2: Designation (only on owned sectors)
  if (layers.designation && sector.owner > 0 && sector.designation !== 18) {
    const desTile = ts.designation[sector.designation];
    if (desTile) {
      if (ts.tileType === 'image') {
        // Image tilesets: designation tile is a full scene, draw full-cell
        drawTile(ctx, desTile, px, py, cw, ch);
      } else {
        // Emoji/char: draw smaller in center so terrain/veg shows around edges
        const inset = Math.max(2, cw * 0.15);
        const dw = cw - inset * 2;
        const dh = ch - inset * 2;
        drawTile(ctx, desTile, px + inset, py + inset, dw, dh);
      }
    }
  }

  // Layer 3: Resources (small indicators)
  if (layers.resources) {
    const hasJewels = sector.jewels > 0;
    const hasMetal = sector.metal > 0;
    if (hasJewels || hasMetal) {
      ctx.font = `${Math.max(8, ch * 0.3)}px serif`;
      ctx.textBaseline = 'top';
      ctx.textAlign = 'left';
      if (hasJewels) {
        ctx.fillText('💎', px + 1, py + 1);
      }
      if (hasMetal) {
        ctx.fillText('⛏️', px + cw * 0.5, py + 1);
      }
    }
  }

  // Layer 4: Ownership (colored border)
  if (layers.ownership && sector.owner > 0) {
    const isOwn = sector.owner === nationId;
    ctx.strokeStyle = isOwn ? 'rgba(85,255,85,0.5)' : `hsla(${(sector.owner * 37) % 360}, 70%, 50%, 0.5)`;
    ctx.lineWidth = 1;
    ctx.strokeRect(px + 0.5, py + 0.5, cw - 1, ch - 1);
  }

  // Layer 5: Units (armies/navies)
  if (layers.units) {
    const armiesHere = state.armies.filter(a => a.soldiers > 0 && a.x === absX && a.y === absY);
    if (armiesHere.length > 0) {
      // Draw army tile
      drawTile(ctx, ts.army, px, py + ch * 0.3, cw, ch * 0.7);
      // Army count badge if multiple
      if (armiesHere.length > 1) {
        ctx.fillStyle = 'rgba(0,0,0,0.7)';
        const badgeR = Math.max(6, ch * 0.2);
        ctx.beginPath();
        ctx.arc(px + cw - badgeR, py + badgeR, badgeR, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillStyle = '#ff5555';
        ctx.font = `bold ${Math.max(8, badgeR)}px sans-serif`;
        ctx.textBaseline = 'middle';
        ctx.textAlign = 'center';
        ctx.fillText(String(armiesHere.length), px + cw - badgeR, py + badgeR);
      }
    }
    // Navy — only on water
    if (sector.altitude === 0) {
      // Check for navies (would need navy data in state)
      // For now, just show navy tile if applicable
    }
  }
}

/**
 * Full composited map render.
 * Draws all visible sectors with multi-layer compositing.
 */
export function renderCompositedMap(
  ctx: CanvasRenderingContext2D,
  state: GameState,
  ts: TileSet,
  fontSize: number,
  canvasWidth: number,
  canvasHeight: number,
  layers?: LayerConfig,
): void {
  const { cw, ch } = getScaledCellSize(ts, fontSize);
  const activeLayers = layers ?? layersForMode(state.displayMode);

  // Pixel art: ensure nearest-neighbor scaling for crisp tiles
  if (ts.tileType === 'image') {
    ctx.imageSmoothingEnabled = false;
  }

  const tilesX = Math.floor(canvasWidth / cw);
  const tilesY = Math.floor(canvasHeight / ch);

  for (let sx = 0; sx < tilesX; sx++) {
    for (let sy = 0; sy < tilesY; sy++) {
      const absX = sx + state.xOffset;
      const absY = sy + state.yOffset;
      const px = sx * cw;
      const py = sy * ch;

      const sector = getSector(state, absX, absY);

      if (!sector) {
        // Fog of war — black
        ctx.fillStyle = '#000';
        ctx.fillRect(px, py, cw, ch);
        continue;
      }

      // Clear cell background
      ctx.fillStyle = '#000';
      ctx.fillRect(px, py, cw, ch);

      // Composite all enabled layers
      compositeRenderSector(ctx, ts, sector, state, absX, absY, px, py, cw, ch, activeLayers);
    }
  }
}
