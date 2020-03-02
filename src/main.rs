#![cfg_attr(test, allow(dead_code, unused_imports))]
use clap;
use std::io::Error;

mod game;
mod lang_pack;
mod passage_controller;
mod dirs {
    pub mod setup_dirs;
}

pub mod actions;
pub mod config;
pub mod info;
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
const VERSION: &str = "DEBUG";

#[cfg(not(debug_assertions))]
const VERSION: &str = "1.5.0";

fn main() -> Result<(), Error> {
    let typeracer_config = config::get_config();
    let args = clap::App::new("Terminal typing game. Type through passages to see what the fastest times are you can get!")
        .version(&*format!("Typeracer version: {}, lang pack version: {}", VERSION, typeracer_config.repo_version))
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
            .help("Run with debug info")
        )
        .arg(
            clap::Arg::with_name("SHOW_PACKS")
            .short("s")
            .long("show-packs")
            .required(false)
            .takes_value(false)
            .help("Show all currently available language packs")
        )
        .arg(
            clap::Arg::with_name("INSTANT_DEATH")
            .short("i")
            .long("instant-death")
            .required(false)
            .takes_value(false)
            .help("Play with instant death mode. One mistype and you lose!")
        )
        .get_matches();

    let mut passage_controller =
        passage_controller::Controller::new(typeracer_config.history_size, &typeracer_config);

    if args.is_present("SHOW_PACKS") {
        let (filtered_dirs, all_dirs) = passage_controller.get_quote_dir_shortnames();

        println!("Enabled packs:\t{}", filtered_dirs.join(", "));
        println!("All packs:\t{}", all_dirs.join(", "));
        return Ok(());
    }

    // Get user input text and strip out characters that are difficult to type
    if args.is_present("READ_TEXT") {
        let mut constructed_string = "".to_owned();
        let input = args.values_of("READ_TEXT").unwrap();

        for word in input {
            if word == " " || word == "\n" {
                continue;
            } else {
                constructed_string.push_str(word.trim());
                constructed_string.push_str(" ");
            }
        }
        passage_controller
            .write_initial_passage(&constructed_string[0..constructed_string.chars().count() - 1]);
    }

    let debug_enabled = args.is_present("DEBUG_MODE") || debug_enabled_default();

    let legacy_wpm = args.is_present("LEGACY_WPM");

    let instant_death = args.is_present("INSTANT_DEATH");

    let stats = &mut stats::Stats::new(legacy_wpm);

    if !lang_pack::check_lang_pack(&typeracer_config.repo_version) {
        let result =
            lang_pack::retrieve_lang_pack(&typeracer_config.repo_version, &typeracer_config);
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
            instant_death,
            VERSION,
            &typeracer_config,
        );
        stats.reset();
    }

    Ok(())
}
