use std::cmp::max;
#[cfg(not(test))]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
#[derive(Debug, Clone)]
struct Time {
    seconds: u64,
}

#[cfg(not(test))]
#[derive(Debug, Clone)]
struct Time {}

impl Time {
    #[cfg(test)]
    pub fn new() -> Self {
        Time { seconds: 0 }
    }

    #[cfg(not(test))]
    pub fn new() -> Self {
        Time {}
    }

    #[cfg(test)]
    /// Return the time used for testing
    pub fn now(&self) -> u64 {
        self.seconds
    }

    #[cfg(not(test))]
    /// Return the current time in seconds since UNIX_EPOCH
    pub fn now(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }
}

/// Store the current stats to be displayed to the user
#[derive(Debug, Clone)]
pub struct Stats {
    pub errors: u16,
    wpm: u64,
    legacy_wpm: bool,
    start_time: u64,
    time: Time,
    char_properly_typed: Vec<bool>,
    pub combo: usize,
    highest_combo: usize,
}

impl Stats {
    /// Create a new Stats struct
    pub fn new(legacy_wpm: bool) -> Self {
        Stats {
            errors: 0,
            wpm: 0,
            legacy_wpm,
            start_time: 0,
            time: Time::new(),
            char_properly_typed: Vec::new(),
            combo: 0,
            highest_combo: 0,
        }
    }

    /// Update the words per minute based on a words per minute algorithm
    /// If legacy is set to true, use the actual words per minute, otherwise use chars/5 per minute
    /// See: https://en.wikipedia.org/wiki/Words_per_minute#Alphanumeric_entry
    pub fn update_wpm(&mut self, word_idx: usize, word_vec: &[&str]) {
        let word_count_float = if self.legacy_wpm {
            // Get words per minute where a word is a set of characters delimited by a space.
            word_idx as f64
        } else {
            // Get words per minute where a word is 5 chars.
            // +1 for the space
            let char_count: usize = word_vec
                .iter()
                .take(word_idx)
                .map(|word| word.chars().count() + 1)
                .sum();
            char_count as f64 / 5.0
        };
        let minute_float = ((self.time.now() - self.start_time) as f64) / 60.0;
        self.wpm = (word_count_float / minute_float).ceil() as u64
    }

    /// Update the start time to the current time in seconds - 1
    pub fn update_start_time(&mut self) {
        if self.start_time == 0 {
            self.start_time = self.time.now() - 1;
        }
    }

    /// Increment the number of errors by 1
    pub fn increment_errors(&mut self, current_letter: usize) {
        self.errors += 1;
        self.reset_combo();
        self.update_accuracy(false, current_letter);
    }

    pub fn increment_combo(&mut self, current_letter: usize) {
        self.combo += 1;
        self.update_accuracy(true, current_letter);
        self.highest_combo = max(self.highest_combo, self.combo);
    }

    pub fn get_typing_accuracy(&self) -> f64 {
        let letter_count = self.char_properly_typed.len();
        let mut mistakes = 0;

        if letter_count == 0 {
            return 0.0;
        }

        for typed_correctly in self.char_properly_typed.iter() {
            if !typed_correctly {
                mistakes += 1;
            }
        }

        ((letter_count - mistakes) as f64 / letter_count as f64) * 100.0
    }

    fn update_accuracy(&mut self, error: bool, current_letter: usize) {
        if self.char_properly_typed.len() <= current_letter {
            self.char_properly_typed.push(error);
        } else {
            // If the user has made a mistake, it is forever, otherwise we are
            // allowed to update.
            // If we didn't, then accuracy would always be 100% at the end!
            if self.char_properly_typed[current_letter] {
                self.char_properly_typed[current_letter] = error;
            }
        }
    }

    fn reset_combo(&mut self) {
        self.combo = 0;
    }

    /// Reset the Stats struct to default values
    pub fn reset(&mut self) {
        self.wpm = 0;
        self.errors = 0;
        self.start_time = 0;
        self.combo = 0;
        self.char_properly_typed = Vec::new();
    }

    /// Create the vector of text elements
    pub fn text(&self) -> Vec<Vec<String>> {
        vec![
            vec!["WPM".to_string(), self.wpm.to_string()],
            vec!["Errors".to_string(), self.errors.to_string()],
            vec!["Combo".to_string(), self.combo.to_string()],
            vec![
                "Acc".to_string(),
                format!("{:.4}%", self.get_typing_accuracy().to_string()),
            ],
        ]
    }

    pub fn get_wpm(&self) -> u64 {
        self.wpm
    }

    pub fn get_highest_combo(&self) -> usize {
        self.highest_combo
    }

    /// Get the value of `legacy_wpm`
    pub fn get_legacy_wpm(&self) -> bool {
        self.legacy_wpm
    }

    /// Get the value of `start_time`
    pub fn get_start_time(&self) -> u64 {
        self.start_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_wpm() {
        let word_idx = 2;
        let word_vec = vec!["There's", "a", "time", "when", "the", "operation"];

        // Setup stats with testing start_time and time struct
        let mut stats = Stats::new(false);
        stats.start_time = 500;
        stats.time = Time { seconds: 501 };

        // Where a word is 5 characters, we know the user has typed 10 characters
        // in 1 second, which comes out to 120 wpm.
        stats.update_wpm(word_idx, &word_vec);
        assert_eq!(stats.wpm, 120);

        let word_idx = 4;
        stats.start_time = 500;
        stats.time = Time { seconds: 510 };

        // Where a word is 5 characters, we know the user has typed 20 characters in
        // 10 seconds. Which comes out to 24 wpm.
        stats.update_wpm(word_idx, &word_vec);
        assert_eq!(stats.wpm, 24);
    }

    #[test]
    fn test_legacy_get_wpm() {
        let word_idx = 2;

        // Setup stats with testing start_time and time struct
        let mut stats = Stats::new(true);
        stats.start_time = 500;
        stats.time = Time { seconds: 501 };

        // The user has typed 2 words in 1 second, which comes out to 120 wpm.
        stats.update_wpm(word_idx, &[]);
        assert_eq!(stats.wpm, 120);

        let word_idx = 4;
        stats.start_time = 500;
        stats.time = Time { seconds: 510 };

        // The user has typed 4 words in 10 seconds, which comes out to 24 wpm.
        stats.update_wpm(word_idx, &[]);
        assert_eq!(stats.wpm, 24);
    }

    #[test]
    fn test_errors() {
        let mut stats = Stats::new(false);
        assert_eq!(stats.errors, 0);
        stats.increment_errors(0);
        assert_eq!(stats.errors, 1);
    }

    #[test]
    fn test_wpm_is_first_text() {
        // wpm is the most important stat for the user so the table places it in the header row
        // due to this it assumes the wpm is the first item in the returned text vectors
        let stats = Stats::new(false);
        let text = stats.text();
        assert_eq!(text[0][0], "WPM");
    }

    #[test]
    fn test_properly_typed_vec_correct() {
        // does it give good values on increment_combo?
        let mut stats = Stats::new(false);

        let correct_vec = vec![true, true, true, true, true];

        stats.increment_combo(0);
        stats.increment_combo(1);
        stats.increment_combo(2);
        stats.increment_combo(3);
        stats.increment_combo(4);

        assert_eq!(correct_vec, stats.char_properly_typed);
        assert_eq_float(stats.get_typing_accuracy(), 100.0);
    }

    #[test]
    fn test_properly_typed_incorrect() {
        let mut stats = Stats::new(false);

        let correct_vec = vec![false, false, false, false, false];

        stats.increment_errors(0);
        stats.increment_errors(1);
        stats.increment_errors(2);
        stats.increment_errors(3);
        stats.increment_errors(4);

        assert_eq!(correct_vec, stats.char_properly_typed);
        assert_eq_float(stats.get_typing_accuracy(), 0.0);
    }

    #[test]
    fn test_no_overwrite_incorrect() {
        let mut stats = Stats::new(false);

        let correct_vec = vec![false, false, false];

        stats.increment_errors(0);
        stats.increment_errors(1);
        stats.increment_combo(1);
        stats.increment_combo(2);
        stats.increment_errors(2);
        stats.increment_combo(1);
        stats.increment_combo(2);

        assert_eq!(correct_vec, stats.char_properly_typed);
        assert_eq_float(stats.get_typing_accuracy(), 0.0);
    }

    fn assert_eq_float(v1: f64, v2: f64) {
        let error_margin = f64::EPSILON;
        assert!((v1 - v2).abs() < error_margin);
    }
}
