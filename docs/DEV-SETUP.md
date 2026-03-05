# Conquer - Development Setup

## Prerequisites

- macOS (arm64) or Linux with clang/gcc
- ncurses development headers (usually included on macOS)
- make

## Building

```bash
cd gpl-release/
make clean && make all
```

The build uses `-std=gnu99 -D_GNU_SOURCE -w` flags. On macOS, `PREFIX` defaults to `$HOME/conquer`.

Binaries:
- `conqrun` — admin tool (make world, add player, run update)
- `conquer` — game client (curses-based player UI)
- `conqsort` — utility for sorting news files

## Headless Mode

Set `CONQUER_HEADLESS=1` to run without a terminal (no curses interaction):

```bash
# Generate a new world (non-interactive)
CONQUER_HEADLESS=1 CONQUER_PASSWORD=god123 CONQUER_SEED=42 \
  ./conqrun -m

# Run a game update/turn
CONQUER_HEADLESS=1 ./conqrun -x
```

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `CONQUER_HEADLESS` | (unset) | Set to `1` to enable headless mode |
| `CONQUER_PASSWORD` | `god123` | God/superuser password |
| `CONQUER_MAPX` | `32` | Map X size (must be divisible by 8, >= 24) |
| `CONQUER_MAPY` | `32` | Map Y size (must be divisible by 8, >= 24) |
| `CONQUER_WATER` | `30` | Water percentage (0-100) |
| `CONQUER_SEED` | (time) | RNG seed for deterministic generation |

## Oracle (State Dumper)

The oracle reads game data files and outputs JSON to stdout:

```bash
cd oracle/
make

# Dump world metadata
./oracle -d ~/conquer/lib -m

# Dump nations
./oracle -d ~/conquer/lib -n

# Dump everything (meta + nations + armies + sectors)
./oracle -d ~/conquer/lib
```

See [ORACLE.md](ORACLE.md) for details.

## Quick Test Cycle

```bash
# Full cycle: build, generate world, advance turn, dump state
cd gpl-release && make all && cd ..
rm -f ~/conquer/lib/data
CONQUER_HEADLESS=1 CONQUER_PASSWORD=god123 CONQUER_SEED=42 gpl-release/conqrun -m
CONQUER_HEADLESS=1 gpl-release/conqrun -x
oracle/oracle -d ~/conquer/lib > snapshots/latest.json
```

## Deterministic Testing

With `CONQUER_SEED` set, world gen and turn updates produce identical output:

```bash
# Two runs with same seed should produce identical state
CONQUER_SEED=42 ... conqrun -m   # run 1
oracle/oracle > /tmp/run1.json
CONQUER_SEED=42 ... conqrun -m   # run 2
oracle/oracle > /tmp/run2.json
diff /tmp/run1.json /tmp/run2.json  # should be empty
```
