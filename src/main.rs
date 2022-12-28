#![cfg_attr(test, allow(dead_code, unused_imports))]
#![allow(clippy::match_like_matches_macro)]
use clap::{Arg, Command};
use crossbeam_channel::unbounded;
use std::io::{Error, ErrorKind};

#[cfg(debug_assertions)]
#[cfg(feature = "git-versioning")]
use git_version::git_version;

mod game;
mod input;
mod lang_pack;
mod passage_controller;
mod dirs {
    pub mod setup_dirs;
}

pub mod actions;
pub mod config;
pub mod db;
pub mod graphs;
pub mod info;
pub mod stats;

use actions::Action;
use rusqlite::Connection;
use termion::event::Key;

#[cfg(not(debug_assertions))]
fn debug_enabled_default() -> bool {
    false
}

#[cfg(debug_assertions)]
fn debug_enabled_default() -> bool {
    true
}

#[cfg(not(debug_assertions))]
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(debug_assertions)]
const VERSION: &str = "DEBUG";

fn main() -> Result<(), Error> {
    let typeracer_config = config::get_config();
    let args = Command::new("Terminal typing game. Type through passages to see what the fastest times are you can get!")
        .version(&*format!("Typeracer version: {}, lang pack version: {}", VERSION, typeracer_config.repo_version))
        .author("Darrien Glasser <me@darrien.dev>")
        .trailing_var_arg(true)
        .arg(
            Arg::new("READ_TEXT")
            .short('r')
            .long("read-text")
            .multiple_values(true)
            .required(false)
            .takes_value(true)
            .help("Read passage as an arg rather than from local set of passages.")
        )
        .arg(
            Arg::new("LEGACY_WPM")
            .short('l')
            .long("legacy-wpm")
            .required(false)
            .takes_value(false)
            .help("Derive words per minute as actual words/minute instead of letters/5 over minute")
        )
        .arg(
            Arg::new("DEBUG_MODE")
            .short('d')
            .long("debug-mode")
            .required(false)
            .takes_value(false)
            .help("Run with debug info")
        )
        .arg(
            Arg::new("SHOW_PACKS")
            .short('s')
            .long("show-packs")
            .required(false)
            .takes_value(false)
            .help("Show all currently available language packs")
        )
        .arg(
            Arg::new("INSTANT_DEATH")
            .short('i')
            .long("instant-death")
            .required(false)
            .takes_value(false)
            .help("Play with instant death mode. One mistype and you lose!")
        )
        .arg(
            Arg::new("TRAINING")
            .short('t')
            .long("training")
            .required(false)
            .takes_value(false)
            .help("Play in training mode. All the words you typed wrong are back to haunt you!")
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
        let input = args.values_of("READ_TEXT").unwrap();
        passage_controller.write_initial_passage(&input.collect::<Vec<&str>>().join(" "));
    }

    let debug_enabled = args.is_present("DEBUG_MODE") || debug_enabled_default();

    let legacy_wpm = args.is_present("LEGACY_WPM");

    let game_mode = if args.is_present("INSTANT_DEATH") {
        game::GameMode::InstantDeath
    } else if args.is_present("TRAINING") {
        game::GameMode::Training
    } else {
        game::GameMode::Default
    };

    let stats = &mut stats::Stats::new(legacy_wpm);

    if !lang_pack::check_lang_pack(&typeracer_config) {
        let result =
            lang_pack::retrieve_lang_pack(&typeracer_config.repo_version, &typeracer_config);
        match result {
            Err(e) => return Err(e),
            Ok(false) => return Ok(()),
            Ok(true) => (),
        }
    }
    if !db::check_stats_db() {
        match db::create_database(&db::db_path(&dirs::setup_dirs::get_db_dir())) {
            Ok(_) => (),
            Err(e) => return Result::Err(Error::new(ErrorKind::ConnectionRefused, e)),
        }
    }
    if !db::check_for_migration(&db::db_path(&dirs::setup_dirs::get_db_dir())) {
        match db::do_migration(&db::db_path(&dirs::setup_dirs::get_db_dir())) {
            Ok(_) => (),
            Err(e) => return Result::Err(Error::new(ErrorKind::ConnectionRefused, e)),
        }
    }

    // setup input
    let (input_sender, input_receiver) = unbounded::<Key>();
    input::capture(input_sender);

    let mut action = Action::NextPassage;

    while action != actions::Action::Quit {
        let mistaken_words_passage = match passage_controller.retrieve_mistaken_words_passage(
            &Connection::open(db::db_path(&dirs::setup_dirs::get_db_dir())).unwrap(),
        ) {
            Ok(p) => p,
            _ => {
                return Result::Err(Error::new(
                    ErrorKind::NotFound,
                    "Couldn't get mistaken words from database!",
                ))
            }
        };

        let passage_info = match game_mode {
            game::GameMode::Training => &mistaken_words_passage,
            _ => passage_controller.retrieve_passage(action),
        };

        action = game::play_game(
            passage_info,
            &input_receiver,
            stats,
            debug_enabled,
            game_mode,
            VERSION,
            &typeracer_config,
        );
        stats.reset();
    }

    Ok(())
}
