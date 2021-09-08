#!/usr/bin/env bash

set -eou pipefail

CONTAINER_NAME="$(uuidgen)"
IMAGE_NAME="typeracer-linux-build"

BINARY="typeracer"

# create our context or if we've already created it, just move on
docker buildx create --platform linux/amd64 --name typeracerx86 || true
docker buildx use typeracerx86
docker buildx build -t "$IMAGE_NAME" .
docker run --rm -d --name "$CONTAINER_NAME" "$IMAGE_NAME"

LINUX_X86_FOLDER="target/aarch64-unknown-linux-gnu"
mkdir -p "$LINUX_X86_FOLDER"
LINUX_X86_TARGET="$LINUX_X86_FOLDER/$BINARY"

docker cp $CONTAINER_NAME:/project/typeracer ./target/aarch64-unknown-linux-gnu/
docker kill --signal SIGKILL "$CONTAINER_NAME"

cargo build --release --target aarch64-apple-darwin

# openssl must be installed with rosetta brew
# brew install openssl
X86_64_APPLE_DARWIN_OPENSSL_DIR="/usr/local/opt/openssl" cargo build --target x86_64-apple-darwin --release

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
echo -e "Consider running cargo clean to clean up alternative architectures and docker rmi to remove old docker images"
