use crate::graphs::{Mode, UserResults};

pub fn relevant_data(user_results: &UserResults, mode: &Mode) -> f64 {
    match *mode {
        Mode::Wpm => user_results.wpm as f64,
        Mode::Accuracy => user_results.accuracy,
        Mode::Combo => user_results.highest_combo as f64,
    }
}
