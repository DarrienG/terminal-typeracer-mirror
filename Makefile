CC=cargo
CFLAGS=--release
BIN=typeracer
BIN_PATH=target/release

all: release mac

release:
	$(CC) build $(CFLAGS)
	strip $(BIN_PATH)/$(BIN)

mac:
	#bash build-macos.sh
	@echo "broken :("

check:
	cargo clippy --all -- -D warnings
	cargo test
	cargo fmt -- --check

fmt:
	cargo fmt
