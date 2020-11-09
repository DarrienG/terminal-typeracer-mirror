# Terminal Typeracer quickstart and doc router

* Typeracer is configurable, for full info about the config file check
  [here](docs/config.md)
* For info on supported languages and alternative language packs, go
  [here](docs/supported-languages.md)
* If you're a dev working on typeracer, build instructions are
  [here](docs/building.md)
* If you're interested in learning about the data format typeracer uses to store
  data, go [here](docs/lang-pack-format.md)

## FAQ

### I don't like how text disappears when I type, how do I make it stop?

In your config file at the very bottom, add this section:

```toml
[display_settings]
always_full = true
```

Config file is fully documented [here](https://gitlab.com/ttyperacer/terminal-typeracer/tree/master/docs/config.md)

### Where's my config file?

```
# Linux
$HOME/.config/typeracer/config.toml

# macOS:
$HOME/Library/Preferences/org.darrienglasser.com.typeracer/config.toml
```

Config file is fully documented [here](https://gitlab.com/ttyperacer/terminal-typeracer/tree/master/docs/config.md)

### Some passages are too long and hard to type, can I disable them?

Run `typeracer -s` to see all of your currently enabled and synced langauge
packs. You should see:

```
Enabled packs:	difficult, easy, medium, very_difficult
All packs:	difficult, easy, medium, very_difficult
```

If you'd like to disable the more difficult ones, add this section in your
cofig:

```
[lang_packs]
blacklisted = ["difficult", "very_difficult"]
```

When you run `typeracer -s` again you'll see those two language packs are now
disabled.

Config file is fully documented [here](https://gitlab.com/ttyperacer/terminal-typeracer/tree/master/docs/config.md)

### I want to add another language to typeracer

The process isn't too hard, but it's too long to put here, go
[here](docs/supported-languages.md) for the full instructions.

## Here you'll find information on

* Building Typeracer from source
* Using the Typeracer config file to customize your Typeracer
* Information on the lang_pack format if you'd like to use your own packs
