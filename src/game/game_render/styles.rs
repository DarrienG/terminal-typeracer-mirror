use tui::layout::Constraint;
use tui::style::{Color, Style};

use crate::game::game_render::GameState;

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

pub fn instant_death_border_style(game_state: &GameState) -> Style {
    if game_state.config.display_settings.simple_borders && !game_state.instant_death {
        return Style::default().fg(Color::Red);
    }

    Style::default().fg(
        if game_state.stats.combo >= game_state.config.combo_config.combo_trigger {
            Color::Magenta
        } else {
            Color::Red
        },
    )
}

pub fn regular_border_style(game_state: &GameState) -> Style {
    if game_state.config.display_settings.simple_borders && !game_state.instant_death {
        return Style::default().fg(Color::Reset);
    }

    Style::default().fg(if game_state.stats.errors == 0 {
        if game_state.stats.combo >= game_state.config.combo_config.combo_trigger {
            Color::Cyan
        } else {
            Color::Green
        }
    } else if game_state.stats.combo >= game_state.config.combo_config.combo_trigger {
        Color::Blue
    } else {
        Color::Reset
    })
}
