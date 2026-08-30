.PHONY: help all update lint build install clean

help:
	@echo "update     update cargo dependencies"
	@echo "lint       run cargo clippy"
	@echo "build      build release binary"
	@echo "install    copy built binary to ~/.clap"
	@echo "all        update + lint + build + install"
	@echo "clean      remove build artifacts"

all: update lint build install

lint:
	cargo clippy --all-targets -- -D warnings -D clippy::all -D clippy::pedantic

update:
	cargo update

build:
	cargo build --release

install:
	@cp -v target/release/libnam.so ~/.clap/nam.clap

clean:
	cargo clean
