// SPDX-License-Identifier: GPL-3.0-or-later
/*
 * oracle.c - Conquer game state dumper (JSON output)
 *
 * Reads the conquer data file and dumps game state to stdout as JSON.
 * Used for testing and verification of game state after world gen or turn advance.
 *
 * Usage: oracle [-d DATADIR] [-s] [-n] [-a] [-m]
 *   -d DATADIR  Data directory (default: $PREFIX/lib or DEFAULTDIR)
 *   -s          Dump sectors (map data)
 *   -n          Dump nations
 *   -a          Dump armies
 *   -m          Dump world metadata
 *   (no flags = dump everything)
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

/* We need game headers */
#define ADMIN
#define main conquer_main_unused  /* avoid conflict with data.h extern void main() */
#include "header.h"
#include "data.h"
#undef main

/* Game globals (same as admin.c) */
struct s_sector **sct;
struct s_nation ntn[NTOTAL];
struct s_world world;
char **occ;
short **movecost;
long startgold = 0;
short xoffset = 0, yoffset = 0;
short xcurs = 0, ycurs = 0;
short dismode = 2;
short country = 0;
struct s_nation *curntn;
char datadir[FILELTH];

#ifdef REMAKE
int remake = FALSE;
#endif

FILE *fexe;

/* Stubs for functions defined in admin.c (which we can't link due to main conflict) */
void att_setup(int cntry) { (void)cntry; }
void att_base() {}
void att_bonus() {}

/* JSON helpers */
static void json_string(FILE *out, const char *s) {
    fputc('"', out);
    for (; *s; s++) {
        switch (*s) {
            case '"': fputs("\\\"", out); break;
            case '\\': fputs("\\\\", out); break;
            case '\n': fputs("\\n", out); break;
            case '\t': fputs("\\t", out); break;
            default:
                if ((unsigned char)*s < 0x20)
                    fprintf(out, "\\u%04x", (unsigned char)*s);
                else
                    fputc(*s, out);
        }
    }
    fputc('"', out);
}

static void dump_world_meta(FILE *out) {
    fprintf(out, "\"world\": {\n");
    fprintf(out, "  \"mapx\": %d,\n", (int)world.mapx);
    fprintf(out, "  \"mapy\": %d,\n", (int)world.mapy);
    fprintf(out, "  \"turn\": %ld,\n", TURN);
    /* fprintf(out, "  \"m_pts\": %ld,\n", world.m_pts); -- field doesn't exist */
    fprintf(out, "  \"score\": %ld,\n", WORLDSCORE);
    fprintf(out, "  \"gold\": %ld,\n", WORLDGOLD);
    fprintf(out, "  \"food\": %ld,\n", WORLDFOOD);
    fprintf(out, "  \"jewels\": %ld,\n", WORLDJEWELS);
    fprintf(out, "  \"metal\": %ld\n", WORLDMETAL);
    fprintf(out, "}");
}

static void dump_nations(FILE *out) {
    int i;
    fprintf(out, "\"nations\": [\n");
    for (i = 0; i < NTOTAL; i++) {
        if (i > 0) fprintf(out, ",\n");
        fprintf(out, "  {\n");
        fprintf(out, "    \"id\": %d,\n", i);
        fprintf(out, "    \"name\": "); json_string(out, ntn[i].name); fprintf(out, ",\n");
        fprintf(out, "    \"leader\": "); json_string(out, ntn[i].leader); fprintf(out, ",\n");
        fprintf(out, "    \"active\": %d,\n", (int)ntn[i].active);
        fprintf(out, "    \"race\": \"%c\",\n", ntn[i].race ? ntn[i].race : '?');
        fprintf(out, "    \"mark\": \"%c\",\n", ntn[i].mark ? ntn[i].mark : '?');
        fprintf(out, "    \"tgold\": %ld,\n", ntn[i].tgold);
        fprintf(out, "    \"tfood\": %ld,\n", ntn[i].tfood);
        fprintf(out, "    \"tciv\": %ld,\n", ntn[i].tciv);
        fprintf(out, "    \"tmil\": %ld,\n", ntn[i].tmil);
        fprintf(out, "    \"tsctrs\": %ld,\n", ntn[i].tsctrs);
        fprintf(out, "    \"score\": %ld,\n", ntn[i].score);
        fprintf(out, "    \"metals\": %ld,\n", ntn[i].metals);
        fprintf(out, "    \"jewels\": %ld,\n", ntn[i].jewels);
        fprintf(out, "    \"capx\": %d,\n", (int)ntn[i].capx);
        fprintf(out, "    \"capy\": %d\n", (int)ntn[i].capy);
        fprintf(out, "  }");
    }
    fprintf(out, "\n]");
}

static void dump_armies(FILE *out) {
    int i, j;
    int first = 1;
    fprintf(out, "\"armies\": [\n");
    for (i = 0; i < NTOTAL; i++) {
        for (j = 0; j < MAXARM; j++) {
            if (ntn[i].arm[j].sold <= 0) continue;
            if (!first) fprintf(out, ",\n");
            first = 0;
            fprintf(out, "  {\"nation\": %d, \"army\": %d, \"xloc\": %d, \"yloc\": %d, ",
                    i, j, (int)ntn[i].arm[j].xloc, (int)ntn[i].arm[j].yloc);
            fprintf(out, "\"sold\": %d, \"type\": %d, \"stat\": %d}",
                    (int)ntn[i].arm[j].sold, (int)ntn[i].arm[j].unittyp,
                    (int)ntn[i].arm[j].stat);
        }
    }
    fprintf(out, "\n]");
}

static void dump_sectors(FILE *out) {
    int x, y;
    fprintf(out, "\"sectors\": [\n");
    for (x = 0; x < MAPX; x++) {
        for (y = 0; y < MAPY; y++) {
            if (x > 0 || y > 0) fprintf(out, ",\n");
            fprintf(out, "  {\"x\": %d, \"y\": %d, \"owner\": %d, \"des\": \"%c\", ",
                    x, y, (int)sct[x][y].owner, sct[x][y].designation);
            fprintf(out, "\"alt\": %d, \"veg\": %d, \"people\": %d, \"metal\": %d, \"jewels\": %d}",
                    (int)sct[x][y].altitude, (int)sct[x][y].vegetation,
                    (int)sct[x][y].people, (int)sct[x][y].metal, (int)sct[x][y].jewels);
        }
    }
    fprintf(out, "\n]");
}

int main(int argc, char **argv) {
    int opt;
    int do_sectors = 0, do_nations = 0, do_armies = 0, do_meta = 0;
    int first;
    char defaultdir[BIGLTH];

    strcpy(datadir, "");

    while ((opt = getopt(argc, argv, "d:snam")) != -1) {
        switch (opt) {
            case 'd': strcpy(datadir, optarg); break;
            case 's': do_sectors = 1; break;
            case 'n': do_nations = 1; break;
            case 'a': do_armies = 1; break;
            case 'm': do_meta = 1; break;
            default:
                fprintf(stderr, "Usage: %s [-d datadir] [-s] [-n] [-a] [-m]\n", argv[0]);
                return 1;
        }
    }

    /* if no flags, dump everything */
    if (!do_sectors && !do_nations && !do_armies && !do_meta) {
        do_sectors = do_nations = do_armies = do_meta = 1;
    }

    /* resolve data directory */
    if (datadir[0] != '/') {
        if (strlen(datadir) > 0) {
            sprintf(defaultdir, "%s/%s", DEFAULTDIR, datadir);
        } else {
            strcpy(defaultdir, DEFAULTDIR);
        }
    } else {
        strcpy(defaultdir, datadir);
    }

    if (chdir(defaultdir)) {
        fprintf(stderr, "oracle: cannot chdir to %s\n", defaultdir);
        return 1;
    }

    /* read game data */
    readdata();

    /* dump JSON */
    printf("{\n");
    first = 1;
    if (do_meta) {
        if (!first) printf(",\n");
        dump_world_meta(stdout);
        first = 0;
    }
    if (do_nations) {
        if (!first) printf(",\n");
        dump_nations(stdout);
        first = 0;
    }
    if (do_armies) {
        if (!first) printf(",\n");
        dump_armies(stdout);
        first = 0;
    }
    if (do_sectors) {
        if (!first) printf(",\n");
        dump_sectors(stdout);
        first = 0;
    }
    printf("\n}\n");

    return 0;
}
