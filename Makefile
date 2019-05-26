CC=cargo
CFLAGS=--release
BIN=typeracer
BIN_PATH=target/release

release:
	$(CC) build $(CFLAGS)
	strip $(BIN_PATH)/$(BIN)
