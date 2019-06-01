use rand::Rng;
use std::fs;
use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use termion::cursor::{Left, Right};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::Color;
use tui::style::{Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Terminal;

use crate::actions;

use crate::dirs::setup_dirs;

// Convenience method for retrieving constraints for the typing layout.
// At some point this may be refactored to be more dynamic based on
// terminal layout size so we can skip resolution checks.
fn get_typing_bounds() -> [Constraint; 4] {
    [
        Constraint::Percentage(20),
        Constraint::Percentage(30),
        Constraint::Percentage(30),
        Constraint::Percentage(20),
    ]
}

// Convenience method for retrieving constraints for the wpm layout.
// At some point this may be refactored to be more dynamic based on
// terminal layout size so we can skip resolution checks.
fn get_wpm_bounds() -> [Constraint; 3] {
    [
        Constraint::Percentage(20),
        Constraint::Percentage(30),
        Constraint::Percentage(60),
    ]
}

// Get the words per minute based on a words per minute algorithm.
// If legacy is set to true, use the actual words per minute, otherwise use chars/5 per minute.
// See: https://en.wikipedia.org/wiki/Words_per_minute#Alphanumeric_entry
fn derive_wpm(
    word_idx: usize,
    word_vec: &[&str],
    duration: u64,
    start_time: u64,
    legacy: bool,
) -> u64 {
    if legacy {
        get_legacy_wpm(word_idx, duration, start_time)
    } else {
        get_wpm(word_idx, word_vec, duration, start_time)
    }
}

// Get words per minute where a word is 5 chars.
fn get_wpm(word_idx: usize, word_vec: &[&str], duration: u64, start_time: u64) -> u64 {
    let mut char_count = 0;
    for item in word_vec.iter().take(word_idx) {
        // add 1 for space
        char_count += item.chars().count() + 1;
    }
    let minute_float = ((duration - start_time) as f64) / 60.0;
    let word_count_float = char_count as f64 / 5.0;
    (word_count_float / minute_float).ceil() as u64
}

// Get words per minute where a word is a set of characters delimited by a space.
fn get_legacy_wpm(word_idx: usize, duration: u64, start_time: u64) -> u64 {
    let minute_float = ((duration - start_time) as f64) / 60.0;
    let word_count_float = (word_idx + 1) as f64;
    (word_count_float / minute_float).ceil() as u64
}

// Determine if two words are the same.
fn check_word(word: &str, input: &str) -> bool {
    *word == *input
}

// Retrieve a random passage and title from quote database.
// Defaults to boring passage if no files are found.
// Returns (passage, author/title)
fn get_passage() -> (String, String) {
    let quote_dir = setup_dirs::get_quote_dir().to_string();
    let num_files = fs::read_dir(quote_dir).unwrap().count();
    let random_file_num = rand::thread_rng().gen_range(0, num_files);
    let fallback = (
        "The quick brown fox jumps over the lazy dog".to_owned(),
        "darrienglasser.com".to_owned(),
    );

    if num_files == 0 {
        return fallback;
    } else {
        let read_dir_iter = setup_dirs::get_quote_dir().to_string();
        for (count, path) in fs::read_dir(read_dir_iter).unwrap().enumerate() {
            let path = path.unwrap().path();
            if count == random_file_num && path.file_stem().unwrap() != "version" {
                let file = File::open(path).expect("File somehow did not exist.");
                let mut passage: Vec<String> = vec![];
                for line in BufReader::new(file).lines() {
                    passage.push(line.unwrap());
                }
                if passage.len() >= 2 {
                    return (passage[0].trim().to_string(), passage[1].clone());
                }
            }
        }
    }

    fallback
}

// Get formatted version of a single word in a passage and the user's current input
// All similar characters up until the first different character are highlighted with green/
// The first error character in the word is highlighted with red and the rest unformatted.
// The entire error is colored red on the user's input.
// returns a tuple with the formatted version of the: word and the input.
fn get_formatted_words(word: &str, input: &str) -> (Vec<Text<'static>>, Vec<Text<'static>>) {
    let indexable_word: Vec<char> = word.chars().collect();
    let indexable_input: Vec<char> = input.chars().collect();
    let idx_word_count = indexable_word.len();
    let idx_input_count = indexable_input.len();

    let mut formatted_word: Vec<Text> = Vec::new();
    let mut formatted_input: Vec<Text> = Vec::new();
    let mut word_dex = 0;

    while word_dex < idx_word_count && word_dex < idx_input_count {
        if indexable_word[word_dex] != indexable_input[word_dex] {
            break;
        }

        formatted_word.push(Text::styled(
            indexable_word[word_dex].to_string(),
            Style::default().fg(Color::Green),
        ));
        formatted_input.push(Text::styled(
            indexable_word[word_dex].to_string(),
            Style::default().fg(Color::Green),
        ));

        word_dex += 1;
    }

    // Fill out whatever is left (the user has made a mistake for the rest of the word)

    // Only show the first error the user made in the passage (if there is any)
    let mut err_first_char = idx_input_count >= idx_word_count;
    for word in indexable_word.iter().skip(word_dex).take(idx_word_count) {
        if err_first_char {
            formatted_word.push(Text::styled(
                word.to_string(),
                Style::default().bg(Color::Red).fg(Color::White),
            ));
            err_first_char = false;
        } else {
            formatted_word.push(Text::raw(word.to_string()));
        }
    }

    // Make all of the user's typed error red
    for input in indexable_input.iter().skip(word_dex).take(idx_input_count) {
        formatted_input.push(Text::styled(
            input.to_string(),
            Style::default().bg(Color::Red).fg(Color::White),
        ));
    }

    (formatted_word, formatted_input)
}

// Gets index of fully formatted text where the word the user is typing starts.
fn get_starting_idx(words: &[&str], current_word_idx: usize) -> usize {
    let mut passage_starting_idx: usize = 0;
    for word in words.iter().take(current_word_idx) {
        passage_starting_idx += word.chars().count() + 1
    }
    passage_starting_idx
}

// Get fully formatted versions of the passage, and the user's input.
fn get_formatted_texts(
    words: &[&str],
    user_input: &str,
    current_word_idx: usize,
    mut formatted_text: Vec<Text<'static>>,
) -> (Vec<Text<'static>>, Vec<Text<'static>>) {
    let (formatted_passage_word, formatted_user_input) =
        get_formatted_words(words[current_word_idx], user_input);

    let starting_idx = get_starting_idx(words, current_word_idx);

    formatted_text[starting_idx..(formatted_passage_word.len() + starting_idx)]
        .clone_from_slice(&formatted_passage_word[..]);
    (formatted_text, formatted_user_input)
}

// Get default string to display when user completes a passage.
fn get_complete_string() -> Vec<Text<'static>> {
    vec![
        Text::styled(
            "COMPLETE\n",
            Style::default().bg(Color::Green).fg(Color::White),
        ),
        Text::raw("^A to play again, ^C to quit"),
    ]
}

// Event loop: Displays the typing input and renders keypresses.
// This is the entrance to the main game.
pub fn play_game(input: &str, legacy_wpm: bool) -> actions::Action {
    let stdout = stdout()
        .into_raw_mode()
        .expect("Failed to manipulate terminal to raw mode");
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);
    let mut terminal = Terminal::new(backend).expect("Failed to get handle to terminal");

    let (raw_passage, raw_title) = match input {
        "" => get_passage(),
        _ => (input.to_string(), "Terminal Typeracer".to_string()),
    };

    let mut formatted_passage: Vec<Text> = raw_passage
        .chars()
        .map(|it| Text::raw(it.to_string()))
        .collect();

    let mut user_input = String::new();
    let mut formatted_user_input: Vec<Text> = vec![];

    // Split the passager into vec of words to work on one at a time
    let words: Vec<&str> = raw_passage.split(' ').collect();
    let mut current_word_idx = 0;

    // Timing and wpm
    let mut wpm = 0;
    let mut start_time = 0;

    loop {
        let stdin = stdin();
        terminal
            .draw(|mut f| {
                // Because there is no way to specify vertical but not horizontal margins
                let padding_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage(5),
                            Constraint::Percentage(90),
                            Constraint::Percentage(5),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let base_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                    .split(padding_layout[1]);
                {
                    let root_layout = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(80), Constraint::Percentage(20)].as_ref(),
                        )
                        .split(base_layout[0]);
                    {
                        // Typing layout
                        let chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .margin(1)
                            .constraints(get_typing_bounds().as_ref())
                            .split(root_layout[0]);
                        let passage_block = Block::default()
                            .borders(Borders::ALL)
                            .title_style(Style::default());
                        Paragraph::new(formatted_passage.iter())
                            .block(passage_block.clone().title(&raw_title))
                            .wrap(true)
                            .alignment(Alignment::Left)
                            .render(&mut f, chunks[2]);

                        let typing_block = Block::default()
                            .borders(Borders::ALL)
                            .title_style(Style::default().modifier(Modifier::BOLD));
                        Paragraph::new(formatted_user_input.iter())
                            .block(typing_block.clone().title("Type out passage here"))
                            .wrap(true)
                            .alignment(Alignment::Left)
                            .render(&mut f, chunks[3]);
                    }
                    {
                        let chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .margin(1)
                            .constraints(get_wpm_bounds().as_ref())
                            .split(root_layout[1]);

                        let wpm_block = Block::default()
                            .borders(Borders::ALL)
                            .title_style(Style::default());
                        Paragraph::new([Text::raw(format!("WPM\n{}", wpm))].iter())
                            .block(wpm_block.clone().title("WPM"))
                            .alignment(Alignment::Center)
                            .render(&mut f, chunks[2]);
                    }
                    if user_input == "" && current_word_idx == 0 {
                        let chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .margin(0)
                            .constraints([Constraint::Percentage(100)].as_ref())
                            .split(base_layout[1]);

                        let shortcut_block = Block::default()
                            .borders(Borders::NONE)
                            .title_style(Style::default());
                        Paragraph::new(
                            [Text::raw("^C exit  ^N next passage  ^U clear word")].iter(),
                        )
                        .block(shortcut_block.clone())
                        .alignment(Alignment::Center)
                        .render(&mut f, chunks[0]);
                    }
                }
            })
            .expect("Failed to draw terminal widgets.");

        if current_word_idx == words.len() {
            break;
        }

        let c = stdin.keys().find_map(Result::ok);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        match c.unwrap() {
            Key::Ctrl('c') => return actions::Action::Quit,
            Key::Ctrl('n') => return actions::Action::NextPassage,
            // Get some basic readline bindings
            Key::Ctrl('u') => user_input.clear(),
            Key::Backspace => {
                user_input.pop();
                if user_input.chars().count() > 0 {
                    write!(terminal.backend_mut(), "{}", Left(1))
                        .expect("Failed to write to terminal.");
                }
            }
            Key::Char(c) => {
                if start_time == 0 {
                    start_time = now.as_secs() - 1;
                }

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
                    write!(terminal.backend_mut(), "{}", Right(1))
                        .expect("Failed to write to terminal.");
                }
                wpm = derive_wpm(
                    current_word_idx,
                    &words,
                    now.as_secs(),
                    start_time,
                    legacy_wpm,
                );
            }
            _ => {}
        }

        let (return_passage, return_input) = get_formatted_texts(
            &words,
            &user_input.to_string(),
            current_word_idx,
            formatted_passage,
        );

        formatted_passage = return_passage;
        formatted_user_input = return_input;

        if current_word_idx == words.len() {
            // We want one more render cycle at the end.
            // Ignore the dangerous function call, and then do another bounds check and break
            // before taking user input again.
            user_input.clear();
            formatted_user_input = get_complete_string();
        } else if current_word_idx + 1 == words.len()
            && check_word(words[current_word_idx], &user_input)
        {
            // Special case for the last word so the user doesn't need to hit space
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");

            current_word_idx += 1;
            wpm = derive_wpm(
                current_word_idx,
                &words,
                now.as_secs(),
                start_time,
                legacy_wpm,
            );
            user_input.clear();
            formatted_user_input = get_complete_string();
        }
    }

    loop {
        let stdin = stdin();
        for c in stdin.keys() {
            let checked = c.unwrap();
            if checked == Key::Ctrl('c') {
                return actions::Action::Quit;
            }
            if checked == Key::Ctrl('a') {
                return actions::Action::NextPassage;
            }
        }
    }
}
