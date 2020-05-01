use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::symbols;
use tui::terminal::Terminal;
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType};

use crate::graphs::UserResults;

pub fn render<B: Backend>(terminal: &mut Terminal<B>, ordered_user_results: &[UserResults]) {
    let lowest_value = match ordered_user_results.first() {
        Some(s) => s.when_played_secs,
        None => 0,
    } as f64;

    let highest_value = match ordered_user_results.last() {
        Some(s) => s.when_played_secs,
        None => 0,
    } as f64;

    let data_increment = (highest_value - lowest_value) / 4.0;

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

            let chart_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default())
                .title_style(Style::default());

            let filtered_results: Vec<(f64, f64)> = (*ordered_user_results)
                .iter()
                .map(|result| (result.when_played_secs as f64, result.wpm as f64))
                .collect::<Vec<(f64, f64)>>();

            let datasets = [Dataset::default()
                .name("User results")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Yellow))
                .graph_type(GraphType::Line)
                .data(&filtered_results)];

            f.render_widget(
                Chart::default()
                    .block(chart_block)
                    .x_axis(
                        Axis::default()
                            .title("X Axis")
                            .style(Style::default().fg(Color::Gray))
                            .labels_style(Style::default().modifier(Modifier::ITALIC))
                            .bounds([lowest_value, highest_value])
                            .labels(&[
                                format!("{}", lowest_value),
                                format!("{}", lowest_value + data_increment * 1.0),
                                format!("{}", lowest_value + data_increment * 2.0),
                                format!("{}", lowest_value + data_increment * 3.0),
                                format!("{}", highest_value),
                            ]),
                    )
                    .y_axis(
                        Axis::default()
                            .title("WPM")
                            .style(Style::default().fg(Color::Gray))
                            .labels_style(Style::default().modifier(Modifier::ITALIC))
                            .bounds([0.0, 216.0])
                            .labels(&["0", "100", "216"]),
                    )
                    .datasets(&datasets),
                padding_layout[1],
            );
        })
        .expect("Failed to draw to terminal");
}
