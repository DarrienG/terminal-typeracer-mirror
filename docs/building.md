# Building

It's a rust project with one external dependency: git

You may need openssl installed as well.

Rust version required is 1.42.0 or higher. Apparently rust changed the
Cargo.lock format with 1.42.0 and this breaks builds with older version of rust.
So I guess latest is greatest.

```bash
$ cargo build --release
```

You may find that even though you have libssl-dev installed and pkg-config, you
still can't compile. If you can't, you'll want to supply the PKG_CONFIG path.

On Ubuntu, it looks like this:

```bash
PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig cargo build --release
```

The binary you'll get is called `typeracer` and runnable immediately!

## Cross compile

**NOTE: This is currently broken, but are planning on having it working again in
the future.**

If you're trying to cross compile to macOS from Linux you'll need:

```
clang
g++
gcc
git
zlib1g-dev
libmpc-dev
libmpfr-dev
libgmp-dev
```

and to add the apple target via rustup:

```bash
$ rustup target add x86_64-apple-darwin
```

You can then build just the macOS target with

```
$ make mac
```
