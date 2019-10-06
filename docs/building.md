# Building

It's a rust project with one external dependency: git

You may need openssl installed as well.

Rust version required is 1.33.0 or higher (using some newer time APIs).

```bash
$ cargo build --release
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
