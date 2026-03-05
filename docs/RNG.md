# Conquer RNG — Portable Seeded Random Number Generator

## Overview

The game's `rand()`/`srand()` calls are replaced with a portable Linear Congruential Generator (LCG) via `conquer_rand.h`. This ensures:

1. **Deterministic output** — same seed always produces same game state
2. **Cross-platform consistency** — LCG is platform-independent (unlike libc `rand()`)
3. **Reproducible tests** — set `CONQUER_SEED=42` for identical world gen across runs

## Algorithm

```c
// LCG parameters (same as glibc internal)
seed = seed * 1103515245 + 12345;
return (seed >> 16) & 0x7fff;
```

- **Period**: 2^32
- **Output range**: 0 to 32767 (RAND_MAX = 0x7fff)
- **State**: single 32-bit unsigned long

## Usage

### In C code

No changes needed — `rand()` and `srand()` are macro-redirected to `conquer_rand()` and `conquer_srand()` via `conquer_rand.h` (included from `header.h`).

### From command line

```bash
# Deterministic world gen
CONQUER_HEADLESS=1 CONQUER_SEED=42 CONQUER_PASSWORD=god123 ./conqrun -m

# Deterministic turn advance
CONQUER_HEADLESS=1 CONQUER_SEED=42 ./conqrun -x
```

### Seeding behavior

| CONQUER_SEED | Behavior |
|---|---|
| Set (e.g. `42`) | Deterministic — identical output every run |
| Unset | Time-based seed (original behavior) |

## Verification

```bash
# Run 1
CONQUER_SEED=42 conqrun -m && oracle > /tmp/a.json
# Run 2
CONQUER_SEED=42 conqrun -m && oracle > /tmp/b.json
# Must match
diff /tmp/a.json /tmp/b.json  # empty output = deterministic ✓
```

## Files

- `gpl-release/conquer_rand.h` — LCG implementation + rand/srand macros
- `gpl-release/header.h` — includes conquer_rand.h (so all .c files use it)
- `gpl-release/admin.c` — reads CONQUER_SEED env var for initial seeding

## Why LCG?

- Simple, fast, well-understood
- Same parameters as glibc's internal rand() (widely tested)
- No external dependencies
- Output quality sufficient for a strategy game (not cryptographic)
- Easy to verify cross-platform (just a multiply and add)
