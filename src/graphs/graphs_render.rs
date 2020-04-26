use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::symbols;
use tui::terminal::Terminal;
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType};

const DATA: [(f64, f64); 5] = [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (3.0, 3.0), (4.0, 4.0)];

pub fn render<B: Backend>(terminal: &mut Terminal<B>) {
    let term_size = terminal.size().expect("Unable to get terminal size");

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

            let datasets = [Dataset::default()
                .name("data")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Yellow))
                .graph_type(GraphType::Line)
                .data(&DATA)];

            f.render_widget(
                Chart::default()
                    .block(chart_block)
                    .x_axis(
                        Axis::default()
                            .title("X Axis")
                            .style(Style::default().fg(Color::Gray))
                            .labels_style(Style::default().modifier(Modifier::ITALIC))
                            .bounds([0.0, term_size.height.into()])
                            .labels(&["1", "2", "3", "4", "5"]),
                    )
                    .y_axis(
                        Axis::default()
                            .title("Y Axis")
                            .style(Style::default().fg(Color::Gray))
                            .labels_style(Style::default().modifier(Modifier::ITALIC))
                            .bounds([-20.0, 20.0])
                            .labels(&["-20", "0", "20"]),
                    )
                    .datasets(&datasets),
                padding_layout[1],
            );
        })
        .expect("Failed to draw to terminal");
}
