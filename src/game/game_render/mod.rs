use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Color,
    style::{Modifier, Style},
    terminal::Terminal,
    text::Text,
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use crate::{
    config::TyperacerConfig,
    game::{formatter::FormattedTexts, GameMode},
    stats,
};
use std::collections::HashSet;

mod styles;

#[derive(Clone, Debug)]
pub struct GameState<'a> {
    pub texts: &'a FormattedTexts<'a>,
    pub user_input: &'a str,
    pub stats: &'a stats::Stats,
    pub title: &'a str,
    pub game_mode: GameMode,
    pub config: &'a TyperacerConfig,
    // For debug
    pub debug_enabled: bool,
    pub word_idx: usize,
    pub passage_path: &'a str,
    pub current_word: &'a str,
    pub mistaken_words: &'a HashSet<String>,
    pub complete: bool,
}

impl<'a> GameState<'a> {
    fn get_debug_output(&self) -> String {
        format!("Running with options:\n Legacy WPM: {},  word_idx: {},  start: {}\nUser has err: {}, game mode: {}, num words: {}, complete: {}\npassage_path: {}\ncurrent_word: {}\nmistaken_words: {:?}",
            self.stats.get_legacy_wpm(),
            self.word_idx,
            self.stats.get_start_time(),
            self.texts.error,
            self.game_mode,
            self.texts.passage.len(),
            self.complete,
            self.passage_path,
            self.current_word,
            self.mistaken_words
        )
    }
}

/// Convenience method for retrieving constraints for the typing layout.
fn get_typing_bounds(rect: Rect) -> [Constraint; 4] {
    styles::get_typing_bounds(rect.height)
}

/// Convenience method for retrieving constraints for the stats block.
fn get_stats_bounds(rect: Rect) -> [Constraint; 3] {
    styles::get_stats_bounds(rect.height)
}

fn get_border_style(game_state: &GameState, modifiers: &[Modifier]) -> Style {
    if game_state.game_mode == GameMode::InstantDeath {
        styles::instant_death_border_style(game_state, modifiers)
    } else {
        styles::regular_border_style(game_state, modifiers)
    }
}

pub fn render<B: Backend>(
    terminal: &mut Terminal<B>,
    game_state: GameState,
    typeracer_version: &str,
) {
    let term_size = terminal.size().expect("Unable to get terminal size");
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
                        .constraints(get_typing_bounds(term_size).as_ref())
                        .split(root_layout[0]);
                    if game_state.debug_enabled {
                        let debug_block = Block::default()
                            .borders(Borders::ALL)
                            .border_style(get_border_style(&game_state, &[]));
                        f.render_widget(
                            Paragraph::new(vec![Text::raw(game_state.get_debug_output())].iter())
                                .block(debug_block.title("DEBUG ENABLED"))
                                .wrap(true)
                                .alignment(Alignment::Left),
                            chunks[1],
                        );
                    }
                    let passage_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(get_border_style(&game_state, &[]));

                    f.render_widget(
                        Paragraph::new(game_state.texts.passage.iter())
                            .block(passage_block.title(&game_state.title))
                            .wrap(true)
                            .alignment(Alignment::Left),
                        chunks[2],
                    );

                    let typing_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(get_border_style(&game_state, &[Modifier::BOLD]));

                    let style = if game_state.texts.error {
                        Style::default().bg(Color::Red).fg(Color::White)
                    } else if game_state.texts.complete {
                        Style::default().bg(Color::Green).fg(Color::White)
                    } else {
                        Style::default()
                    };

                    f.render_widget(
                        Paragraph::new(game_state.texts.input.iter())
                            .block(typing_block.title("Type out passage here"))
                            .wrap(true)
                            .alignment(Alignment::Left)
                            .style(style),
                        chunks[3],
                    );
                }
                {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints(get_stats_bounds(term_size).as_ref())
                        .split(root_layout[1]);

                    let stats_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(get_border_style(&game_state, &[]));

                    let stats_text = game_state.stats.text();
                    let headers = stats_text
                        .get(0)
                        .expect("Stats produced no text elements")
                        .iter();
                    let rows = stats_text
                        .iter()
                        .skip(1)
                        .map(|name_value_vec| Row::Data(name_value_vec.iter()));
                    // set to be the width of the longest name in the stats
                    let stat_column_width = 5;
                    // set to be an arbitrary value, 5 digits should be plenty to show the values for now
                    let value_column_width = 5;
                    f.render_widget(
                        Table::new(headers, rows)
                            .block(stats_block.title("Stats"))
                            .widths(&[
                                Constraint::Length(stat_column_width),
                                Constraint::Length(value_column_width),
                            ])
                            .column_spacing(1),
                        chunks[2],
                    );
                }

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(base_layout[1]);

                let shortcut_block = Block::default()
                    .borders(Borders::NONE);
                f.render_widget(
                Paragraph::new(
                    [
                        Text::raw(
                            "^C exit  ^U clear word  ^R[estart]  ^N[ext]  ^P[revious]\n^G[raphs]  ^A[bout/docs]\n",
                        ),
                        Text::styled(
                            format!("Build: {}", typeracer_version),
                            Style::default().fg(Color::Gray),
                        ),
                    ]
                    .iter(),
                )
                .block(shortcut_block)
                .alignment(Alignment::Center), chunks[0]);
            }
        })
        .expect("Failed to draw terminal widgets.");
}
