use info::InfoData;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    terminal::Terminal,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::info;

mod styles;

/// Convenience method for retrieving constraints for the info blocks
fn get_info_bounds(rect: Rect) -> [Constraint; 4] {
    styles::get_info_bounds(rect.height)
}

fn get_margin(rect: Rect) -> u16 {
    styles::get_info_margin(rect.height)
}

pub fn render<B: Backend>(terminal: &mut Terminal<B>, info_data: &InfoData) {
    let term_size = terminal.size().unwrap();
    terminal
        .draw(|f| {
            let root_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(get_info_bounds(term_size).as_ref())
                .margin(get_margin(term_size))
                .split(f.size());
            {
                let top_block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded);

                f.render_widget(
                    Paragraph::new(info_data.top_text.clone())
                        .block(top_block.title("About/docs page"))
                        .wrap(Wrap { trim: true })
                        .alignment(Alignment::Left),
                    root_layout[1],
                );

                let bottom_block = Block::default().borders(Borders::NONE);

                f.render_widget(
                    Paragraph::new(info_data.bottom_text.clone())
                        .block(bottom_block)
                        .wrap(Wrap { trim: true })
                        .alignment(Alignment::Center),
                    root_layout[3],
                );
            }
        })
        .expect("Failed to draw terminal widgets.");
}
