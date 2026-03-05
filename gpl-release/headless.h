// SPDX-License-Identifier: GPL-3.0-or-later
/*
 * headless.h - Headless mode support for non-interactive operation
 *
 * When CONQUER_HEADLESS env var is set, curses functions become no-ops
 * and interactive prompts use defaults or env var overrides.
 *
 * Environment variables (all optional):
 *   CONQUER_HEADLESS=1       - Enable headless mode
 *   CONQUER_PASSWORD=xxx     - God password (default: "god123")
 *   CONQUER_MAPX=32          - Map X size (default: 32, must be divisible by 8, >= 24)
 *   CONQUER_MAPY=32          - Map Y size (default: 32)
 *   CONQUER_WATER=30         - Water percentage (default: 30)
 *   CONQUER_SEED=12345       - RNG seed (default: time-based)
 */

#ifndef CONQUER_HEADLESS_H
#define CONQUER_HEADLESS_H

#include <stdlib.h>

static inline int conquer_is_headless(void) {
    const char *h = getenv("CONQUER_HEADLESS");
    return (h != NULL && h[0] != '0' && h[0] != '\0');
}

static inline const char *conquer_get_password(void) {
    const char *p = getenv("CONQUER_PASSWORD");
    return (p && p[0]) ? p : "god123";
}

static inline int conquer_get_mapx(void) {
    const char *v = getenv("CONQUER_MAPX");
    return v ? atoi(v) : 32;
}

static inline int conquer_get_mapy(void) {
    const char *v = getenv("CONQUER_MAPY");
    return v ? atoi(v) : 32;
}

static inline int conquer_get_water(void) {
    const char *v = getenv("CONQUER_WATER");
    return v ? atoi(v) : 30;
}

#endif /* CONQUER_HEADLESS_H */
