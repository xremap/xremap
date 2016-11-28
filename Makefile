current_dir := $(shell pwd)
.PHONY: all

all: xkremap

xkremap: mruby/build/host/bin/xkremap
	cp mruby/build/host/bin/xkremap xkremap

mruby:
	curl -L --fail --retry 3 --retry-delay 1 https://github.com/mruby/mruby/archive/1.2.0.tar.gz -s -o - | tar zxf -
	mv mruby-1.2.0 $@

mruby/build/host/bin/xkremap: mruby build_config.rb tools/xkremap/main.c tools/xkremap/remap.c tools/xkremap/xkremap.h
	cd mruby && MRUBY_CONFIG="$(current_dir)/build_config.rb" make
