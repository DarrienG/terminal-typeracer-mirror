use config::TyperacerConfig;
use graphs::show_graphs;
use info::show_info;
use std::{
    collections::HashSet,
    fmt,
    io::stdout,
    sync::mpsc::{channel, Sender},
    time::Duration,
};
use termion::{event::Key, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, text::Span, Terminal};

use crate::{
    actions::Action, config, dirs::setup_dirs::get_db_path, graphs, info,
    passage_controller::PassageInfo, stats,
};

pub mod formatter;
pub mod indexer;
pub mod split;
pub mod word_processing;

mod game_db;
mod game_render;
mod input;

const TERRIBLE_DB_FAILURE: &str =
    "HELP - TROUBLE STORING DATA IN THE DB, CONTACT THE MAINTAINER AND SHOW THEM THIS ERROR:";

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum GameMode {
    Default,
    InstantDeath,
    Training,
}

impl GameMode {
    // This is supposed to work like a baby state machine/ringbuffer
    // All states should transition to a "next" state, and the states
    // should transition as if they are a ringbuffer
    // Next should always return a new value and never terminate.
    // note - this and prev should real use ordinal
    // There is a crate that provides this which we have opted not to use right now,
    // but if this continues to expand, we should add it in
    pub fn next(self) -> Self {
        match self {
            GameMode::Default => GameMode::InstantDeath,
            GameMode::InstantDeath => GameMode::Training,
            GameMode::Training => GameMode::Default,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            GameMode::Default => GameMode::Training,
            GameMode::Training => GameMode::InstantDeath,
            GameMode::InstantDeath => GameMode::Default,
        }
    }
}

impl fmt::Display for GameMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameMode::InstantDeath => write!(f, "Instant Death"),
            GameMode::Default => write!(f, "Default"),
            GameMode::Training => write!(f, "Training"),
        }
    }
}

// Used to convert to/from a num for sqlite
impl From<GameMode> for i64 {
    fn from(gm: GameMode) -> i64 {
        match gm {
            GameMode::Training => 2,
            GameMode::InstantDeath => 1,
            GameMode::Default => 0,
        }
    }
}
impl From<i64> for GameMode {
    fn from(i: i64) -> Self {
        match i {
            2 => GameMode::Training,
            1 => GameMode::InstantDeath,
            _ => GameMode::Default,
        }
    }
}

/// Event loop: Displays the typing input and renders keypresses.
/// This is the entrance to the main game.
// TODO: Provide get_backend method in game_render
pub fn play_game(
    passage_info: &PassageInfo,
    stats: &mut stats::Stats,
    debug_enabled: bool,
    game_mode: GameMode,
    typeracer_version: &str,
    typeracer_config: &TyperacerConfig,
) -> Action {
    let stdout = stdout()
        .into_raw_mode()
        .expect("Failed to manipulate terminal to raw mode");
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);
    let mut terminal = Terminal::new(backend).expect("Unable to get handle to terminal.");
    terminal.hide_cursor().expect("Failed to hide the cursor");

    let mut formatted_texts = formatter::FormattedTexts {
        passage: passage_info
            .passage
            .chars()
            .map(|it| Span::raw(it.to_string()))
            .collect(),
        input: vec![],
        error: false,
        complete: false,
    };

    let mut user_input = String::new();
    let mut mistaken_words: HashSet<String> = HashSet::new();

    // Split the passage into vec of words to work on one at a time
    let words: Vec<&str> = split::to_words(&passage_info.passage);
    let text_mode = word_processing::get_game_mode(&passage_info.passage);

    let mut current_word_idx = 0;

    let (input_sender, input_receiver) = channel::<Key>();
    let (quit_sender, quit_receiver) = channel::<bool>();
    input::capture(input_sender, quit_receiver);
    let mut got_first_input = false;

    loop {
        game_render::render(
            &mut terminal,
            game_render::GameState {
                texts: &formatted_texts,
                user_input: &user_input,
                stats,
                title: &passage_info.title,
                game_mode,
                config: typeracer_config,
                debug_enabled,
                word_idx: current_word_idx,
                passage_path: &passage_info.passage_path,
                complete: formatted_texts.complete,
                current_word: if current_word_idx == words.len() || formatted_texts.complete {
                    "DONE"
                } else {
                    words[current_word_idx]
                },
                mistaken_words: &mistaken_words,
            },
            typeracer_version,
        );
        if formatted_texts.complete {
            break;
        }

        // backspace and clearing the line are technically new chars, but shouldn't be
        // added to the combo. This lets us keep track of when when the user actually types
        // a new character (useful for combo).
        let mut new_char = false;

        // Last input char, required for determining if non-latin input is
        // set up properly for formatting.
        let mut last_input_char = ' ';

        let mut allowed_to_increment_combo = false;

        let recv_result = input_receiver.recv_timeout(Duration::from_millis(500));
        if recv_result.is_err() {
            if got_first_input {
                stats.update_wpm(current_word_idx, &words);
            }
            // just didn't get anything, let's keep going
            continue;
        }

        match recv_result.unwrap() {
            Key::Ctrl('a') => show_info(&mut terminal, typeracer_version),
            Key::Ctrl('c') => return perform_action(Action::Quit, &quit_sender),
            Key::Ctrl('n') => return perform_action(Action::NextPassage, &quit_sender),
            Key::Ctrl('p') => return perform_action(Action::PreviousPassage, &quit_sender),
            Key::Ctrl('r') => return perform_action(Action::RestartPassage, &quit_sender),
            Key::Ctrl('g') => show_graphs(&mut terminal, &get_db_path(), game_mode)
                .expect("Unable to get data for graph"),
            // Get some basic readline bindings
            Key::Ctrl('u') => user_input.clear(),
            Key::Ctrl('w') => {
                user_input = word_processing::get_all_input_minus_last_word(&user_input)
            }
            Key::Backspace | Key::Ctrl('h') => {
                user_input.pop();
            }
            Key::Char(c) => {
                got_first_input = true;
                new_char = true;
                last_input_char = c;
                stats.update_start_time();

                if word_processing::word_completed(
                    &text_mode,
                    last_input_char,
                    words[current_word_idx],
                    &user_input,
                ) {
                    if !typeracer_config.display_settings.always_full {
                        formatted_texts.passage = word_processing::get_updated_texts(
                            &text_mode,
                            formatted_texts.passage,
                            words[current_word_idx],
                        );
                    }
                    current_word_idx += 1;
                    user_input.clear();
                } else if c == '\n' || c == '\t' {
                    // Ignore a few types that can put the user in a weird spot
                    // We just want to ignore these characters.
                } else {
                    user_input.push(c);
                }

                stats.update_wpm(current_word_idx, &words);
                allowed_to_increment_combo = true;
            }
            _ => {}
        }

        formatted_texts = if current_word_idx >= words.len() {
            formatted_texts
        } else if typeracer_config.display_settings.always_full {
            formatter::get_formatted_texts(
                &text_mode,
                &words,
                &user_input.to_string(),
                current_word_idx,
                last_input_char,
                new_char,
                formatted_texts.passage,
            )
        } else {
            formatter::get_formatted_texts_line_mode(
                &text_mode,
                words[current_word_idx],
                &user_input.to_string(),
                last_input_char,
                new_char,
                formatted_texts.passage,
            )
        };

        let current_letter_idx =
            indexer::get_trying_letter_idx(&text_mode, &words, current_word_idx, &user_input);
        if formatted_texts.error && new_char {
            stats.increment_errors(current_letter_idx);

            // Additionally build the set of mistaken words
            mistaken_words.insert(words[current_word_idx].to_string());

            if game_mode == GameMode::InstantDeath {
                formatted_texts = formatter::get_reformatted_failed_texts(&text_mode, &words);
                continue;
            }
        } else if allowed_to_increment_combo {
            // there's no error, but we should only increment the combo if the customer didn't hit a control character
            // allowed_to_increment_combo will only be set to true if a "regular" non-control character is pressed
            stats.increment_combo(current_letter_idx);
        }

        if word_processing::decide_game_end(&text_mode, current_word_idx, &words, &user_input) {
            // Check to see if the user is on the last word and it is correct.
            // If it is, we need to do a little extra work to set the passage back to the full
            // passage. If the user is running with display_settings.always_max=false then they
            // will only see the last word.
            formatted_texts = formatter::get_reformatted_complete_texts(&text_mode, &words);
            current_word_idx += 1;
            stats.update_wpm(current_word_idx, &words);
            user_input.clear();
        }
    }

    if let Err(e) = game_db::store_stats(&get_db_path(), stats, passage_info, game_mode) {
        println!("{} {}", TERRIBLE_DB_FAILURE, e);
    }

    if game_mode == GameMode::Training {
        if let Err(e) = game_db::roll_to_delete_mistaken_words_typed_correctly(
            &get_db_path(),
            &words,
            &mistaken_words,
        ) {
            println!("{} {}", TERRIBLE_DB_FAILURE, e);
        }
    }

    if let Err(e) = game_db::store_mistaken_words(&get_db_path(), &mistaken_words) {
        println!("{} {}", TERRIBLE_DB_FAILURE, e);
    }

    loop {
        let recv_result = input_receiver.recv_timeout(Duration::from_millis(500));
        if recv_result.is_err() {
            // just didn't get anything, let's keep going
            continue;
        }
        match recv_result.unwrap() {
            Key::Ctrl('a') => {
                show_info(&mut terminal, typeracer_version);
                game_render::render(
                    &mut terminal,
                    game_render::GameState {
                        texts: &formatted_texts,
                        user_input: &user_input,
                        stats,
                        title: &passage_info.title,
                        game_mode,
                        config: typeracer_config,
                        debug_enabled,
                        complete: formatted_texts.complete,
                        word_idx: current_word_idx,
                        passage_path: &passage_info.passage_path,
                        current_word: if current_word_idx == words.len() {
                            "DONE"
                        } else {
                            words[current_word_idx]
                        },
                        mistaken_words: &mistaken_words,
                    },
                    typeracer_version,
                );
            }
            Key::Ctrl('c') => return perform_action(Action::Quit, &quit_sender),
            Key::Ctrl('n') => return perform_action(Action::NextPassage, &quit_sender),
            Key::Ctrl('p') => return perform_action(Action::PreviousPassage, &quit_sender),
            Key::Ctrl('r') => return perform_action(Action::RestartPassage, &quit_sender),
            Key::Ctrl('g') => {
                show_graphs(&mut terminal, &get_db_path(), game_mode)
                    .expect("Unable to get data for graph");
                game_render::render(
                    &mut terminal,
                    game_render::GameState {
                        texts: &formatted_texts,
                        user_input: &user_input,
                        stats,
                        title: &passage_info.title,
                        game_mode,
                        config: typeracer_config,
                        debug_enabled,
                        complete: formatted_texts.complete,
                        word_idx: current_word_idx,
                        passage_path: &passage_info.passage_path,
                        current_word: if current_word_idx == words.len() {
                            "DONE"
                        } else {
                            words[current_word_idx]
                        },
                        mistaken_words: &mistaken_words,
                    },
                    typeracer_version,
                );
            }
            _ => (),
        }
    }
}

fn perform_action(action: Action, sender: &Sender<bool>) -> Action {
    sender
        .send(true)
        .expect("Receiver thread died unexpectedly. Please restart typeracer");
    action
}
