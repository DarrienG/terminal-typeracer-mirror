use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    terminal::Terminal,
    widgets::{Block, Borders, Paragraph, Text},
};

pub fn render<B: Backend>(terminal: &mut Terminal<B>, step_instruction: &str) {
    terminal
        .draw(|mut f| {
            let root_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(5)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(root_layout[0]);
            let passage_block = Block::default()
                .borders(Borders::ALL)
                .title_style(Style::default());
            f.render_widget(
                Paragraph::new([Text::raw(step_instruction)].iter())
                    .block(passage_block.title("Installing quote packs"))
                    .wrap(true)
                    .alignment(Alignment::Left),
                chunks[0],
            );
        })
        .expect("Failed to write to term");
}
