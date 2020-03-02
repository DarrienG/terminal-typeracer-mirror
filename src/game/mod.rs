use config::TyperacerConfig;
use info::show_info;
use std::io::{stdin, stdout};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::style::Color;
use tui::style::Style;
use tui::widgets::Text;
use tui::Terminal;

use crate::actions::Action;
use crate::config;
use crate::info;
use crate::passage_controller::PassageInfo;
use crate::stats;

pub mod game_render;

#[derive(Debug, Clone)]
pub struct FormattedTexts<'a> {
    pub passage: Vec<Text<'a>>,
    pub input: Vec<Text<'a>>,
    pub error: bool,
    pub complete: bool,
}

/// Determine if two words are the same.
fn check_word(word: &str, input: &str) -> bool {
    *word == *input
}

/// Check to see if the "input" is like the word. This is effectively
/// word.contains(input) but only if the first input.len characters are
/// the same. e.g. apple, ap => true, apple ppl => false
fn check_like_word(word: &str, input: &str) -> bool {
    if input.is_empty() {
        return true;
    }
    if input.len() > word.len() {
        return false;
    }

    check_word(&word[..input.len()], input)
}

/// Get formatted version of a single word in a passage and the user's current input.
///
/// All similar characters up until the first different character are highlighted with green.
///
/// On an erroroneous character:
/// - The first error character in the passage's word is highlighted with red and the rest unformatted.
/// - The entirety of the user's input is colored red.
///
/// Returns a tuple with the formatted version of the: word and the input.
fn get_formatted_words<'a>(word: &str, input: &str) -> (Vec<Text<'a>>, Vec<Text<'a>>) {
    let indexable_word: Vec<char> = word.chars().collect();
    let indexable_input: Vec<char> = input.chars().collect();
    let idx_word_count = indexable_word.len();
    let idx_input_count = indexable_input.len();

    let mut formatted_word: Vec<Text> = Vec::new();
    let mut formatted_input: Vec<Text> = Vec::new();
    let mut word_dex = 0;

    let err = !check_like_word(word, input);

    // Make all of the user's input white on red
    for input in indexable_input.iter() {
        if err {
            formatted_input.push(Text::styled(
                input.to_string(),
                Style::default().bg(Color::Red).fg(Color::White),
            ));
        } else {
            formatted_input.push(Text::styled(
                input.to_string(),
                Style::default().fg(Color::Green),
            ));
        }
    }

    formatted_input.push(Text::styled(" ", Style::default().bg(Color::Blue)));

    while word_dex < idx_word_count && word_dex < idx_input_count {
        if indexable_word[word_dex] != indexable_input[word_dex] {
            break;
        }

        formatted_word.push(Text::styled(
            indexable_word[word_dex].to_string(),
            Style::default().fg(Color::Green),
        ));
        word_dex += 1;
    }

    let mut first = true;
    // Show the first error the user makes in the passage they are typing
    for word in indexable_word.iter().skip(word_dex).take(idx_word_count) {
        if first {
            if err {
                formatted_word.push(Text::styled(
                    word.to_string(),
                    Style::default().bg(Color::Red).fg(Color::White),
                ));
            } else {
                formatted_word.push(Text::styled(
                    word.to_string(),
                    Style::default().bg(Color::Blue).fg(Color::White),
                ));
            }
            first = false;
            continue;
        }
        formatted_word.push(Text::raw(word.to_string()));
    }

    (formatted_word, formatted_input)
}

/// Given a vector of word and the current index of the word the user is typing,
/// ["this", "is", "a", "vector"] and current_word_idx of 2,
/// return the index as if we were indexing the previous vector as a space
/// separated string to get the first character of the word the user is
/// currently on.
/// In this case, we would get 8 back.
/// "this is a vector"
/// ---------^
fn get_starting_idx(words: &[&str], current_word_idx: usize) -> usize {
    let mut passage_starting_idx: usize = 0;
    for word in words.iter().take(current_word_idx) {
        passage_starting_idx += word.chars().count() + 1
    }
    passage_starting_idx
}

fn get_trying_letter_idx(words: &[&str], current_word_idx: usize, user_input: &str) -> usize {
    let starting_idx = get_starting_idx(words, current_word_idx);

    let mut letter_on = 0;

    if user_input.is_empty() {
        return starting_idx;
    }

    let user_in_chars: Vec<char> = user_input.chars().collect();
    for c in words[current_word_idx].chars() {
        if letter_on == user_input.len() - 1 || user_in_chars[letter_on] != c {
            break;
        }
        letter_on += 1;
    }

    starting_idx + letter_on
}

fn get_formatted_texts_line_mode<'a>(
    current_word: &str,
    user_input: &str,
    mut formatted_passage: Vec<Text<'a>>,
) -> FormattedTexts<'a> {
    let (formatted_passage_word, formatted_input) = get_formatted_words(current_word, user_input);
    formatted_passage[0..(formatted_passage_word.len())]
        .clone_from_slice(&formatted_passage_word[..]);

    FormattedTexts {
        passage: formatted_passage,
        input: formatted_input,
        error: !check_like_word(current_word, user_input),
        complete: false,
    }
}

/// Get fully formatted versions of the passage, and the user's input.
fn get_formatted_texts<'a>(
    words: &[&str],
    user_input: &str,
    current_word_idx: usize,
    mut formatted_passage: Vec<Text<'a>>,
) -> FormattedTexts<'a> {
    let (formatted_passage_word, formatted_input) =
        get_formatted_words(words[current_word_idx], user_input);

    let starting_idx = get_starting_idx(words, current_word_idx);

    formatted_passage[starting_idx..(formatted_passage_word.len() + starting_idx)]
        .clone_from_slice(&formatted_passage_word[..]);

    FormattedTexts {
        passage: formatted_passage,
        input: formatted_input,
        error: !check_like_word(words[current_word_idx], user_input),
        complete: false,
    }
}

/// Should be the final formatting call.
/// Sets formatted texts fields to expected completion settings.
/// Reformats the entire passage from scratch in the case that the user is
/// running with display_settings.always_max=false.
/// If they are, they will only see the final word, but showing the whole
/// passage to them now that it is complete is a much better user experience.
fn get_reformatted_complete_texts<'a>(words: &[&str]) -> FormattedTexts<'a> {
    get_fully_reformatted_texts(words, Color::Green, "COMPLETE", false)
}

fn get_reformatted_failed_texts<'a>(words: &[&str]) -> FormattedTexts<'a> {
    get_fully_reformatted_texts(words, Color::Red, "FAIL", true)
}

fn get_fully_reformatted_texts<'a>(
    words: &[&str],
    color: Color,
    end_string: &'a str,
    err: bool,
) -> FormattedTexts<'a> {
    let reformatted_complete_texts = (*words)
        .iter()
        .map(|word| Text::styled(format!("{} ", word), Style::default().fg(color)))
        .collect();
    FormattedTexts {
        passage: reformatted_complete_texts,
        input: vec![Text::styled(
            end_string,
            Style::default().bg(color).fg(Color::White),
        )],
        error: err,
        complete: true,
    }
}

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

    let mut formatted_texts = FormattedTexts {
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

                if c == ' ' && check_word(words[current_word_idx], &user_input) {
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
            get_formatted_texts(
                &words,
                &user_input.to_string(),
                current_word_idx,
                formatted_texts.passage,
            )
        } else {
            get_formatted_texts_line_mode(
                &words[current_word_idx],
                &user_input.to_string(),
                formatted_texts.passage,
            )
        };

        let current_letter_idx = get_trying_letter_idx(&words, current_word_idx, &user_input);
        if formatted_texts.error && new_char {
            stats.increment_errors(current_letter_idx);
            if instant_death {
                formatted_texts = get_reformatted_failed_texts(&words);
                continue;
            }
        } else {
            stats.increment_combo(current_letter_idx);
        }

        if current_word_idx + 1 == words.len() && check_word(words[current_word_idx], &user_input) {
            // Check to see if the user is on the last word and it is correct.
            // If it is, we need to do a little extra work to set the passage back to the full
            // passage. If the user is running with display_settings.always_max=false then they
            // will only see the last word.
            formatted_texts = get_reformatted_complete_texts(&words);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_word() {
        assert!(check_word("darrien", "darrien"));
        assert!(!check_word("Darrien", "darrien"));
        assert!(!check_word("Darrien", "Glasser"));
    }

    #[test]
    fn test_check_like_word() {
        // Normal case
        assert!(check_like_word("darrien", "darr"));

        // Full word
        assert!(check_like_word("darrien", "darrien"));

        // Input is longer than word to check
        assert!(!check_like_word("darrien", "darrienglasser.com"));

        // Case sensitivity
        assert!(!check_like_word("darrien", "Darrien"));
    }

    #[test]
    fn test_get_starting_idx() {
        let words = vec!["this", "is", "a", "vector"];
        assert!(get_starting_idx(&words, 2) == 8);
        assert!(get_starting_idx(&words, 0) == 0);
        assert!(get_starting_idx(&words, 1) == 5);
    }

    #[test]
    fn test_get_formatted_words_correct() {
        // Test all letters are correct condition
        let test_word = "terminal-typeracer";
        let (formatted_word, formatted_input) = get_formatted_words(test_word, test_word);
        let properly_formatted_word: Vec<Text> = test_word
            .chars()
            .map(|it| Text::styled(it.to_string(), Style::default().fg(Color::Green)))
            .collect();
        let mut properly_formatted_input: Vec<Text> = test_word
            .chars()
            .map(|it| Text::styled(it.to_string(), Style::default().fg(Color::Green)))
            .collect();
        properly_formatted_input.push(Text::styled(" ", Style::default().bg(Color::Blue)));
        assert!(formatted_word == properly_formatted_word);
        assert!(formatted_input == properly_formatted_input);
    }

    #[test]
    fn test_get_formatted_words_err() {
        let test_word = "terminal-type";
        let test_input = "termimal-type";

        // There has to be a better way to do this
        let properly_formatted_word = vec![
            Text::styled("t", Style::default().fg(Color::Green)),
            Text::styled("e", Style::default().fg(Color::Green)),
            Text::styled("r", Style::default().fg(Color::Green)),
            Text::styled("m", Style::default().fg(Color::Green)),
            Text::styled("i", Style::default().fg(Color::Green)),
            Text::styled("n", Style::default().fg(Color::White).bg(Color::Red)),
            Text::raw("a"),
            Text::raw("l"),
            Text::raw("-"),
            Text::raw("t"),
            Text::raw("y"),
            Text::raw("p"),
            Text::raw("e"),
        ];

        let mut properly_formatted_input: Vec<Text> = test_input
            .chars()
            .map(|it| {
                Text::styled(
                    it.to_string(),
                    Style::default().fg(Color::White).bg(Color::Red),
                )
            })
            .collect();
        properly_formatted_input.push(Text::styled(" ", Style::default().bg(Color::Blue)));

        let (formatted_word, formatted_input) = get_formatted_words(test_word, test_input);

        assert!(properly_formatted_word == formatted_word);
        assert!(properly_formatted_input == formatted_input);
    }

    #[test]
    fn test_get_formatted_texts() {
        // Test that words are added in place to a set of formatted texts
        // Do not need to check correct vs incorrect. All we need to verify is that the formatted
        // texts are properly applied to the full set of formatted texts.
        let words = vec!["the", "quick", "brown", "fox"];
        let user_input = "bro";
        let current_word_idx = 2;
        let input_formatted_passage: Vec<Text> = vec![
            Text::styled("t", Style::default().fg(Color::Green)),
            Text::styled("h", Style::default().fg(Color::Green)),
            Text::styled("e", Style::default().fg(Color::Green)),
            Text::styled(" ", Style::default().fg(Color::Green)),
            Text::styled("q", Style::default().fg(Color::Green)),
            Text::styled("u", Style::default().fg(Color::Green)),
            Text::styled("i", Style::default().fg(Color::Green)),
            Text::styled("c", Style::default().fg(Color::Green)),
            Text::styled("k", Style::default().fg(Color::Green)),
            Text::styled(" ", Style::default().fg(Color::Green)),
            Text::raw("b"),
            Text::raw("r"),
            Text::raw("o"),
            Text::raw("w"),
            Text::raw("n"),
            Text::raw(" "),
            Text::raw("f"),
            Text::raw("o"),
            Text::raw("x"),
        ];

        let expected_formatted_passage: Vec<Text> = vec![
            Text::styled("t", Style::default().fg(Color::Green)),
            Text::styled("h", Style::default().fg(Color::Green)),
            Text::styled("e", Style::default().fg(Color::Green)),
            Text::styled(" ", Style::default().fg(Color::Green)),
            Text::styled("q", Style::default().fg(Color::Green)),
            Text::styled("u", Style::default().fg(Color::Green)),
            Text::styled("i", Style::default().fg(Color::Green)),
            Text::styled("c", Style::default().fg(Color::Green)),
            Text::styled("k", Style::default().fg(Color::Green)),
            Text::styled(" ", Style::default().fg(Color::Green)),
            Text::styled("b", Style::default().fg(Color::Green)),
            Text::styled("r", Style::default().fg(Color::Green)),
            Text::styled("o", Style::default().fg(Color::Green)),
            Text::styled("w", Style::default().fg(Color::White).bg(Color::Blue)),
            Text::raw("n"),
            Text::raw(" "),
            Text::raw("f"),
            Text::raw("o"),
            Text::raw("x"),
        ];

        let formatted_texts = get_formatted_texts(
            &words,
            user_input,
            current_word_idx,
            input_formatted_passage,
        );

        assert!(expected_formatted_passage == formatted_texts.passage);
        assert!(!formatted_texts.error);
    }

    #[test]
    fn test_get_formatted_line_mode() {
        // Test that words are added in place to a set of formatted texts
        // Do not need to check correct vs incorrect. All we need to verify is that the formatted
        // texts are properly applied to the full set of formatted texts.
        let words = vec!["the", "quick", "brown", "fox"];
        let user_input = "bro";
        let current_word_idx = 2;
        let input_formatted_passage: Vec<Text> = vec![
            Text::raw("b"),
            Text::raw("r"),
            Text::raw("o"),
            Text::raw("w"),
            Text::raw("n"),
            Text::raw(" "),
            Text::raw("f"),
            Text::raw("o"),
            Text::raw("x"),
        ];

        let expected_formatted_passage: Vec<Text> = vec![
            Text::styled("b", Style::default().fg(Color::Green)),
            Text::styled("r", Style::default().fg(Color::Green)),
            Text::styled("o", Style::default().fg(Color::Green)),
            Text::styled("w", Style::default().fg(Color::White).bg(Color::Blue)),
            Text::raw("n"),
            Text::raw(" "),
            Text::raw("f"),
            Text::raw("o"),
            Text::raw("x"),
        ];

        let formatted_texts = get_formatted_texts_line_mode(
            &words[current_word_idx],
            user_input,
            input_formatted_passage,
        );

        assert!(expected_formatted_passage == formatted_texts.passage);
        assert!(!formatted_texts.error);
    }
}
