# Supported languages

Typeracer natively supports almost all languages. If the language does not blend
wide and thin character charsets it should work as expected without a hitch.

Even though we support almost all languages, we only have language packs for a
few languages.

Language pack support is broken into three tiers.

* **Tier 1**: any lang packs in the official repos with more than 150 quotes in
  multiple categories.
* **Tier 2**: any lang packs in the official repos with less than 150 quotes.
* **Tier 3**: any lang packs outside the official repos.
  * If a tier 3 repo has over 150 built in quotes, it is granted the tier 3+
    moniker.

## Tier 1

* [English](https://gitlab.com/ttyperacer/lang-packs)
  * This is our primary language pack of over 6500 quotes. It is used by default
    in all typeracer installations.

## Tier 2

* [Chinese](https://gitlab.com/ttyperacer/extra-packs/chinese-pack)
  * A simple set of a few Chinese quotes. We could use more :)

* [English (Legacy)](https://gitlab.com/ttyperacer/extra-packs/legacy-pack)
  * The old typeracer packs you used to know and love. These are read only. New
    English contributions should go to the primary English language pack.

# How to contribute new languages and packs

We'd love all the help we can get! Whether you're interested in adding more
passages to the English repos, or looking to expand our collection of other
languages, we'd love to have them!

Typeracer's extensible by default nature also means if you would prefer not to
commit your own language packs to our repos, you are free to do so as well
(although we'd love to have them).

## Quickstart

0. (optional-a) If your language does not exist in the official repos and you'd
   like to add it, request a repo to be made in the official typeracer repo
   [here](https://gitlab.com/ttyperacer/terminal-typeracer/-/issues).
0. (optional-b) If you are not planning on contributing to the typeracer repo,
   make a publicly hosted, https accessible repo somewhere.
1. Set up your new repo in accordance with the [lang pack
   specs](lang-pack-format.md). This will briefly be described below.

### new lang pack folder setup
In your the root of your new git repo, make a file called `version` with 2
lines, a version and a random identifier. For instance:

```
1.0.0
7d3a01f8-69a2-4ca8-a4d0-792394ab8021
```

Afterwards, make one folder, your new quotes will go in there. The more folders
you add, the more configurable your quotes will be. You can add as many as you
like. For now we will make one called `fables`

Inside `fables` you may put as many quote files as you like. Files may be named
whatever you like (official repos use UUIDs). Let's make one called `the-angel`.

In this file we need 2 lines. One with the text the user will type out, and the
author. Inside our `the-angel` file we will have the text:

```
Close by grew a slender, beautiful, rose-bush, but some wicked hand had broken the stem, and the half-opened rosebuds hung faded and withered on the trailing branches.
Aesop
```

All lines after the first 2 lines will be ignored.

You may make as many files and folders as you like, however it is important no
files other than the ones above specified are added and all folders only have
valid quotes in them.

Once you've completed that, let's move onto the next step.

2. Make a branch with the same name as the version specified in the version file
   discussed above. In this case, we will be making a branch called 1.0.0 `git
   checkout -b 1.0.0`[^1]
3. Commit and push your changes to the remote repository.
4. Add the new lang packs to your config file. The full config file is
   documented [here](docs/config.md) but this will briefly be discussed here.

### adding the new lang pack to your config file

Open your typeracer config file, located in either of these two places based on
your OS:


```
# Linux
$HOME/.config/typeracer/config.toml

# macOS:
$HOME/Library/Application Support/org.darrienglasser.com.typeracer/config.toml
```

Add your new repo in a section like so:

```toml
extra_repos = [
  { name = "aesop", url = "https://gitlab.com/your-lang-pack.git", version = "1.0.0" }
]
```

Run `typeracer` once to sync the new language packs. You can quit right
afterwards.

If you would like _only_ your lang pack to be enabled, first check enabled lang
packs with `typeracer -s`

```
$ typeracer -s
Enabled packs:	aesop/fables, difficult, easy, medium, very_difficult
All packs:	aesop/fables, difficult, easy, medium, very_difficult
```

Then add this section below your `extra_repos` section to enable just your new
lang pack.

```toml
[lang_packs]
whitelisted = ["aesop/fables"]
```

5. Open up typeracer and it will ask if you'd like to sync the new lang packs.
   After this, you're done!

***

[^1]: It is assumed this branch is readonly. If you want new changes in the
branch, it is recommended to make a new branch, e.g. 1.0.1 and push new changes
to that.
