LD=gcc
LDFLAGS=-lX11
OBJS=src/main.o
.PHONY: all

all: xkremap

xkremap: $(OBJS)
	$(LD) $(OBJS) $(LDFLAGS) -o $@
