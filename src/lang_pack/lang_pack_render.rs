use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    terminal::Terminal,
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

pub fn render<B: Backend>(terminal: &mut Terminal<B>, step_instruction: Text) {
    terminal
        .draw(|f| {
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
                .border_type(BorderType::Rounded);
            f.render_widget(
                Paragraph::new(step_instruction)
                    .block(passage_block.title("Installing quote packs"))
                    .wrap(Wrap { trim: true })
                    .alignment(Alignment::Left),
                chunks[0],
            );
        })
        .expect("Failed to write to term");
}
