use std::cmp::Ordering;
use tui::style::{Color, Style};

use crate::game::GameMode;
use crate::graphs::{Mode, UserResults};

pub struct YAxisData {
    pub bounds: [f64; 2],
    pub labels: [String; 3],
}

pub fn borders(game_mode: GameMode) -> Style {
    if game_mode == GameMode::InstantDeath {
        Style::default().fg(Color::Red)
    } else {
        Style::default()
    }
}

pub fn graph_title(mode: &Mode) -> String {
    match *mode {
        Mode::Wpm => "WPM".to_owned(),
        Mode::Accuracy => "Accuracy".to_owned(),
        Mode::Combo => "Combo".to_owned(),
    }
}

pub fn y_axis_data(mode: &Mode, user_results: &[UserResults]) -> YAxisData {
    let resorted_results = resorted_results(mode, user_results);

    let default_results = &UserResults::default();
    let first = resorted_results.first().unwrap_or(default_results);
    let last = resorted_results.last().unwrap_or(default_results);

    YAxisData {
        bounds: y_axis_bounds(mode, first, last),
        labels: y_axis_labels(mode, first, last),
    }
}

fn resorted_results(mode: &Mode, user_results: &[UserResults]) -> Vec<UserResults> {
    let mut resorted_results = user_results.to_vec();
    resorted_results.clone_from_slice(user_results);

    match *mode {
        Mode::Wpm => {
            resorted_results.sort_by(|a, b| a.wpm.partial_cmp(&b.wpm).unwrap_or(Ordering::Equal))
        }
        Mode::Accuracy => resorted_results.sort_by(|a, b| {
            a.accuracy
                .partial_cmp(&b.accuracy)
                .unwrap_or(Ordering::Equal)
        }),
        Mode::Combo => resorted_results.sort_by(|a, b| a.highest_combo.cmp(&b.highest_combo)),
    };

    resorted_results
}

fn y_axis_bounds(mode: &Mode, first: &UserResults, last: &UserResults) -> [f64; 2] {
    match *mode {
        Mode::Wpm => [0.0, last.wpm as f64 + 1.0],
        Mode::Accuracy => [first.accuracy - 1.0, last.accuracy],
        Mode::Combo => [0.0, last.highest_combo as f64],
    }
}

fn y_axis_labels(mode: &Mode, first: &UserResults, last: &UserResults) -> [String; 3] {
    match *mode {
        Mode::Wpm => [
            format!("{:.2}", first.wpm),
            format!("{:.2}", (first.wpm + last.wpm) / 2),
            format!("{:.2}", last.wpm),
        ],
        Mode::Accuracy => [
            format!("{:.2}", first.accuracy),
            format!("{:.2}", (first.accuracy + last.accuracy) / 2.0),
            format!("{:.2}", last.accuracy),
        ],

        Mode::Combo => [
            format!("{:.2}", first.highest_combo),
            format!("{:.2}", (first.highest_combo + last.highest_combo) / 2),
            format!("{:.2}", last.highest_combo),
        ],
    }
}
