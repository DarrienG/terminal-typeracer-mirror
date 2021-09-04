use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    terminal::Terminal,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
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
                .border_style(styles::borders(game_mode));

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
                .data(&filtered_results)]
            .to_vec();

            let y_axis_data = styles::y_axis_data(active_mode, ordered_user_results);

            f.render_widget(
                Chart::new(datasets)
                    .block(chart_block)
                    .x_axis(
                        Axis::default()
                            .title("Progress since last play")
                            .style(
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::ITALIC),
                            )
                            .bounds([
                                -ordered_user_results.first().unwrap().days_back_played,
                                ordered_user_results.last().unwrap().days_back_played,
                            ])
                            .labels(vec![
                                Span::raw(day_prettifier::num_to_day(days_played_for)),
                                Span::raw(day_prettifier::num_to_day(data_increment * 3.0)),
                                Span::raw(day_prettifier::num_to_day(data_increment * 2.0)),
                                Span::raw(day_prettifier::num_to_day(data_increment * 1.0)),
                                Span::raw(day_prettifier::num_to_day(0.0)),
                            ]),
                    )
                    .y_axis(
                        Axis::default()
                            .title(Span::raw(&styles::graph_title(active_mode)))
                            .style(
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::ITALIC),
                            )
                            .bounds(y_axis_data.bounds)
                            .labels(
                                y_axis_data
                                    .labels
                                    .iter()
                                    .map(|label| Span::raw(label))
                                    .collect::<Vec<Span>>(),
                            ),
                    ),
                main_layout[1],
            );

            f.render_widget(
                Paragraph::new(Span::raw(
                    "^C to go back  ⇕ cycle game mode  ⇔ switch graph",
                ))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE)),
                main_layout[2],
            );
        })
        .expect("Failed to draw to terminal");
}
