SUBDIRS := c

all: build

build:
	$(MAKE) -C $(SUBDIRS) all
	bindgen c/poly.h -o src/bindings.rs

.PHONY: clean

clean: 
	$(MAKE) -C $(SUBDIRS) clean

