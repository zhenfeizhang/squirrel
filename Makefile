SUBDIRS := cpoly

all: build

build:
	$(MAKE) -C $(SUBDIRS) all

test: build
	cpoly/c/test

.PHONY: clean

clean: 
	$(MAKE) -C $(SUBDIRS) clean

