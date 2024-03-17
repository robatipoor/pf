.PHONY: local
local:
	cargo build --release

.PHONY: run
run:
ifndef ARGS
	@echo Run "make run" with ARGS set to pass arguments…
endif
	cargo run --release -- $(ARGS)

.PHONY: build-linux
build-linux:
	cargo build --target x86_64-unknown-linux-musl --release --locked
	strip target/x86_64-unknown-linux-musl/release/api
	upx --lzma target/x86_64-unknown-linux-musl/release/api
	strip target/x86_64-unknown-linux-musl/release/cli
	upx --lzma target/x86_64-unknown-linux-musl/release/cli

.PHONY: build-win
build-win:
	RUSTFLAGS="-C linker=x86_64-w64-mingw32-gcc" cargo build --target x86_64-pc-windows-gnu --release --locked
	strip target/x86_64-pc-windows-gnu/release/api.exe
	upx --lzma target/x86_64-pc-windows-gnu/release/api.exe
	strip target/x86_64-pc-windows-gnu/release/cli.exe
	upx --lzma target/x86_64-pc-windows-gnu/release/cli.exe

.PHONY: build-mac
build-mac:
	cargo build --target x86_64-apple-darwin --release --locked
	strip target/x86_64-apple-darwin/release/api
	upx --lzma target/x86_64-apple-darwin/release/api
	strip target/x86_64-apple-darwin/release/cli
	upx --lzma target/x86_64-apple-darwin/release/cli