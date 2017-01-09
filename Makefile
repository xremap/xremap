current_dir := $(shell pwd)
CSRCS := $(wildcard tools/xremap/*.[ch])
MRBSRCS := $(wildcard mrblib/xremap/*.rb)
MRBCSRCS := $(wildcard src/*.[ch])
# Using master to apply https://github.com/mruby/mruby/pull/3192
REVISION=0ff3ae1fbaed62010c54c43235e29cdc85da2f78
DESTDIR := /usr/local/bin
.PHONY: all clean install

all: xremap

clean:
	rm -rf mruby/build/host

install: xremap
	mv xremap $(DESTDIR)/xremap

xremap: mruby/build/host/bin/xremap
	cp mruby/build/host/bin/xremap xremap

mruby:
	git clone https://github.com/mruby/mruby
	git -C mruby reset --hard $(REVISION)

mruby/build/host/bin/xremap: mruby build_config.rb $(CSRCS) $(MRBSRCS) $(MRBCSRCS)
	cd mruby && MRUBY_CONFIG="$(current_dir)/build_config.rb" make
