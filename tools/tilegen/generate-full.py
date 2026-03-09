#!/usr/bin/env python3
"""
Full tileset generator — reads tile-list-full.json, skips existing tiles,
generates via OpenAI Images API, downscales to target size.

Usage:
  python3 generate-full.py                    # Generate all missing pixel32 tiles
  python3 generate-full.py --phase 4          # Only phase 4
  python3 generate-full.py --dry-run          # Show what would be generated
  python3 generate-full.py --style pixel64    # Different style
"""

import json
import os
import sys
import time
import argparse
import base64
import urllib.request
import urllib.error

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
TILE_LIST = os.path.join(SCRIPT_DIR, 'tile-list-full.json')
CONFIG = os.path.join(SCRIPT_DIR, 'config.json')
CONQUER_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIR))


def load_style_prompt(style_id):
    with open(CONFIG) as f:
        cfg = json.load(f)
    for s in cfg['styles']:
        if s['id'] == style_id:
            return s['prompt'], s['size']
    raise ValueError(f"Style '{style_id}' not found in config.json")


def load_tiles(phase_filter=None):
    with open(TILE_LIST) as f:
        data = json.load(f)
    tiles = [t for t in data['tiles'] if 'tile_id' in t]
    if phase_filter is not None:
        tiles = [t for t in tiles if t['phase'] == phase_filter]
    return tiles


def tile_to_filename(tile_id):
    return tile_id.replace('.', '_') + '.png'


def generate_image(prompt, api_key):
    """Call OpenAI Images API directly, return PNG bytes."""
    url = "https://api.openai.com/v1/images/generations"
    payload = json.dumps({
        "model": "gpt-image-1",
        "prompt": prompt,
        "n": 1,
        "size": "1024x1024",
        "quality": "high",
    }).encode('utf-8')

    req = urllib.request.Request(url, data=payload, headers={
        "Content-Type": "application/json",
        "Authorization": f"Bearer {api_key}",
    })

    try:
        with urllib.request.urlopen(req, timeout=120) as resp:
            result = json.loads(resp.read().decode('utf-8'))

        # gpt-image-1 returns b64_json
        if result.get('data') and result['data'][0].get('b64_json'):
            return base64.b64decode(result['data'][0]['b64_json'])

        # Fallback: URL-based response
        if result.get('data') and result['data'][0].get('url'):
            img_url = result['data'][0]['url']
            with urllib.request.urlopen(img_url, timeout=60) as img_resp:
                return img_resp.read()

        raise ValueError(f"Unexpected API response: {json.dumps(result)[:200]}")
    except urllib.error.HTTPError as e:
        body = e.read().decode('utf-8', errors='replace')
        raise RuntimeError(f"API error {e.code}: {body[:300]}")


def downscale(src_path, dst_path, target_size=32):
    """Downscale image to target size using nearest-neighbor."""
    from PIL import Image
    img = Image.open(src_path)
    img = img.resize((target_size, target_size), Image.NEAREST)
    img.save(dst_path)


def main():
    parser = argparse.ArgumentParser(description='Generate tileset images')
    parser.add_argument('--style', default='pixel32', help='Style ID from config.json')
    parser.add_argument('--phase', type=int, default=None, help='Only generate this phase')
    parser.add_argument('--dry-run', action='store_true', help='Show what would be generated')
    parser.add_argument('--limit', type=int, default=None, help='Max tiles to generate')
    args = parser.parse_args()

    api_key = os.environ.get('OPENAI_API_KEY')
    if not api_key and not args.dry_run:
        print("ERROR: OPENAI_API_KEY not set")
        sys.exit(1)

    master_prompt, target_size = load_style_prompt(args.style)
    tiles = load_tiles(args.phase)

    # Output dirs
    source_dir = os.path.join(SCRIPT_DIR, 'runs', f'{args.style}_full')
    final_dir = os.path.join(CONQUER_ROOT, 'web', 'public', 'tilesets', args.style)
    os.makedirs(source_dir, exist_ok=True)
    os.makedirs(final_dir, exist_ok=True)

    # Filter out existing tiles
    to_generate = []
    skipped = 0
    for t in tiles:
        filename = tile_to_filename(t['tile_id'])
        if os.path.exists(os.path.join(final_dir, filename)):
            skipped += 1
        else:
            to_generate.append(t)

    if args.limit:
        to_generate = to_generate[:args.limit]

    print("═" * 57)
    print(f"  Tileset Generation: {args.style}")
    print(f"  Phase: {args.phase or 'all'}")
    print(f"  Master: {master_prompt[:70]}...")
    print(f"  Target size: {target_size}px")
    print(f"  Source: {source_dir}")
    print(f"  Final:  {final_dir}")
    print("═" * 57)
    print(f"\nTo generate: {len(to_generate)}  |  Skipped (exist): {skipped}")
    print()

    if not to_generate:
        print("Nothing to generate — all tiles exist!")
        return

    if args.dry_run:
        print("=== DRY RUN ===")
        for t in to_generate:
            fn = tile_to_filename(t['tile_id'])
            print(f"  Phase {t['phase']}: {fn:40s} {t['prompt_fragment'][:60]}...")
        return

    # Generate!
    start = time.time()
    generated = 0
    failed = 0

    for i, t in enumerate(to_generate):
        filename = tile_to_filename(t['tile_id'])
        src_path = os.path.join(source_dir, filename)
        dst_path = os.path.join(final_dir, filename)
        elapsed = int(time.time() - start)

        print(f"[{i+1}/{len(to_generate)}] ({elapsed}s) {t['tile_id']}")

        full_prompt = f"{master_prompt} {t['prompt_fragment']}"

        try:
            png_data = generate_image(full_prompt, api_key)

            # Save 1024px source
            with open(src_path, 'wb') as f:
                f.write(png_data)

            # Downscale to target
            downscale(src_path, dst_path, target_size)

            print(f"  → {filename} ✓ (1024→{target_size}px)")
            generated += 1

        except Exception as e:
            print(f"  → FAILED: {e}")
            failed += 1

        # Brief pause between requests
        time.sleep(0.5)

    total_time = int(time.time() - start)
    total_tiles = len([f for f in os.listdir(final_dir) if f.endswith('.png')])

    print()
    print("═" * 57)
    print(f"  Done in {total_time}s ({total_time // 60}m {total_time % 60}s)")
    print(f"  Generated: {generated}  |  Failed: {failed}")
    print(f"  Total tiles in {args.style}: {total_tiles}")
    print("═" * 57)


if __name__ == '__main__':
    main()
