# Building

It's a rust project with two external dependencies: git and sqlite

You may need openssl installed as well.

Rust version required is 1.58.1 or higher. Apparently rust changed the
Cargo.lock format with 1.58.1 and this breaks builds with older version of rust.
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

There are three dependencies:
- Python/pip
- Cargo
- Zig


Zig will be installed along the way if you don't have it.

To build run:

```
./build-all.sh
```

This will take a little time!
