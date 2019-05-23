use directories::ProjectDirs;
use flate2::read::GzDecoder;
use rand::Rng;
use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, Error, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io};
use tar::Archive;
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
fn get_passage() -> (String, String) {
    let quote_dir = get_quote_dir().to_string();
    let num_files = fs::read_dir(quote_dir).unwrap().count();
    let random_file_num = rand::thread_rng().gen_range(0, num_files);
    let fallback = (
        "The quick brown fox jumps over the lazy dog".to_owned(),
        "darrienglasser.com".to_owned(),
    );

    if num_files == 0 {
        return fallback;
    } else {
        let mut count = 0;
        let read_dir_iter = get_quote_dir().to_string();
        for path in fs::read_dir(read_dir_iter).unwrap() {
            if count == random_file_num {
                let file = File::open(path.unwrap().path()).expect("File somehow did not exist.");
                let mut passage: Vec<String> = vec![];
                for line in BufReader::new(file).lines() {
                    passage.push(line.unwrap());
                }
                if passage.len() >= 2 {
                    return (passage[0].trim().to_string(), passage[1].clone());
                }
            }
            count += 1;
        }
    }

    fallback
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

fn start_game() -> Result<(), Error> {
    let stdout = stdout().into_raw_mode()?;
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);
    let mut terminal = Terminal::new(backend)?;

    let (raw_passage, raw_title) = get_passage();
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

fn resolution_check() -> Result<(), Error> {
    let stdout = stdout().into_raw_mode()?;
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);
    let mut terminal = Terminal::new(backend)?;
    let mut need_input = false;

    loop {
        terminal.draw(|mut f| {
            let size = f.size();
            let height = size.height;
            let width = size.width;

            let recommended_height = 30;
            let recommended_width = 80;

            if height < recommended_height || width < recommended_width {
                need_input = true;
            }

            if need_input {
                let root_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(f.size());
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(5)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(root_layout[0]);
                let passage_block = Block::default()
                    .borders(Borders::ALL)
                    .title_style(Style::default());
                Paragraph::new(
                    [Text::raw(format!(
                        "Terminal width and height too small!\nwidth: {}\nheight: {}\n\nIt is strongly recommended to play this with a height of at least: {} and a width of at least: {}\nConsider making your terminal fullscreen!\n\nCheck again <ENTER>, Ignore check: ^D, Exit: ^C",
                        width,
                        height,
                        recommended_height,
                        recommended_width
                    ))]
                    .iter(),
                )
                .block(passage_block.clone().title("Checking bounds"))
                .wrap(true)
                .alignment(Alignment::Left)
                .render(&mut f, chunks[0]);
            }
        })?;

        if need_input {
            let stdin = stdin();
            for c in stdin.keys() {
                let checked = c.unwrap();
                if checked == Key::Ctrl('c') {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "User wants to exit",
                    ));
                }
                if checked == Key::Ctrl('d') {
                    return Ok(());
                }
                if checked == Key::Char('\n') {
                    break;
                }
            }
        } else {
            return Ok(());
        }
    }
}

fn get_quote_dir() -> String {
    format!("{}/{}", create_data_dir(), "quote-pack").to_string()
}

fn create_data_dir() -> String {
    let dirs = ProjectDirs::from("org", "darrienglasser.com", "typeracer").unwrap();
    fs::create_dir_all(dirs.data_dir()).expect("Failed to create data dir");
    dirs.data_dir().to_str().unwrap().to_string()
}

fn download(url: &str, file_path: &str) {
    let mut resp = reqwest::get(url).expect("request failed");
    let mut out = File::create(file_path).expect("failed to create file");
    io::copy(&mut resp, &mut out).expect("Failed to write quote pack to disk");
}

fn expand_lang_pack(file_path: &str, extract_path: &str) -> Result<(), Error> {
    let tar_gz = File::open(file_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(extract_path)
}

fn retrieve_lang_pack() -> Result<(), Error> {
    let stdout = stdout().into_raw_mode()?;
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);

    let mut terminal = Terminal::new(backend)?;

    let lang_pack_url = "https://gitlab.com/DarrienG/terminal-typeracer/raw/271755ddf217ae8932c7ecd44bd3db963c27634f/assets/quote-pack.tar.gz";

    let mut step_instruction = "Lang pack (~40Ki) not installed. Would you like to install now? (requires an internet connection)\nYes: y, No: n\n".to_string();
    let mut step_count = 0;

    let mut data_dir: String = "".to_string();
    let mut file_path: String = "".to_string();

    let mut result: Result<(), Error> = Ok(());

    loop {
        let stdin = stdin();
        terminal.draw(|mut f| {
            let root_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(5)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(root_layout[0]);
            let passage_block = Block::default()
                .borders(Borders::ALL)
                .title_style(Style::default());
            Paragraph::new([Text::raw(&step_instruction)].iter())
                .block(passage_block.clone().title("Checking bounds"))
                .wrap(true)
                .alignment(Alignment::Left)
                .render(&mut f, chunks[0]);
        })?;

        if step_count == 0 {
            for c in stdin.keys() {
                let checked = c.unwrap();
                if checked == Key::Char('y') {
                    step_count += 1;
                    data_dir = create_data_dir();
                    step_instruction.push_str(&format!("\nMaking data dir at: {}\n", data_dir));
                    break;
                }
                if checked == Key::Ctrl('n') {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "User wants to exit",
                    ));
                }
            }
        } else if step_count == 1 {
            step_count += 1;
            step_instruction.push_str("Downloading lang pack...\n");
            file_path = format!("{}/{}", &data_dir, "quote-pack.tar.gz");
            download(lang_pack_url, &file_path);
            step_instruction.push_str("Lang pack downloaded!\n");
        } else if step_count == 2 {
            step_count += 1;
            step_instruction.push_str("Extracting lang pack.\n");
            result = expand_lang_pack(&file_path, &data_dir);
            if result.is_err() {
                step_instruction.push_str(
                    "Failed to extract lang pack. Please quit and try again.\n^D to exit.\n",
                );
            } else {
                step_instruction
                    .push_str("Lang pack downloaded and ready to go!\n^D to continue\n");
            }
        } else {
            for c in stdin.keys() {
                if c.unwrap() == Key::Ctrl('d') {
                    return result;
                }
            }
        }
    }
}

fn check_lang_pack() -> bool {
    let data_dir = create_data_dir();
    return fs::read_dir(data_dir).unwrap().count() > 0;
}

fn main() -> Result<(), Error> {
    if !resolution_check().is_err() {
        if !check_lang_pack() {
            let result = retrieve_lang_pack();
            if result.is_err() {
                return result;
            }
        }
        return start_game();
    }
    Ok(())
}
