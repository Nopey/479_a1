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

# Benchmark
# (Prefer hyperfine if it is present, but fall back to time)
.PHONY: bench
ifeq (, $(shell which hyperfine))
bench: balls
	time ./balls A1-input1.txt
	time ./balls A1-input2.txt
	time ./balls A1-input3.txt
	time ./balls A1-input4.txt
else
bench: balls
	@# Hyperfine needs the command to be in quotes
	hyperfine './balls A1-input1.txt'
	hyperfine './balls A1-input2.txt'
	hyperfine './balls A1-input3.txt'
	hyperfine './balls A1-input4.txt'
endif

# build everything
all: balls balls_dbg doc

# professor-proofing the makefile by adding aliases
.PHONY: docs build ball build_dbg debug benchmark
docs: doc
build: balls
ball: balls
build_dbg: balls_dbg
debug: balls_dbg
benchmark: bench


# Clean build dir
.PHONY: clean
clean:
	rm -rf balls balls.d balls_dbg balls_dbg.d doc balls*.o
