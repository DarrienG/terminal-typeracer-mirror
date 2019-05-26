# Typeracer

An open source terminal based version of Typeracer written in rust.

Gives you a random passage and you type it out. While you type it will tell you
where you're making errors and give you a set of words per minute.

![User typing away having a great time in their terminal](/assets/typing.jpg)

![Or see it in action here!](https://asciinema.org/a/248451)

Later I'll post some binaries here too.

## Running

```
$ typeracer
# Or if you want to take the passage from somewhere else
$ typeracer -r $(echo 'racing using a passage from elsewhere')
```

Hit ^C at any time to quit. If you don't like the passage you're typing out,
hit ^N (next) for another passage!

## Building
You need rust version 1.33.0 or higher (using some newer time APIs) and OpenSSL.
Generally the package you're looking for is libssl-dev

```
$ cargo build --release
```

The binary you'll get is called `typeracer` and runnable immediately!

## Contributing

I'd love to have more contributors! This is my first "real" not hello world
program in rust.

License is GPLv3 in the spirit of open source.

## TODO

Detect last word no space
