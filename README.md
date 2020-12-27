# Typeracer

[![pipeline status](https://gitlab.com/DarrienG/terminal-typeracer/badges/master/pipeline.svg)](https://gitlab.com/DarrienG/terminal-typeracer/commits/master)
[![coverage report](https://gitlab.com/ttyperacer/terminal-typeracer/badges/master/coverage.svg)](https://gitlab.com/ttyperacer/terminal-typeracer/-/commits/master)
![Development status](https://img.shields.io/badge/<Dev Status>-<Maintenance Mode>-<orange>.svg)
![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)

An open source terminal based version of Typeracer written in rust.

Gives you a random passage and you type it out. While you type it will tell you
where you're making errors and give you a set of words per minute.

![User typing away having a great time in their terminal](/assets/typing.jpg)

[Or see it in action here!](https://asciinema.org/a/290136)

## Installing

We're on crates.io! Grab the latest version with:

```bash
$ cargo install typeracer
```

Or if you prefer, binaries are included with each release.

Binaries are statically linked and available for a few platforms (currently
x86_64 Linux and macOS). To use them, download and execute like any regular
binary.

Windows is not supported if you are not running Windows Subsystem for Linux. If
you would like to run on Windows, install WSL and use the Linux binaries.

[Releases here](https://gitlab.com/DarrienG/terminal-typeracer/tags)

## Running

```bash
$ typeracer
# Or if you want to take the passage from somewhere else
$ typeracer -r $(echo 'racing using a passage from elsewhere')
```

Hit ^C at any time to quit.

## Language support

Typeracer natively supports reading almost all languages. If the language does
not blend wide and thin character charsets it should work as expected without a
hitch.

The primary language our lang packs support is English, but we would love
contributions for other languages :)

For more info on which languages we support in our lang packs and how to
contribute, please check [here](docs/supported-languages.md).

## Configuration

What good would a typing game be without a config file?

You can find docs on configuring [here](https://gitlab.com/ttyperacer/terminal-typeracer/tree/master/docs/config.md).

## Building

There isn't much to building, but steps are documented [here](https://gitlab.com/ttyperacer/terminal-typeracer/tree/master/docs/building.md).

## Contributing

I'd love to have more contributors! If you're looking to make any drastic
changes (e.g. Redis integration or something like that) consider contacting me
via email first so we can discuss.

All rust should be formatted with rustfmt. And if you're adding a new feature,
please add some tests too!

License is GPLv3 in the spirit of open source.
