use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    terminal::Terminal,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Text},
};

use crate::game::GameMode;
use crate::graphs::{Mode, UserResults};

mod dataset;

mod day_prettifier;
mod styles;

pub fn render<B: Backend>(
    terminal: &mut Terminal<B>,
    ordered_user_results: &[UserResults],
    game_mode: GameMode,
    active_mode: &Mode,
) {
    let days_played_for = match ordered_user_results.first() {
        Some(s) => s.days_back_played,
        None => 0.0,
    };

    let data_increment = days_played_for / 4.0;

    terminal
        .draw(|mut f| {
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

            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(5),
                        Constraint::Percentage(80),
                        Constraint::Percentage(10),
                        Constraint::Percentage(5),
                    ]
                    .as_ref(),
                )
                .split(padding_layout[1]);

            let chart_block = Block::default()
                .borders(Borders::ALL)
                .border_style(styles::borders(game_mode))
                .title_style(Style::default());

            let filtered_results: Vec<(f64, f64)> = (*ordered_user_results)
                .iter()
                .map(|result| {
                    (
                        -result.days_back_played,
                        dataset::relevant_data(&result, &active_mode),
                    )
                })
                .collect::<Vec<(f64, f64)>>();

            let datasets = [Dataset::default()
                .name(format!(
                    "{}: {} over time",
                    game_mode,
                    styles::graph_title(&active_mode)
                ))
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Yellow))
                .graph_type(GraphType::Line)
                .data(&filtered_results)];

            let y_axis_data = styles::y_axis_data(active_mode, ordered_user_results);

            f.render_widget(
                Chart::default()
                    .block(chart_block)
                    .x_axis(
                        Axis::default()
                            .title("Progress since last play")
                            .style(Style::default().fg(Color::Gray))
                            .labels_style(Style::default().modifier(Modifier::ITALIC))
                            .bounds([
                                -ordered_user_results.first().unwrap().days_back_played,
                                ordered_user_results.last().unwrap().days_back_played,
                            ])
                            .labels(&[
                                day_prettifier::num_to_day(days_played_for),
                                day_prettifier::num_to_day(data_increment * 3.0),
                                day_prettifier::num_to_day(data_increment * 2.0),
                                day_prettifier::num_to_day(data_increment * 1.0),
                                day_prettifier::num_to_day(0.0),
                            ]),
                    )
                    .y_axis(
                        Axis::default()
                            .title(&styles::graph_title(active_mode))
                            .style(Style::default().fg(Color::Gray))
                            .labels_style(Style::default().modifier(Modifier::ITALIC))
                            .bounds(y_axis_data.bounds)
                            .labels(&y_axis_data.labels),
                    )
                    .datasets(&datasets),
                main_layout[1],
            );

            f.render_widget(
                Paragraph::new(
                    &mut [Text::raw(
                        "^C to go back  ⇕ cycle game mode  ⇔ switch graph",
                    )]
                    .iter(),
                )
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE)),
                main_layout[2],
            );
        })
        .expect("Failed to draw to terminal");
}
