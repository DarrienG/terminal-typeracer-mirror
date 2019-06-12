use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::Color;
use tui::style::{Modifier, Style};
use tui::terminal::Terminal;
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};

use crate::game::FormattedTexts;
use crate::stats;

#[derive(Clone, Debug)]
pub struct GameState<'a> {
    pub texts: &'a FormattedTexts<'a>,
    pub user_input: &'a str,
    pub stats: &'a stats::Stats,
    pub title: &'a str,
    // For debug
    pub debug_enabled: bool,
    pub word_idx: usize,
    pub passage_path: &'a str,
    pub current_word: &'a str,
}

impl<'a> GameState<'a> {
    fn get_debug_output(&self) -> String {
        format!("Running with options:\n Legacy WPM: {},  word_idx: {},  start: {}\npassage_path: {}\ncurrent_word: {}",
                self.stats.get_legacy_wpm(),
                self.word_idx,
                self.stats.get_start_time(),
                self.passage_path,
                self.current_word,
            )
    }
}

// Convenience method for retrieving constraints for the typing layout.
// At some point this may be refactored to be more dynamic based on
// terminal layout size so we can skip resolution checks.
fn get_typing_bounds() -> [Constraint; 4] {
    [
        Constraint::Percentage(20),
        Constraint::Percentage(30),
        Constraint::Percentage(30),
        Constraint::Percentage(20),
    ]
}

// Convenience method for retrieving constraints for the stats block.
// At some point this may be refactored to be more dynamic based on
// terminal layout size so we can skip resolution checks.
fn get_stats_bounds() -> [Constraint; 3] {
    [
        Constraint::Percentage(20),
        Constraint::Percentage(30),
        Constraint::Percentage(60),
    ]
}

pub fn render<B: Backend>(terminal: &mut Terminal<B>, game_state: GameState) {
    terminal
        .draw(|mut f| {
            // Because there is no way to specify vertical but not horizontal margins
            let padding_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(5),
                        Constraint::Percentage(90),
                        Constraint::Percentage(5),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let base_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                .split(padding_layout[1]);
            {
                let root_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                    .split(base_layout[0]);
                {
                    // Typing layout
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints(get_typing_bounds().as_ref())
                        .split(root_layout[0]);
                    if game_state.debug_enabled {
                        let debug_block = Block::default()
                            .borders(Borders::ALL)
                            .title_style(Style::default());
                        Paragraph::new(vec![Text::raw(game_state.get_debug_output())].iter())
                            .block(debug_block.clone().title("DEBUG ENABLED"))
                            .wrap(true)
                            .alignment(Alignment::Left)
                            .render(&mut f, chunks[1]);
                    }
                    let passage_block = Block::default()
                        .borders(Borders::ALL)
                        .title_style(Style::default());
                    Paragraph::new(game_state.texts.passage.iter())
                        .block(passage_block.clone().title(&game_state.title))
                        .wrap(true)
                        .alignment(Alignment::Left)
                        .render(&mut f, chunks[2]);

                    let typing_block = Block::default()
                        .borders(Borders::ALL)
                        .title_style(Style::default().modifier(Modifier::BOLD));

                    let style = if game_state.texts.error {
                        Style::default().bg(Color::Red).fg(Color::White)
                    } else {
                        Style::default()
                    };

                    Paragraph::new(game_state.texts.input.iter())
                        .block(typing_block.clone().title("Type out passage here"))
                        .wrap(true)
                        .alignment(Alignment::Left)
                        .style(style)
                        .render(&mut f, chunks[3]);
                }
                {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints(get_stats_bounds().as_ref())
                        .split(root_layout[1]);

                    let stats_block = Block::default()
                        .borders(Borders::ALL)
                        .title_style(Style::default());
                    Paragraph::new(game_state.stats.text().iter())
                        .block(stats_block.clone().title("Stats"))
                        .alignment(Alignment::Center)
                        .render(&mut f, chunks[2]);
                }
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(base_layout[1]);

                let shortcut_block = Block::default()
                    .borders(Borders::NONE)
                    .title_style(Style::default());
                Paragraph::new([Text::raw("^C exit  ^N next passage  ^U clear word")].iter())
                    .block(shortcut_block.clone())
                    .alignment(Alignment::Center)
                    .render(&mut f, chunks[0]);
            }
        })
        .expect("Failed to draw terminal widgets.");
}
