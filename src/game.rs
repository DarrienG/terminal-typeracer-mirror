use rand::Rng;
use std::fs;
use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::style::Color;
use tui::style::Style;
use tui::widgets::Text;
use tui::Terminal;

use crate::actions;
use crate::dirs::setup_dirs;
use crate::stats;

mod game_render;

pub struct PassageInfo {
    pub passage: String,
    pub title: String,
    pub passage_path: String,
}

pub struct FormattedTexts<'a> {
    pub passage: Vec<Text<'a>>,
    pub input: Vec<Text<'a>>,
    pub error: bool,
}

// Determine if two words are the same.
fn check_word(word: &str, input: &str) -> bool {
    *word == *input
}

// Check to see if the "input" is like the word. This is effectively
// word.contains(input) but only if the first input.len characters are
// the same. e.g. apple, ap => true, apple ppl => false
fn check_like_word(word: &str, input: &str) -> bool {
    if input.is_empty() {
        return true;
    }
    if input.len() > word.len() {
        return false;
    }

    check_word(&word[..input.len()], input)
}

// Retrieve a random passage and title from quote database.
// Defaults to boring passage if no files are found.
// Returns (passage, author/title)
// TODO: Test
// Difficult to test with unit tests. Expects a database file.
fn get_passage() -> PassageInfo {
    let quote_dir = setup_dirs::get_quote_dir().to_string();
    let num_files = fs::read_dir(quote_dir).unwrap().count();
    let random_file_num = rand::thread_rng().gen_range(0, num_files);
    let fallback = PassageInfo {
        passage: "The quick brown fox jumps over the lazy dog".to_owned(),
        title: "darrienglasser.com".to_owned(),
        passage_path: "FALLBACK_PATH".to_owned(),
    };

    if num_files == 0 {
        return fallback;
    } else {
        let read_dir_iter = setup_dirs::get_quote_dir().to_string();
        for (count, path) in fs::read_dir(read_dir_iter).unwrap().enumerate() {
            let path = path.unwrap().path();
            if count == random_file_num && path.file_stem().unwrap() != "version" {
                let file = File::open(&path).expect("File somehow did not exist.");
                let mut passage: Vec<String> = vec![];
                for line in BufReader::new(file).lines() {
                    passage.push(line.unwrap());
                }
                if passage.len() >= 2 {
                    return PassageInfo {
                        passage: passage[0].trim().to_string(),
                        title: passage[1].clone(),
                        passage_path: path.to_string_lossy().into_owned(),
                    };
                }
            }
        }
    }

    fallback
}

// Get formatted version of a single word in a passage and the user's current input
// All similar characters up until the first different character are highlighted with green/
// The first error character in the word is highlighted with red and the rest unformatted

// The entire error is colored red on the user's input.
// returns a tuple with the formatted version of the: word and the input.
// TODO: Test
// Difficult to test, because Text does not implement eq
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

// Given a vector of word and the current index of the word the user is typing,
// ["this", "is", "a", "vector"] and current_word_idx of 2,
// return the index as if we were indexing the previous vector as a space
// separated string to get the first character of the word the user is
// currently on.
// In this case, we would get 8 back.
// "this is a vector"
// ---------^
fn get_starting_idx(words: &[&str], current_word_idx: usize) -> usize {
    let mut passage_starting_idx: usize = 0;
    for word in words.iter().take(current_word_idx) {
        passage_starting_idx += word.chars().count() + 1
    }
    passage_starting_idx
}

// Get fully formatted versions of the passage, and the user's input.
// TODO: Test
// Text doesn't derive eq, so it's difficult to test.
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
    }
}

// Get default string to display when user completes a passage.
fn get_complete_string() -> Vec<Text<'static>> {
    vec![Text::styled(
        "COMPLETE",
        Style::default().bg(Color::Green).fg(Color::White),
    )]
}

// Event loop: Displays the typing input and renders keypresses.
// This is the entrance to the main game.
// TODO: Provide get_backend method in game_render
pub fn play_game(input: &str, stats: &mut stats::Stats, debug_enabled: bool) -> actions::Action {
    let stdout = stdout()
        .into_raw_mode()
        .expect("Failed to manipulate terminal to raw mode");
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);
    let mut terminal = Terminal::new(backend).expect("Unable to get handle to terminal.");

    terminal.hide_cursor().expect("Failed to hide the cursor");

    let passage_info = match input {
        "" => get_passage(),
        _ => PassageInfo {
            passage: input.to_owned(),
            title: "Terminal Typeracer".to_owned(),
            passage_path: "User input".to_owned(),
        },
    };

    let mut formatted_texts = FormattedTexts {
        passage: passage_info
            .passage
            .chars()
            .map(|it| Text::raw(it.to_string()))
            .collect(),
        input: vec![],
        error: false,
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
                debug_enabled,
                word_idx: current_word_idx,
                passage_path: &passage_info.passage_path,
                current_word: if current_word_idx == words.len() {
                    "DONE"
                } else {
                    words[current_word_idx]
                },
            },
        );
        if current_word_idx == words.len() {
            break;
        }

        let stdin = stdin();
        let c = stdin.keys().find_map(Result::ok);
        match c.unwrap() {
            Key::Ctrl('c') => return actions::Action::Quit,
            Key::Ctrl('n') => return actions::Action::NextPassage,
            // Get some basic readline bindings
            Key::Ctrl('u') => user_input.clear(),
            Key::Backspace => {
                user_input.pop();
            }

            Key::Char(c) => {
                stats.update_start_time();

                if c == ' ' && check_word(words[current_word_idx], &user_input) {
                    current_word_idx += 1;
                    // BUG: Cursor stays in a forward position after clearing
                    // As soon as the user types it goes back to the beginning position
                    // Moving the cursor manually to the left does not fix
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

        formatted_texts = get_formatted_texts(
            &words,
            &user_input.to_string(),
            current_word_idx,
            formatted_texts.passage,
        );

        if current_word_idx == words.len() {
            // We want one more render cycle at the end.
            // Ignore the dangerous function call, and then do another bounds check and break
            // before taking user input again.
            user_input.clear();
            formatted_texts.input = get_complete_string();
        } else if current_word_idx + 1 == words.len()
            && check_word(words[current_word_idx], &user_input)
        {
            // Special case for the last word so the user doesn't need to hit space
            current_word_idx += 1;
            stats.update_wpm(current_word_idx, &words);
            user_input.clear();
            formatted_texts.input = get_complete_string();
        }
    }

    loop {
        let stdin = stdin();
        for c in stdin.keys() {
            let checked = c.unwrap();
            if checked == Key::Ctrl('c') {
                return actions::Action::Quit;
            }
            if checked == Key::Ctrl('n') {
                return actions::Action::NextPassage;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game;

    #[test]
    fn test_check_word() {
        assert!(game::check_word("darrien", "darrien"));
        assert!(!game::check_word("Darrien", "darrien"));
        assert!(!game::check_word("Darrien", "Glasser"));
    }

    #[test]
    fn test_check_like_word() {
        // Normal case
        assert!(game::check_like_word("darrien", "darr"));

        // Full word
        assert!(game::check_like_word("darrien", "darrien"));

        // Input is longer than word to check
        assert!(!game::check_like_word("darrien", "darrienglasser.com"));

        // Case sensitivity
        assert!(!game::check_like_word("darrien", "Darrien"));
    }

    #[test]
    fn test_get_starting_idx() {
        let words = vec!["this", "is", "a", "vector"];
        assert!(game::get_starting_idx(&words, 2) == 8);
        assert!(game::get_starting_idx(&words, 0) == 0);
        assert!(game::get_starting_idx(&words, 1) == 5);
    }

}
