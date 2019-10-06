# Lang Pack Format

The language pack format is pretty boring and more or less just a set of files.

It expects a folder to be in the "proper" data directory for your operating
system called `lang-packs`

The location of where it will be differs based on your OS.

Linux:

```
 ~/.local/share/typeracer/lang-packs
```

And roughly in:

macOS:

```
$HOME/Library/Application Support
```

Windows:

```
{FOLDERID_RoamingAppData}
```

-- Note typeracer uses whatever the proper mechanism is for data and config
folders are for your OS. e.g. XDG_DIRS on Linux. If you customize your XDG_DIRS,
it will respect that and may be elsewhere.

## Directory structure

```bash
lang-packs [remotes/origin/lang-0.3] tree -La 1
.
├── default
├── .git
├── harry-potter
└── version

3 directories, 1 file
```

## Root directory

Let's break it down. `default` and `harry-potter` are just arbitrary folders.
They each contain an inordinate amount of text files. We'll get into what's
inside these later.

There could be any number of these folders. Typeracer does not care how many
folders that contain text files you have.

At least one folder with one quote in it is required.

There's the `version` file. This displays what version your files are on, and is
included so that we don't need to enforce git usage with language packs. It is
required.

It only has two lines. Version, and a random uuid:

```
lang-0.3
c725044b-f4af-45d6-8cc0-dcc7e48fa999
```

Finally there is the `.git` folder. This is optional, and only required if you
want to let Typeracer auto upgrade and switch versions for you. The default lang
pack is managed using git.

## Inside a quote directory

Also not very exciting. If you cd into a quote directory provided by one of the
default lang packs and `ls`, you'll see a bunch of uuids. All files are more or
less randomly named. They can be named anything though, Typeracer doesn't care.

Open up a file though and you'll also see it's not very exciting. File are _not_
serialized using a format like JSON, they are just two lines. The passage, and
the attributor.

```
default [remotes/origin/lang-0.3] cat 0022791a-573f-414b-954f-7582f5bcd09e
The best is the enemy of the good.
Voltaire%
```

Note the % is there because there is no trailing newlines. You can have extra
newlines if you like though. As usual, Typeracer doesn't care. Your file must
have two lines though, otherwise it will be ignored, and a placeholder fallback
passage will be used instead.
