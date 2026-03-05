# Conquer Oracle — Game State Dumper

## Overview

The oracle is a standalone C program that reads the Conquer game data file and dumps the game state as JSON to stdout. It's used for:

- **Snapshot testing** — capture state after world gen or turn advance
- **Regression detection** — diff snapshots across code changes
- **Game analysis** — inspect nation stats, army positions, map layout

## Building

```bash
cd oracle/
make
```

The oracle links against the game's compiled object files, so build the game first (`cd gpl-release && make all`).

## Usage

```bash
./oracle [-d DATADIR] [-s] [-n] [-a] [-m]
```

| Flag | Description |
|------|-------------|
| `-d DIR` | Data directory (default: `$PREFIX/lib` or DEFAULTDIR from header.h) |
| `-s` | Dump sectors (map grid with altitude, vegetation, owner, people, resources) |
| `-n` | Dump nations (stats, gold, food, military, score) |
| `-a` | Dump armies (position, size, type, status per nation) |
| `-m` | Dump world metadata (map size, turn, global totals) |
| (none) | Dump everything |

## JSON Schema

```json
{
  "world": {
    "mapx": 32, "mapy": 32,
    "turn": 1,
    "score": 1, "gold": 772001, "food": 1311001,
    "jewels": 150001, "metal": 150001
  },
  "nations": [
    {
      "id": 0, "name": "unowned", "leader": "god",
      "active": 0, "race": "-", "mark": "-",
      "tgold": 0, "tfood": 0, "tciv": 0, "tmil": 0,
      "tsctrs": 0, "score": 0, "metals": 0, "jewels": 0,
      "capx": 0, "capy": 0
    }
  ],
  "armies": [
    {"nation": 1, "army": 0, "xloc": 10, "yloc": 15,
     "sold": 500, "type": 1, "stat": 2}
  ],
  "sectors": [
    {"x": 0, "y": 0, "owner": 0, "des": " ",
     "alt": 3, "veg": 2, "people": 0, "metal": 0, "jewels": 0}
  ]
}
```

## Snapshots

Store snapshots in `snapshots/`:

```bash
# After world gen
oracle/oracle -d ~/conquer/lib > snapshots/turn-001.json

# After turn advance
CONQUER_HEADLESS=1 gpl-release/conqrun -x
oracle/oracle -d ~/conquer/lib > snapshots/turn-002.json

# Compare
diff snapshots/turn-001.json snapshots/turn-002.json
```

## Implementation Notes

- Links against all admin `.o` files except `admin.o` (which contains `main()`)
- Provides stubs for `att_setup()`, `att_base()`, `att_bonus()` (only needed for live game updates)
- Uses the game's `readdata()` function to load the binary data file
- JSON output is human-readable (indented, one entry per line for arrays)
