# Oracle - Conquer game state dumper
# Builds against the game's object files

GPLDIR = ../gpl-release
CC = cc
CFLAGS = -std=gnu99 -D_GNU_SOURCE -w -DADMIN -I$(GPLDIR)

# We need these .o files from the game build (all admin objects except admin.o which has main)
GAME_OBJS = $(GPLDIR)/combat.o $(GPLDIR)/cexecuteA.o $(GPLDIR)/ioA.o \
            $(GPLDIR)/makeworl.o $(GPLDIR)/navyA.o $(GPLDIR)/spew.o \
            $(GPLDIR)/newlogin.o $(GPLDIR)/update.o $(GPLDIR)/magicA.o \
            $(GPLDIR)/npc.o $(GPLDIR)/miscA.o $(GPLDIR)/randeven.o \
            $(GPLDIR)/dataA.o $(GPLDIR)/tradeA.o $(GPLDIR)/check.o

# macOS libraries (no -lcrypt on macOS — crypt() is in libc)
LIBRARIES = -lcurses

all: oracle

# Compile oracle.c with game headers
oracle.o: oracle.c $(GPLDIR)/header.h $(GPLDIR)/data.h
	$(CC) $(CFLAGS) -c oracle.c -o oracle.o

# Link oracle with game objects
oracle: oracle.o $(GAME_OBJS)
	$(CC) $(CFLAGS) -o oracle oracle.o $(GAME_OBJS) $(LIBRARIES)

clean:
	rm -f oracle oracle.o

.PHONY: all clean
