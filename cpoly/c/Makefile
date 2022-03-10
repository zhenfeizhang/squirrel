CC = /usr/bin/gcc
CFLAGS = -Wall -g -Wextra -march=native -mtune=native -mavx2

SOURCES = poly.c misc.c hvc_ntt.c hots_ntt.c foreign/*.c
HEADERS = poly.h misc.h foreign/*.h params.h

all: test

build: $(SOURCES) $(HEADERS)
	$(CC) $(CFLAGS) -UDBENCH $< $(SOURCES) -o $@

test: test.c $(SOURCES) $(HEADERS)
	$(CC) $(CFLAGS) -UDBENCH $< $(SOURCES) -o $@

.PHONY: clean

clean:
	rm -f test
	rm -f *.o
