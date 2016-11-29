current_dir := $(shell pwd)
CSRCS := $(wildcard tools/xkremap/*.[ch])
MRBSRCS := $(wildcard mrblib/xkremap/*.rb)
MRBCSRCS := $(wildcard src/*.[ch])
# Using master to apply https://github.com/mruby/mruby/pull/3192
REVISION=0ff3ae1fbaed62010c54c43235e29cdc85da2f78
.PHONY: all clean

all: xkremap

clean:
	rm -rf mruby/build/host

xkremap: mruby/build/host/bin/xkremap
	cp mruby/build/host/bin/xkremap xkremap

mruby:
	git clone https://github.com/mruby/mruby
	git -C mruby reset --hard $(REVISION)

mruby/build/host/bin/xkremap: mruby build_config.rb $(CSRCS) $(MRBSRCS) $(MRBCSRCS)
	cd mruby && MRUBY_CONFIG="$(current_dir)/build_config.rb" make
