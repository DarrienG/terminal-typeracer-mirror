# Configuration

Example at the bottom!

What good would a terminal app be without the ability to configure it?

We try to configure Typeracer with sane defaults, but there are cases where you
may want to change how it runs.

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

Windows:

```
{FOLDERID_RoamingAppData}
```

-- Note typeracer uses whatever the proper mechanism is for data and config
folders are for your OS. e.g. XDG_DIRS on Linux. If you customize your XDG_DIRS,
it will respect that and may be elsewhere.

## Configuration

Parameters you can configure in the default namespace:

`repo` = where your lang packs are located
* default: `https://gitlab.com/ttyperacer/lang-packs.git`
* For more information on the lang-pack format, see
    [here](https://gitlab.com/ttyperacer/terminal-typeracer/tree/master/docs/lang-pack-format.md)

`repo_version` = which version of the lang pack to use
* default: The recommended version compiled with the program

`history_size` = number of previous passages to remember during runtime
* default: `20`
* This buffer is dynamically allocated, so memory usage will not balloon on
    start if this is set to a high number
* The content of passages is stored in a buffer in history though, so a long
    runtime with a huge history may use more memory than expected.

Parameters you can configure in the `[lang_packs]` namespace:

`whitelisted` = Takes a list, enabled lang packs
* default: Everything

`blacklisted` = Takes a list, disabled lang packs
* default: Nothing

blacklisted and whitelisted cannot both be filled out at the same time.

You can see what is and isn't enabled, and what is and isn't available with the
`-s` flag: `typeracer -s`

Parameters you can configure in the `[display_settings]` namespace:

`always_full` = Decide whether or not to show the full passage at all times.
* default: `false`
* With this set to false, after a word is successfully typed, it will disappear
    to make room for the rest of the words.
* Setting to true sets the behavior back to the way it was in version `<=1.2.3`

`simple_borders` = Decide whether or not to color borders in accordance with
events
* default: `false`
* With this set to true, the border colors will stay constant over the course of
    the game.
* Things that currently will change border colors mid game:
    * Missing a single letter in regular mode
    * Getting a combo (consecutively typing letters correctly) > 60
* Setting to false ensures the game borders never change once started


## Example config

Assuming you want to customize everything, a fully configured file might look
like this:

```toml
repo = "https://gitlab.com/ttyperacer/lang-packs.git"
repo_version = "lang-0.2"
history_size = 50

[lang_packs]
blacklisted = ["default"]

[display_settings]
always_full = true
simple_borders = true
```

Remember though that the config file is entirely optional and all parameters are
also optional.
