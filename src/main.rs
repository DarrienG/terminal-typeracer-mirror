use std::io::{stdin, stdout, Error, Write};
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

// TODO: Calculate constraints based on terminal size
// e.g. smaller terminal means smaller padding on top and bottom
fn get_typing_bounds() -> [Constraint; 4] {
    [
        Constraint::Percentage(20),
        Constraint::Percentage(30),
        Constraint::Percentage(30),
        Constraint::Percentage(20),
    ]
}

// TODO: Calculate constraints based on terminal size
// e.g. smaller terminal means smaller padding on top and bottom
fn get_wpm_bounds() -> [Constraint; 3] {
    [
        Constraint::Percentage(20),
        Constraint::Percentage(30),
        Constraint::Percentage(60),
    ]
}

fn check_word(word: &str, input: &String) -> bool {
    return *word == *input;
}

// TODO: Read in passage from somewhere
// Or allow the user to pass it as an arg/stdin
fn get_passage() -> String {
    "The quick brown fox jumps over the lazy dog".to_owned()
}

// Get formatted version of a single word in a passage and the user's current input
// All similar characters up until the first different character are highlighted with green
// The first error character in the word is highlighted with red and the rest unformatted.
// The entire error is colored red on the user's input.
// returns a tuple with the formatted version of the: word and the input
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
    for i in word_dex..idx_word_count {
        if err_first_char {
            formatted_word.push(Text::styled(
                indexable_word[i].to_string(),
                Style::default().bg(Color::Red),
            ));
            err_first_char = false;
        } else {
            formatted_word.push(Text::raw(indexable_word[i].to_string()));
        }
    }

    // Make all of the user's typed error red
    for i in word_dex..idx_input_count {
        formatted_input.push(Text::styled(
            indexable_input[i].to_string(),
            Style::default().fg(Color::Red),
        ));
    }

    (formatted_word, formatted_input)
}

// Gets index of fully formatted text where the word the user is typing starts.
fn get_starting_idx(words: &Vec<&str>, current_word_idx: &usize) -> usize {
    let mut passage_starting_idx: usize = 0;
    for i in 0..*current_word_idx {
        passage_starting_idx += words[i].chars().count() + 1
    }
    passage_starting_idx
}

// Get fully formatted versions of the passage, and the user's input.
fn get_formatted_texts(
    words: &Vec<&str>,
    user_input: &String,
    current_word_idx: &usize,
    mut formatted_text: Vec<Text<'static>>,
) -> (Vec<Text<'static>>, Vec<Text<'static>>) {
    let (formatted_passage_word, formatted_user_input) =
        get_formatted_words(words[*current_word_idx].clone(), user_input);

    let starting_idx = get_starting_idx(words, current_word_idx);

    for i in 0..formatted_passage_word.len() {
        formatted_text[starting_idx + i] = formatted_passage_word[i].clone();
    }
    (formatted_text, formatted_user_input)
}

fn get_complete_string() -> Vec<Text<'static>> {
    vec![
        Text::styled(
            "COMPLETE\n",
            Style::default().bg(Color::Green).fg(Color::White),
        ),
        Text::raw("^C to quit"),
    ]
}

fn main() -> Result<(), Error> {
    // Initialize the terminal
    let stdout = stdout().into_raw_mode()?;
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);
    let mut terminal = Terminal::new(backend)?;

    let raw_passage = get_passage();
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
        terminal.draw(|mut f| {
            let root_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                .split(f.size());
            {
                // Typing layout
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(5)
                    .constraints(get_typing_bounds().as_ref())
                    .split(root_layout[0]);
                let passage_block = Block::default()
                    .borders(Borders::ALL)
                    .title_style(Style::default());
                Paragraph::new(formatted_passage.iter())
                    .block(passage_block.clone().title("Passage to type"))
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
                    .margin(5)
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
        })?;

        if current_word_idx == words.len() {
            break;
        }

        for c in stdin.keys() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            match c.unwrap() {
                Key::Ctrl('c') => return Ok(()),
                Key::Backspace => {
                    user_input.pop();
                    if user_input.chars().count() > 0 {
                        write!(terminal.backend_mut(), "{}", Left(1))?;
                    }
                    break;
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
                        break;
                    } else {
                        user_input.push(c);
                        write!(terminal.backend_mut(), "{}", Right(1))?;
                    }
                    let minute_float = ((now.as_secs() - start_time) as f64) / 60.0;
                    let word_count_float = current_word_idx as f64;
                    wpm = (word_count_float / minute_float) as u64;
                    break;
                }
                _ => {
                    break;
                }
            }
        }

        if current_word_idx == words.len() {
            // We want one more render cycle at the end.
            // Ignore the dangerous function call, and then do another bounds check and break
            // before taking user input again.
            user_input.clear();
            formatted_user_input = get_complete_string();
        } else {
            let (return_passage, return_input) = get_formatted_texts(
                &words,
                &user_input.to_string(),
                &current_word_idx,
                formatted_passage,
            );

            formatted_passage = return_passage;
            formatted_user_input = return_input;
        }
    }

    loop {
        let stdin = stdin();
        for c in stdin.keys() {
            if c.unwrap() == Key::Ctrl('c') {
                return Ok(());
            }
        }
    }
}
