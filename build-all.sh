#!/usr/bin/env bash

set -eou pipefail

# Zig is our cross compiler. Yeah it's weird.
pip3 install ziglang
cargo install cargo-zigbuild

LINUX_X86_TARGET="target/x86_64-unknown-linux-gnu/release/typeracer"
cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.28
LINUX_ARM_TARGET="target/aarch64-unknown-linux-gnu/release/typeracer"
cargo zigbuild --release --target aarch64-unknown-linux-gnu.2.28

cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

UNIVERSAL_FOLDER="target/universal-apple-darwin"

mkdir -p "$UNIVERSAL_FOLDER"

MACOS_ARM_TARGET="target/aarch64-apple-darwin/release/$BINARY"
MACOS_X86_TARGET="target/x86_64-apple-darwin/release/$BINARY"
MACOS_UNIVERSAL_TARGET="$UNIVERSAL_FOLDER/$BINARY"

lipo -create -output "$MACOS_UNIVERSAL_TARGET" "$MACOS_ARM_TARGET" "$MACOS_X86_TARGET"
strip "$MACOS_UNIVERSAL_TARGET"

echo -e "All done!"
echo -e "Binaries can be found in the following locations:"
echo -e "MACOS_UNIVERSAL:\n\t- $MACOS_UNIVERSAL_TARGET"
echo -e "LINUX_X86:\n\t- $LINUX_X86_TARGET"
echo -e "LINUX_ARM:\n\t- $LINUX_ARM_TARGET"
