# Typeracer

[![pipeline status](https://gitlab.com/DarrienG/terminal-typeracer/badges/master/pipeline.svg)](https://gitlab.com/DarrienG/terminal-typeracer/commits/master)

An open source terminal based version of Typeracer written in rust.

Gives you a random passage and you type it out. While you type it will tell you
where you're making errors and give you a set of words per minute.

![User typing away having a great time in their terminal](/assets/typing.jpg)

[Or see it in action here!](https://asciinema.org/a/hEcf1pD2v60wUxiSIHdFWs5zN)

## Installing

We're on crates.io! Grab the latest version with:

```bash
$ cargo install typeracer
```

Or if you prefer, binaries are included with each release.

Binaries are statically linked and available for a few platforms (currently
x86_64 Linux and macOS). To use them, download and execute like any regular
binary.

[Releases here](https://gitlab.com/DarrienG/terminal-typeracer/tags)

## Running

```bash
$ typeracer
# Or if you want to take the passage from somewhere else
$ typeracer -r $(echo 'racing using a passage from elsewhere')
```

Hit ^C at any time to quit.

## Configuration

What good would a typing game be without a config file?

Where you can find your config file:

Linux:
```
~/.config/typeracer/config.toml
```

And roughly in:

macOS:
```
$HOME/Library/Preferences
```

Windows;

```
{FOLDERID_RoamingAppData}
```

-- Note typeracer uses whatever the proper mechanism is for data and config
folders are for your OS. If you customized the variables used, it may be
elsewhere.

With your config, you can enable or disable language packs.

```toml
[lang_packs]
whitelisted = ["default"]
```

```toml
[lang_packs]
blacklisted = ["harry-potter"]
```

You can also choose where to get language packs. Default langpack is
`https://gitlab.com/ttyperacer/lang-packs.git`, you can override this in the
configuration file using `repo` key. You can also specify the version with
repo_version.

For all lang pack versions, see [here!][https://gitlab.com/ttyperacer/lang-packs/-/branches] Bear in mind there may be incompatibilities between older lang packs and newer typeracers and the other way around.

```toml
repo = "https://example.com/your-lang-pack.git"
repo_version = "lang-0.2"
```

## Building
You need rust version 1.33.0 or higher (using some newer time APIs) and git. If
you're on macOS, you'll probably need to install openssl too.

```bash
$ cargo build --release
```

The binary you'll get is called `typeracer` and runnable immediately!

### Cross compile is currently broken
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
