#!/usr/bin/env bash

export PATH=/usr/local/osx-ndk-x86/bin:$PATH
export PKG_CONFIG_ALLOW_CROSS=1

cargo build --target=x86_64-apple-darwin --release
