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

There is full cross compile support, but only if you are on macOS! This is
because I only use macOS at home. If you would like to be able to compile to
macOS from Linux, I'm happy to take PRs :)

There are two dependencies:
- docker
- openssl (the x86 and ARM versions)


To build run:

```
./build-all.sh
```

If you're on an M1 Mac, it will build binaries for all common architectures at
once! This may take a minute or two.

The script makes an assumption about the location of openssl - that it is
installed in the default homebrew location.

If on your machine it's somewhere else, you should make a change to the script!
