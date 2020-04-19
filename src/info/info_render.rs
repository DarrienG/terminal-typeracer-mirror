use info::InfoData;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::terminal::Terminal;
use tui::widgets::{Block, Borders, Paragraph};

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
        .draw(|mut f| {
            let root_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(get_info_bounds(term_size).as_ref())
                .margin(get_margin(term_size))
                .split(f.size());
            {
                let top_block = Block::default().borders(Borders::ALL);
                Paragraph::new(info_data.top_text.iter())
                    .block(top_block.clone().title("About page"))
                    .wrap(true)
                    .alignment(Alignment::Left);

                f.render_widget(top_block, root_layout[1]);

                if !info_data.bottom_text.is_empty() {
                    let bottom_block = Block::default().borders(Borders::NONE);
                    Paragraph::new(info_data.bottom_text.iter())
                        .block(bottom_block.clone())
                        .wrap(true)
                        .alignment(Alignment::Center);

                    f.render_widget(bottom_block, root_layout[3]);
                }
            }
        })
        .expect("Failed to draw terminal widgets.")
}
