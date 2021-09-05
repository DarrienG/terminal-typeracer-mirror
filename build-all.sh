#!/usr/bin/env bash

set -eou pipefail

CONTAINER_NAME="$(uuidgen)"
IMAGE_NAME="typeracer-linux-build"

docker build -t "$IMAGE_NAME" .
docker run --rm -d --name "$CONTAINER_NAME" "$IMAGE_NAME"
mkdir -p ./target/aarch64-unknown-linux-gnu

docker cp $CONTAINER_NAME:/project/typeracer ./target/aarch64-unknown-linux-gnu/
docker kill --signal SIGKILL "$CONTAINER_NAME"

make
