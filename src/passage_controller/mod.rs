use rand::Rng;
use std::fs::{read_dir, DirEntry, File};
use std::io::{BufRead, BufReader};

use crate::actions::Action;
use crate::dirs::setup_dirs;

#[derive(Debug, Clone, PartialEq)]
pub struct PassageInfo {
    pub passage: String,
    pub title: String,
    pub passage_path: String,
}

#[derive(Debug, Clone)]
pub struct Controller {
    passages: Vec<PassageInfo>,
    current_passage_idx: usize,
    history_size: usize,
    start_idx: usize,
    first_run: bool,
}

/// A slightly smarter ringbuffer for preserving history
/// Saves the last 20 passages as history.
impl Controller {
    pub fn new(history_size: usize) -> Self {
        // We want to initialize one value in the vector before we start.
        // We could do all history_size, but not lazy loading with bigger values
        // could be expensive.
        Controller {
            passages: vec![],
            current_passage_idx: 0,
            history_size,
            start_idx: 0,
            first_run: true,
        }
    }

    /// Retrieve a passage.
    /// Takes into account history and the previous action given.
    pub fn retrieve_passage(&mut self, action: Action) -> &PassageInfo {
        match action {
            Action::NextPassage => self.retrieve_next_passage(),
            Action::PreviousPassage => self.retrieve_previous_passage(),
            Action::RestartPassage => &self.passages[self.current_passage_idx],
            Action::Quit => &self.passages[self.current_passage_idx],
        }
    }

    fn retrieve_next_passage(&mut self) -> &PassageInfo {
        // Because we can't guarantee we are starting with the ability to read passages (e.g. the
        // user may not have downloaded the lang_pack yet, our elegant always incrementing
        // algorithm will throw an out of bounds error on the first run if we increment
        // immediately.
        // Doing nothing on the first run solves this problem.
        if self.first_run {
            self.first_run = false;
            self.passages.push(self.get_new_passage());
        } else {
            self.current_passage_idx = (self.current_passage_idx + 1) % self.history_size;

            // The only times we need to get a new passage rather than look in history:
            // - When we have forced the start_idx to push forward one
            // - When we have not yet filled the history up
            if self.current_passage_idx == self.start_idx {
                self.start_idx = (self.start_idx + 1) % self.history_size;
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
        }
        &self.passages[self.current_passage_idx]
    }

    fn retrieve_previous_passage(&mut self) -> &PassageInfo {
        // If we're at the starting position, we shouldn't go back any further.
        if self.current_passage_idx != self.start_idx {
            // current_passage_idx is a usize, we can't go below 0, otherwise we get an underflow.
            if self.current_passage_idx != 0 {
                self.current_passage_idx -= 1;
                self.current_passage_idx %= self.history_size;
            } else {
                self.current_passage_idx = self.history_size - 1;
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

    fn pick_quote_dir(&self) -> DirEntry {
        let mut quote_dirs = self.get_quote_dirs();
        quote_dirs.remove(rand::thread_rng().gen_range(0, quote_dirs.len()))
    }

    /// Get shortnames of quote directories.
    pub fn get_quote_dir_shortnames(&self) -> Vec<String> {
        let mut dirs: Vec<String> = self
            .get_quote_dirs()
            .iter()
            .map(|dir| {
                dir.path()
                    .file_stem()
                    .expect("Unable to get file")
                    .to_string_lossy()
                    .to_string()
            })
            .collect();

        dirs.sort();
        dirs
    }

    fn get_quote_dirs(&self) -> Vec<DirEntry> {
        self.without_bad_paths(
            read_dir(setup_dirs::get_quote_dir().to_string())
                .unwrap()
                .map(|dir| dir.unwrap())
                .collect(),
        )
    }

    fn without_bad_paths(&self, entries: Vec<DirEntry>) -> Vec<DirEntry> {
        let mut true_quote_dirs: Vec<DirEntry> = vec![];
        for entry in entries {
            let str_entry = entry
                .path()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            if str_entry != "version" && str_entry != ".git" {
                true_quote_dirs.push(entry);
            }
        }
        true_quote_dirs
    }

    // Retrieve a random passage and title from quote database.
    // Defaults to boring passage if no files are found.
    // Returns (passage, author/title)
    // TODO: Test
    // Difficult to test with unit tests. Expects a database file.
    #[cfg(not(test))]
    fn get_new_passage(&self) -> PassageInfo {
        let quote_dir = self.pick_quote_dir();
        let num_files: usize = read_dir(quote_dir.path()).unwrap().count();
        let random_file_num = rand::thread_rng().gen_range(0, num_files);
        let fallback = PassageInfo {
            passage: "The quick brown fox jumps over the lazy dog".to_owned(),
            title: "darrienglasser.com".to_owned(),
            passage_path: "FALLBACK_PATH".to_owned(),
        };

        if num_files > 0 {
            let read_dir_iter = quote_dir.path();
            for (count, path) in read_dir(read_dir_iter)
                .expect("Failed to read from data dir")
                .enumerate()
            {
                let path = path
                    .expect("Failed to evaluate path while reading files")
                    .path();
                if count == random_file_num {
                    let file = File::open(&path).expect("Unable to open quote file");
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
        // Since we aren't working with real passages, we need some source of randomness
        // The chance that two of these collide is close enough to zero that we can assume they
        // will always be different.
        PassageInfo {
            passage: format!("{}", rand::thread_rng().gen_range(0, 10_000_000)),
            title: format!("{}", rand::thread_rng().gen_range(0, 10_000_000)),
            passage_path: format!("{}", rand::thread_rng().gen_range(0, 10_000_000)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_next_passage_overwrite() {
        // Check to see if we keep asking for next passages they're always valid
        // History of 5 so we can loop through history multiple times
        let mut passage_controller = Controller::new(5);
        for _ in 0..4000 {
            passage_controller.retrieve_next_passage();
        }

        // And ensure the size is still 5
        assert!(passage_controller.passages.len() == 5);
    }

    #[test]
    fn test_get_vastly_previous_passage() {
        // Check to make sure we can keep asking for previous passages and they're always valid

        // Since we return a reference to a passage_info, and these methods require a mutable
        // reference, we have to clone to make the borrow checker happy.
        let mut passage_controller = Controller::new(5);
        passage_controller.retrieve_next_passage();
        let mut previous_passage = (*passage_controller.retrieve_previous_passage()).clone();
        for _ in 0..4000 {
            let passage = (*passage_controller.retrieve_previous_passage()).clone();
            assert!(passage == previous_passage);
            previous_passage = passage;
        }
    }

    #[test]
    fn test_verify_history_integrity() {
        let mut passage_controller = Controller::new(5);
        passage_controller.retrieve_next_passage();
        let passage0 = (*passage_controller.retrieve_passage(Action::PreviousPassage)).clone();
        let passage1 = (*passage_controller.retrieve_passage(Action::NextPassage)).clone();
        let passage2 = (*passage_controller.retrieve_passage(Action::NextPassage)).clone();

        // Check going backwards and forwards
        assert!(passage1 == *passage_controller.retrieve_passage(Action::PreviousPassage));
        assert!(passage0 == *passage_controller.retrieve_passage(Action::PreviousPassage));
        assert!(passage1 == *passage_controller.retrieve_passage(Action::NextPassage));
        assert!(passage2 == *passage_controller.retrieve_passage(Action::NextPassage));

        let passage3 = (*passage_controller.retrieve_passage(Action::NextPassage)).clone();
        let passage4 = (*passage_controller.retrieve_passage(Action::NextPassage)).clone();

        // Make sure the passages are unique
        assert!(passage0 != passage1);
        assert!(passage1 != passage2);
        assert!(passage2 != passage3);
        assert!(passage3 != passage4);

        // Ensure we start overwriting old history when we roll past the history limit
        assert!(passage0 != *passage_controller.retrieve_passage(Action::NextPassage));
        assert!(passage1 != *passage_controller.retrieve_passage(Action::NextPassage));
        assert!(passage2 != *passage_controller.retrieve_passage(Action::NextPassage));
        assert!(passage3 != *passage_controller.retrieve_passage(Action::NextPassage));
        assert!(passage4 != *passage_controller.retrieve_passage(Action::NextPassage));
    }

    #[test]
    fn test_verify_restart() {
        let mut passage_controller = Controller::new(5);
        passage_controller.retrieve_next_passage();

        // restarting on the initial passage doesn't break and gives the correct passage
        let passage0 = (*passage_controller.retrieve_passage(Action::PreviousPassage)).clone();
        let passage0_restart =
            (*passage_controller.retrieve_passage(Action::RestartPassage)).clone();
        assert_eq!(passage0, passage0_restart);

        // can move forwards and restart then skip some and restart a previous one
        let passage0 = (*passage_controller.retrieve_passage(Action::NextPassage)).clone();
        let passage0_restart =
            (*passage_controller.retrieve_passage(Action::RestartPassage)).clone();
        let _ = (*passage_controller.retrieve_passage(Action::NextPassage)).clone();
        let passage2 = (*passage_controller.retrieve_passage(Action::PreviousPassage)).clone();
        let passage2_restart =
            (*passage_controller.retrieve_passage(Action::RestartPassage)).clone();
        assert_eq!(passage0, passage0_restart);
        assert_eq!(passage2, passage2_restart);
    }
}
