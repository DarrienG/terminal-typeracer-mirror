use tui::layout::Constraint;

const LARGE_LAYOUT: [Constraint; 4] = [
    Constraint::Percentage(20),
    Constraint::Percentage(40),
    Constraint::Percentage(10),
    Constraint::Percentage(30),
];

const LARGE_MARGIN: u16 = 5;

const MEDIUM_LAYOUT: [Constraint; 4] = [
    Constraint::Percentage(15),
    Constraint::Percentage(45),
    Constraint::Percentage(10),
    Constraint::Percentage(30),
];

const MEDIUM_MARGIN: u16 = 3;

const SMALL_LAYOUT: [Constraint; 4] = [
    Constraint::Percentage(0),
    Constraint::Percentage(50),
    Constraint::Percentage(10),
    Constraint::Percentage(40),
];

const SMALL_MARGIN: u16 = 1;

pub fn get_info_bounds(height: u16) -> [Constraint; 4] {
    match height {
        0..=24 => SMALL_LAYOUT,
        25..=32 => MEDIUM_LAYOUT,
        _ => LARGE_LAYOUT,
    }
}

pub fn get_info_margin(height: u16) -> u16 {
    match height {
        0..=24 => SMALL_MARGIN,
        25..=32 => MEDIUM_MARGIN,
        _ => LARGE_MARGIN,
    }
}
