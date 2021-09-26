# quick n dirty makefile to build rust binary
# based off the output of `cargo build --release --verbose` and `cargo build --verbose`

SHELL := /bin/bash

ROOT := $(shell realpath .)
COMMON_FLAGS = main.rs --edition=2018 --crate-type bin
RUSTC = rustc $(COMMON_FLAGS) --emit=dep-info,link

# release build
balls:
	$(RUSTC) --crate-name $@ -C opt-level=3
-include balls.d

# debug build
balls_dbg:
	$(RUSTC) --crate-name $@ -C debuginfo=2
-include balls_dbg.d

# Generate Documentation
# (Try opening doc/balls/index.html in a browser)
.PHONY: doc
doc:
	rustdoc $(COMMON_FLAGS) --crate-name balls --document-private-items

# build release and debug binary
all: balls balls_dbg

# professor-proofing the makefile by adding aliases
.PHONY: docs build build_dbg
docs: doc
build: balls
build_dbg: balls_dbg


# Clean build dir
.PHONY: clean
clean:
	rm -rf balls balls.d balls_dbg balls_dbg.d doc balls*.o
