// SPDX-License-Identifier: GPL-3.0-or-later
/*
 * conquer_rand.h - Portable seeded RNG for deterministic game state
 *
 * Replaces platform rand()/srand() with a simple LCG for reproducibility.
 * When CONQUER_SEED env var is set, uses that seed for deterministic output.
 * Otherwise uses time-based seed (original behavior).
 *
 * LCG parameters: seed = seed * 1103515245 + 12345
 * Output: (seed >> 16) & 0x7fff  (same as glibc's internal LCG)
 */

#ifndef CONQUER_RAND_H
#define CONQUER_RAND_H

#include <stdlib.h>

static unsigned long _conquer_rng_state = 1;

static inline void conquer_srand(unsigned int seed) {
    _conquer_rng_state = seed;
}

static inline int conquer_rand(void) {
    _conquer_rng_state = _conquer_rng_state * 1103515245UL + 12345UL;
    return (int)((_conquer_rng_state >> 16) & 0x7fff);
}

/* Override standard rand/srand */
#define rand() conquer_rand()
#define srand(x) conquer_srand((x))

/* RAND_MAX for our LCG */
#undef RAND_MAX
#define RAND_MAX 0x7fff

#endif /* CONQUER_RAND_H */
