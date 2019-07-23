
if [ ! -d "./osxcross" ]; then
  bash setup-macos.sh
fi

MACOS_TARGET="x86_64-apple-darwin"

echo "Building target for platform ${MACOS_TARGET}"
echo

# Add osxcross toolchain to path
export PATH="$PWD/osxcross/target/bin:$PATH"

# Make libz-sys (git2-rs -> libgit2-sys -> libz-sys) build as a statically linked lib
# This prevents the host zlib from being linked
export LIBZ_SYS_STATIC=1

# Use Clang for C/C++ builds
export CC=o64-clang
export CXX=o64-clang++

export PKG_CONFIG_ALLOW_CROSS=1
cross build --release --target "${MACOS_TARGET}"

echo
echo Done
