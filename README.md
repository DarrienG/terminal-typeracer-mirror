# Typeracer

[![pipeline status](https://gitlab.com/DarrienG/terminal-typeracer/badges/master/pipeline.svg)](https://gitlab.com/DarrienG/terminal-typeracer/commits/master)

An open source terminal based version of Typeracer written in rust.

Gives you a random passage and you type it out. While you type it will tell you
where you're making errors and give you a set of words per minute.

![User typing away having a great time in their terminal](/assets/typing.jpg)

[Or see it in action here!](https://asciinema.org/a/hEcf1pD2v60wUxiSIHdFWs5zN)

## Installing

Included are statically linked binaries for a few platforms (currently x86_64
Linux and macOS). To use them, download and execute like any regular binary.

[Releases here](https://gitlab.com/DarrienG/terminal-typeracer/tags)

## Running

```bash
$ typeracer
# Or if you want to take the passage from somewhere else
$ typeracer -r $(echo 'racing using a passage from elsewhere')
```

Hit ^C at any time to quit.

## Building
You need rust version 1.33.0 or higher (using some newer time APIs) and OpenSSL.
Generally the package you're looking for is libssl-dev

```bash
$ cargo build --release
```

The binary you'll get is called `typeracer` and runnable immediately!

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

## Contributing

I'd love to have more contributors! If you're looking to make any drastic
changes (e.g. Redis integration or something like that) consider contacting me
via email first so we can discuss.

All rust should be formatted with rustfmt. And if you're adding a new feature,
please add some tests too!

License is GPLv3 in the spirit of open source.
