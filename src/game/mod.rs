use config::TyperacerConfig;
use info::show_info;
use std::io::{stdin, stdout};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::widgets::Text;
use tui::Terminal;

use crate::actions::Action;
use crate::config;
use crate::info;
use crate::passage_controller::PassageInfo;
use crate::stats;

pub mod formatter;
pub mod indexer;

mod game_render;

/// Event loop: Displays the typing input and renders keypresses.
/// This is the entrance to the main game.
// TODO: Provide get_backend method in game_render
pub fn play_game(
    passage_info: &PassageInfo,
    stats: &mut stats::Stats,
    debug_enabled: bool,
    instant_death: bool,
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
            .map(|it| Text::raw(it.to_string()))
            .collect(),
        input: vec![],
        error: false,
        complete: false,
    };

    let mut user_input = String::new();

    // Split the passage into vec of words to work on one at a time
    let words: Vec<&str> = passage_info.passage.split(' ').collect();
    let mut current_word_idx = 0;

    loop {
        game_render::render(
            &mut terminal,
            game_render::GameState {
                texts: &formatted_texts,
                user_input: &user_input,
                stats,
                title: &passage_info.title,
                instant_death,
                config: typeracer_config,
                debug_enabled,
                word_idx: current_word_idx,
                passage_path: &passage_info.passage_path,
                current_word: if current_word_idx == words.len() {
                    "DONE"
                } else {
                    words[current_word_idx]
                },
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

        let stdin = stdin();
        let c = stdin.keys().find_map(Result::ok);
        match c.unwrap() {
            Key::Ctrl('a') => show_info(&mut terminal, typeracer_version),
            Key::Ctrl('c') => return Action::Quit,
            Key::Ctrl('n') => return Action::NextPassage,
            Key::Ctrl('p') => return Action::PreviousPassage,
            Key::Ctrl('r') => return Action::RestartPassage,
            // Get some basic readline bindings
            Key::Ctrl('u') => user_input.clear(),
            Key::Backspace => {
                user_input.pop();
            }
            Key::Char(c) => {
                new_char = true;
                stats.update_start_time();

                if c == ' ' && (words[current_word_idx] == user_input) {
                    if !typeracer_config.display_settings.always_full {
                        formatted_texts.passage = formatted_texts.passage
                            [words[current_word_idx].len() + 1..formatted_texts.passage.len()]
                            .to_vec();
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
            }
            _ => {}
        }

        formatted_texts = if typeracer_config.display_settings.always_full {
            formatter::get_formatted_texts(
                &words,
                &user_input.to_string(),
                current_word_idx,
                formatted_texts.passage,
            )
        } else {
            formatter::get_formatted_texts_line_mode(
                &words[current_word_idx],
                &user_input.to_string(),
                formatted_texts.passage,
            )
        };

        let current_letter_idx =
            indexer::get_trying_letter_idx(&words, current_word_idx, &user_input);
        if formatted_texts.error && new_char {
            stats.increment_errors(current_letter_idx);
            if instant_death {
                formatted_texts = formatter::get_reformatted_failed_texts(&words);
                continue;
            }
        } else {
            stats.increment_combo(current_letter_idx);
        }

        if current_word_idx + 1 == words.len() && (words[current_word_idx] == user_input) {
            // Check to see if the user is on the last word and it is correct.
            // If it is, we need to do a little extra work to set the passage back to the full
            // passage. If the user is running with display_settings.always_max=false then they
            // will only see the last word.
            formatted_texts = formatter::get_reformatted_complete_texts(&words);
            current_word_idx += 1;
            stats.update_wpm(current_word_idx, &words);
            user_input.clear();
        }
    }

    loop {
        let stdin = stdin();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Ctrl('a') => {
                    show_info(&mut terminal, typeracer_version);
                    game_render::render(
                        &mut terminal,
                        game_render::GameState {
                            texts: &formatted_texts,
                            user_input: &user_input,
                            stats,
                            title: &passage_info.title,
                            instant_death,
                            config: &typeracer_config,
                            debug_enabled,
                            word_idx: current_word_idx,
                            passage_path: &passage_info.passage_path,
                            current_word: if current_word_idx == words.len() {
                                "DONE"
                            } else {
                                words[current_word_idx]
                            },
                        },
                        typeracer_version,
                    );
                }
                Key::Ctrl('c') => return Action::Quit,
                Key::Ctrl('n') => return Action::NextPassage,
                Key::Ctrl('p') => return Action::PreviousPassage,
                Key::Ctrl('r') => return Action::RestartPassage,
                _ => (),
            }
        }
    }
}
