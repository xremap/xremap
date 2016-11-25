LD=gcc
OBJS=src/main.o
.PHONY: all

all: xkremap

xkremap: $(OBJS)
	$(LD) $(OBJS) -o $@
