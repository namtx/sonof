INSTALL_DIR := /Users/namtx/.cargo/bin

.PHONY: install build release clean

install: release
	cp target/release/sonof $(INSTALL_DIR)/sonof

release:
	cargo build --release

build:
	cargo build

clean:
	cargo clean
