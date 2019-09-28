use tui::layout::Constraint;

const LARGE_TYPING_LAYOUT: [Constraint; 4] = [
    Constraint::Percentage(0),
    Constraint::Percentage(30),
    Constraint::Percentage(40),
    Constraint::Percentage(30),
];

const LARGE_STATS_LAYOUT: [Constraint; 3] = [
    Constraint::Percentage(10),
    Constraint::Percentage(20),
    Constraint::Percentage(70),
];

const MEDIUM_TYPING_LAYOUT: [Constraint; 4] = [
    Constraint::Percentage(0),
    Constraint::Percentage(20),
    Constraint::Percentage(50),
    Constraint::Percentage(30),
];

const MEDIUM_STATS_LAYOUT: [Constraint; 3] = [
    Constraint::Percentage(20),
    Constraint::Percentage(0),
    Constraint::Percentage(80),
];

const SMALL_TYPING_LAYOUT: [Constraint; 4] = [
    Constraint::Percentage(0),
    Constraint::Percentage(0),
    Constraint::Percentage(70),
    Constraint::Percentage(30),
];

const SMALL_STATS_LAYOUT: [Constraint; 3] = [
    Constraint::Percentage(0),
    Constraint::Percentage(0),
    Constraint::Percentage(100),
];

pub fn get_typing_bounds(height: u16) -> [Constraint; 4] {
    match height {
        0..=24 => SMALL_TYPING_LAYOUT,
        25..=32 => MEDIUM_TYPING_LAYOUT,
        _ => LARGE_TYPING_LAYOUT,
    }
}

pub fn get_stats_bounds(height: u16) -> [Constraint; 3] {
    match height {
        0..=24 => SMALL_STATS_LAYOUT,
        25..=32 => MEDIUM_STATS_LAYOUT,
        _ => LARGE_STATS_LAYOUT,
    }
}
