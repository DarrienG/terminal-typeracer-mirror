use clap;
use std::io::Error;

mod game;
mod lang;
mod term_check;
mod dirs {
    pub mod setup_dirs;
}

pub mod actions;

fn main() -> Result<(), Error> {
    let args = clap::App::new("Terminal typing game. Type through passages to see what the fastest times are you can get!")
        .version("1.0.4")
        .author("Darrien Glasser <me@darrien.dev>")
        .setting(clap::AppSettings::TrailingVarArg)
        .arg(
            clap::Arg::with_name("READ_TEXT")
            .short("r")
            .long("read-text")
            .multiple(true)
            .required(false)
            .takes_value(true)
            .help("Read passage as an arg rather than from local set of passages.")
        )
        .arg(
            clap::Arg::with_name("LEGACY_WPM")
            .short("l")
            .long("legacy-wpm")
            .required(false)
            .takes_value(false)
            .help("Derive words per minute as actual words/minute instead of letters/5 over minute")
        )
        .get_matches();

    // Get user input text and strip out characters that are difficult to type
    let mut read_text = if args.is_present("READ_TEXT") {
        let mut constructed_string = "".to_owned();
        let input = args.values_of("READ_TEXT").unwrap();

        for word in input {
            if word == " " || word == "\n" {
                continue;
            } else {
                constructed_string.push_str(word);
                constructed_string.push_str(" ");
            }
        }
        (&constructed_string[0..constructed_string.chars().count() - 1]).to_string()
    } else {
        "".to_string()
    };

    let legacy_wpm = args.is_present("LEGACY_WPM");

    if !term_check::resolution_check().is_err() {
        if !lang::check_lang_pack() {
            let result = lang::retrieve_lang_pack();
            if result.is_err() {
                return result;
            }
        }
        while match game::play_game(&read_text, legacy_wpm) {
            actions::Action::Quit => false,
            actions::Action::NextPassage => true,
        }{ read_text = "".to_string(); }
    }
    Ok(())
}
