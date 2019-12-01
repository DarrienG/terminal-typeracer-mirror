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
    pub fn increment_errors(&mut self) {
        self.errors += 1
    }

    /// Reset the Stats struct to default values
    pub fn reset(&mut self) {
        self.wpm = 0;
        self.errors = 0;
        self.start_time = 0;
    }

    /// Create the vector of text elements
    pub fn text(&self) -> Vec<Vec<String>> {
        vec![
            vec!["WPM".to_string(), self.wpm.to_string()],
            vec!["Errors".to_string(), self.errors.to_string()],
        ]
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
        stats.increment_errors();
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
}
