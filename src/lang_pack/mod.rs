use flate2::read::GzDecoder;
use rand::Rng;
use std::fs::{read_dir, File};
use std::io;
use std::io::{stdin, stdout, BufRead, BufReader, Error};
use tar::Archive;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

use crate::actions;
use crate::dirs::setup_dirs;

mod lang_pack_render;

#[derive(Debug, Clone)]
pub struct PassageInfo {
    pub passage: String,
    pub title: String,
    pub passage_path: String,
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

pub fn check_lang_pack() -> bool {
    let data_dir = setup_dirs::create_data_dir();
    read_dir(data_dir).unwrap().count() > 0
}

pub fn retrieve_lang_pack() -> Result<(), Error> {
    let stdout = stdout().into_raw_mode()?;
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);

    let mut terminal = Terminal::new(backend)?;

    let lang_pack_url = "https://gitlab.com/DarrienG/terminal-typeracer/raw/c7a1c5259b21b8faea39312cd013425b3bf8440e/assets/quote-pack.tar.gz";

    let mut step_instruction = "Lang pack (~40Ki) not installed. Would you like to install now? (requires an internet connection)\nYes: y, No: n\n".to_string();
    let mut step_count = 0;

    let mut data_dir: String = "".to_string();
    let mut file_path: String = "".to_string();

    let mut result: Result<(), Error> = Ok(());

    loop {
        let stdin = stdin();
        lang_pack_render::render(&mut terminal, &step_instruction);
        if step_count == 0 {
            for c in stdin.keys() {
                let checked = c.unwrap();
                if checked == Key::Char('y') {
                    step_count += 1;
                    data_dir = setup_dirs::create_data_dir();
                    step_instruction.push_str(&format!("\nMaking data dir at: {}\n", data_dir));
                    break;
                }
                if checked == Key::Char('n') {
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

#[derive(Debug, Clone)]
pub struct PassageController {
    passages: Vec<PassageInfo>,
    current_passage_idx: usize,
    history_size: usize,
    start_idx: usize,
}

/// A slightly smarter ringbuffer for preserving history
/// Saves the last 20 passages as history.
impl PassageController {
    pub fn new(history_size: usize) -> Self {
        // We want to initialize one value in the vector before we start.
        // We could do all history_size, but not lazy loading with bigger values
        // could be expensive.
        let mut pc = PassageController {
            passages: vec![],
            current_passage_idx: 0,
            history_size,
            start_idx: 0,
        };

        pc.passages.push(pc.get_new_passage());
        pc
    }

    /// Retrieve a passage.
    /// Takes into account history and the previous action given.
    pub fn retrieve_passage(&mut self, action: actions::Action) -> &PassageInfo {
        match action {
            actions::Action::NextPassage => self.retrieve_next_passage(),
            actions::Action::PreviousPassage => self.retrieve_previous_passage(),
            _ => &self.passages[self.current_passage_idx],
        }
    }

    fn retrieve_next_passage(&mut self) -> &PassageInfo {
        self.current_passage_idx = (self.current_passage_idx + 1) % self.history_size;

        // The only times we need to get a new passage rather than look in history:
        // - When we have forced the start_idx to push forward one
        // - When we have not yet filled the history up
        if self.current_passage_idx == self.start_idx {
            self.start_idx = self.history_size % (self.start_idx + 1);
            // Should we expand the vector, or push a new passage on?
            if self.passages.len() < self.history_size {
                self.passages.push(self.get_new_passage());
            } else {
                self.passages[self.current_passage_idx] = self.get_new_passage();
            }
        } else if self.passages.len() < self.history_size
            && self.current_passage_idx == self.passages.len()
        {
            self.passages.push(self.get_new_passage());
        }
        &self.passages[self.current_passage_idx]
    }

    fn retrieve_previous_passage(&mut self) -> &PassageInfo {
        if self.current_passage_idx == self.start_idx {
            // Don't do anything, we're at the last position in history
        } else {
            // Since the start_idx can be in places other than 0, a -1 could make us negative,
            // so we need to mod it with history_size.
            self.current_passage_idx -= 1;
            if self.current_passage_idx != 0 {
                self.current_passage_idx %= self.history_size;
            }
        }
        &self.passages[self.current_passage_idx]
    }

    // TODO: If we want the user to be able to input for any passage, this should become
    // smarter so it can insert to the next passage every time instead of assuming it is inserting
    // the first time.
    // For now this passage is just for inserting the initial user input if they want it.
    pub fn write_initial_passage(&mut self, passage: &str) {
        self.passages.push(PassageInfo {
            passage: passage.to_owned(),
            title: "User input".to_owned(),
            passage_path: "User input".to_owned(),
        });
    }

    // Retrieve a random passage and title from quote database.
    // Defaults to boring passage if no files are found.
    // Returns (passage, author/title)
    // TODO: Test
    // Difficult to test with unit tests. Expects a database file.
    #[cfg(not(test))]
    fn get_new_passage(&self) -> PassageInfo {
        let quote_dir = setup_dirs::get_quote_dir().to_string();
        let num_files = read_dir(quote_dir).unwrap().count();
        let random_file_num = rand::thread_rng().gen_range(0, num_files);
        let fallback = PassageInfo {
            passage: "The quick brown fox jumps over the lazy dog".to_owned(),
            title: "darrienglasser.com".to_owned(),
            passage_path: "FALLBACK_PATH".to_owned(),
        };

        if num_files > 0 {
            let read_dir_iter = setup_dirs::get_quote_dir().to_string();
            for (count, path) in read_dir(read_dir_iter)
                .expect("Failed to read from data dir")
                .enumerate()
            {
                let path = path
                    .expect("Failed to evaluate path while reading files")
                    .path();
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

    #[cfg(test)]
    fn get_new_passage(&self) -> PassageInfo {
        PassageInfo {
            passage: "the quick brown fox...".to_owned(),
            title: "testing...".to_owned(),
            passage_path: "TEST_GET_PASSAGE".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_next_passage() {}
}
