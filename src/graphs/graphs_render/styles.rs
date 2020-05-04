use tui::style::{Color, Style};

use crate::graphs::Mode;

pub fn borders(instant_death: bool) -> Style {
    if instant_death {
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

pub fn y_axis_bounds(mode: &Mode) -> [f64; 2] {
    match *mode {
        Mode::Wpm => [0.0, 216.0],
        Mode::Accuracy => [80.0, 100.0],
        Mode::Combo => [0.0, 128.0],
    }
}

pub fn y_axis_labels(mode: &Mode) -> [String; 3] {
    match *mode {
        Mode::Wpm => ["0".to_owned(), "100".to_owned(), "216".to_owned()],
        Mode::Accuracy => ["80".to_owned(), "90".to_owned(), "100".to_owned()],
        Mode::Combo => ["0".to_owned(), "64".to_owned(), "128".to_owned()],
    }
}
