#![cfg_attr(test, allow(dead_code, unused_imports))]
use clap;
use std::io::Error;

mod game;
mod lang_pack;
mod passage_controller;
mod term_check;
mod dirs {
    pub mod setup_dirs;
}

pub mod actions;
pub mod stats;

use actions::Action;

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
    "1.1.1"
}

fn get_lang_pack_version() -> &'static str {
    "lang-0.2"
}

fn main() -> Result<(), Error> {
    let args = clap::App::new("Terminal typing game. Type through passages to see what the fastest times are you can get!")
        .version(&*format!("Typeracer version: {}, lang pack version: {}", get_version(), get_lang_pack_version()))
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

    let mut passage_controller = passage_controller::Controller::new(20);
    // Get user input text and strip out characters that are difficult to type
    if args.is_present("READ_TEXT") {
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
        passage_controller
            .write_initial_passage(&constructed_string[0..constructed_string.chars().count() - 1]);
    }

    let debug_enabled = args.is_present("DEBUG_MODE") || debug_enabled_default();

    let legacy_wpm = args.is_present("LEGACY_WPM");

    let stats = &mut stats::Stats::new(legacy_wpm);

    if term_check::resolution_check().is_ok() {
        if !lang_pack::check_lang_pack(get_lang_pack_version()) {
            let result = lang_pack::retrieve_lang_pack(get_lang_pack_version());
            match result {
                Err(e) => return Err(e),
                Ok(false) => return Ok(()),
                Ok(true) => (),
            }
        }

        let mut action = Action::NextPassage;
        while action != actions::Action::Quit {
            action = game::play_game(
                passage_controller.retrieve_passage(action),
                stats,
                debug_enabled,
            );
            stats.reset();
        }
    }
    Ok(())
}
