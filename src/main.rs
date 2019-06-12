use clap;
use std::io::Error;

mod game;
mod lang;
mod term_check;
mod dirs {
    pub mod setup_dirs;
}

pub mod actions;
pub mod stats;

#[cfg(not(debug_assertions))]
fn debug_enabled_default() -> bool {
    false
}

#[cfg(debug_assertions)]
fn debug_enabled_default() -> bool {
    true
}

#[cfg(debug_assertions)]
fn get_version() -> &'static str {
    "DEBUG"
}

#[cfg(not(debug_assertions))]
fn get_version() -> &'static str {
    "1.0.8"
}

fn main() -> Result<(), Error> {
    let args = clap::App::new("Terminal typing game. Type through passages to see what the fastest times are you can get!")
        .version(get_version())
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
        .arg(
            clap::Arg::with_name("DEBUG_MODE")
            .short("d")
            .long("debug-mode")
            .required(false)
            .takes_value(false)
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

    let debug_enabled = args.is_present("DEBUG_MODE") || debug_enabled_default();

    let legacy_wpm = args.is_present("LEGACY_WPM");

    let stats = &mut stats::Stats::new(legacy_wpm);

    if term_check::resolution_check().is_ok() {
        if !lang::check_lang_pack() {
            let result = lang::retrieve_lang_pack();
            if result.is_err() {
                return result;
            }
        }
        while match game::play_game(&read_text, stats, debug_enabled) {
            actions::Action::Quit => false,
            actions::Action::NextPassage => true,
        } {
            read_text = "".to_string();
            stats.reset();
        }
    }
    Ok(())
}
