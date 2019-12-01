use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::Color;
use tui::style::{Modifier, Style};
use tui::terminal::Terminal;
use tui::widgets::{Block, Borders, Paragraph, Row, Table, Text, Widget};

use crate::game::FormattedTexts;
use crate::stats;

mod styles;

#[derive(Clone, Debug)]
pub struct GameState<'a> {
    pub texts: &'a FormattedTexts<'a>,
    pub user_input: &'a str,
    pub stats: &'a stats::Stats,
    pub title: &'a str,
    pub instant_death: bool,
    // For debug
    pub debug_enabled: bool,
    pub word_idx: usize,
    pub passage_path: &'a str,
    pub current_word: &'a str,
}

impl<'a> GameState<'a> {
    fn get_debug_output(&self) -> String {
        format!("Running with options:\n Legacy WPM: {},  word_idx: {},  start: {}\nUser has err: {},  instant_death_enabled: {}\npassage_path: {}\ncurrent_word: {}",
                self.stats.get_legacy_wpm(),
                self.word_idx,
                self.stats.get_start_time(),
                self.texts.error,
                self.instant_death,
                self.passage_path,
                self.current_word,
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

fn get_border_style(game_state: &GameState) -> Style {
    Style::default().fg(if game_state.instant_death {
        Color::Red
    } else if game_state.stats.errors == 0 {
        Color::Green
    } else {
        Color::Reset
    })
}

pub fn render<B: Backend>(
    terminal: &mut Terminal<B>,
    game_state: GameState,
    typeracer_version: &str,
) {
    let term_size = terminal.size().unwrap();
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
                            .border_style(get_border_style(&game_state))
                            .title_style(Style::default());
                        Paragraph::new(vec![Text::raw(game_state.get_debug_output())].iter())
                            .block(debug_block.clone().title("DEBUG ENABLED"))
                            .wrap(true)
                            .alignment(Alignment::Left)
                            .render(&mut f, chunks[1]);
                    }
                    let passage_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(get_border_style(&game_state))
                        .title_style(Style::default());
                    Paragraph::new(game_state.texts.passage.iter())
                        .block(passage_block.clone().title(&game_state.title))
                        .wrap(true)
                        .alignment(Alignment::Left)
                        .render(&mut f, chunks[2]);

                    let typing_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(get_border_style(&game_state))
                        .title_style(Style::default().modifier(Modifier::BOLD));

                    let style = if game_state.texts.error {
                        Style::default().bg(Color::Red).fg(Color::White)
                    } else if game_state.texts.complete {
                        Style::default().bg(Color::Green).fg(Color::White)
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
                        .constraints(get_stats_bounds(term_size).as_ref())
                        .split(root_layout[1]);

                    let stats_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(get_border_style(&game_state))
                        .title_style(Style::default());

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
                    Table::new(headers, rows)
                        .block(stats_block.clone().title("Stats"))
                        .widths(&[stat_column_width, value_column_width])
                        .column_spacing(1)
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
                Paragraph::new(
                    [
                        Text::raw(
                            "^C exit  ^U clear word  ^R[estart]  ^N[ext]  ^P[revious]  ^A[bout]\n",
                        ),
                        Text::styled(
                            format!("Build: {}", typeracer_version),
                            Style::default().fg(Color::Gray),
                        ),
                    ]
                    .iter(),
                )
                .block(shortcut_block.clone())
                .alignment(Alignment::Center)
                .render(&mut f, chunks[0]);
            }
        })
        .expect("Failed to draw terminal widgets.");
}
