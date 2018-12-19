SHELL := /bin/bash
COMPILER = rustc
COMPILER_FLAGS = -O
RUSTDOC = rustdoc
UPX := $(shell command -v upx 2> /dev/null)

.PHONY: all
all:
	cargo build --release 
	strip target/release/pf
ifdef UPX
		upx target/release/pf 
endif
	cargo install --path . 

.PHONY: build
build:
	cargo build --release 
	strip target/release/pf
	upx target/release/pf 

.PHONY: run
run:
	cargo run

.PHONY: install
install:
	cargo install --path .

.PHONY: clean
clean:
	cargo clean