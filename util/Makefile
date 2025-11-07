# shackle - seccomp jail
# Copyright (C) 2025 Olive Hudson
# see LICENCE file for licensing information

.POSIX:

VERSION = 1.0.0

PREFIX = /usr/local

WPROFILE = -Wall -Wextra -Wstrict-prototypes -Wmissing-declarations -Wshadow \
-Wswitch-default -Wunreachable-code -Wcast-align -Wpointer-arith -Wcast-qual \
-Wbad-function-cast -Winline -Wundef -Wnested-externs -Wwrite-strings \
-Wno-unused-parameter -Wfloat-equal -Wpedantic
STD = -D_DEFAULT_SOURCE -D_POSIX_C_SOURCE=200809L -D_XOPEN_SOURCE=500

CFLAGS = $(WPROFILE) $(STD) -Os
LDFLAGS = -lseccomp

SRC = shackle.c
OBJ = $(SRC:.c=.o)

all: shackle

.c.o:
	$(CC) $(CFLAGS) -c $<

shackle: $(OBJ)
	$(CC) $(OBJ) -o $@ $(LDFLAGS)

clean:
	rm -f shackle $(OBJ)

install: all
	mkdir -p $(PREFIX)/bin
	cp -f shackle $(PREFIX)/bin
	chown root:users $(PREFIX)/bin/shackle
	chmod 4754 $(PREFIX)/bin/shackle

uninstall:
	rm -f $(PREFIX)/bin/shackle

.PHONY: all clean install uninstall
