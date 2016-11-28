current_dir := $(shell pwd)
CSRCS := $(wildcard tools/xkremap/*.[ch])
RBSRCS := $(wildcard mrblib/xkremap/*.rb)
.PHONY: all

all: xkremap

xkremap: mruby/build/host/bin/xkremap
	cp mruby/build/host/bin/xkremap xkremap

mruby:
	curl -L --fail --retry 3 --retry-delay 1 https://github.com/mruby/mruby/archive/1.2.0.tar.gz -s -o - | tar zxf -
	mv mruby-1.2.0 $@

mruby/build/host/bin/xkremap: mruby build_config.rb $(CSRCS) $(RBSRCS)
	cd mruby && MRUBY_CONFIG="$(current_dir)/build_config.rb" make
