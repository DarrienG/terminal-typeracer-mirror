use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::Style;
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Terminal;

pub fn render<B: Backend>(
    terminal: &mut Terminal<B>,
    recommended_height: u16,
    recommended_width: u16,
) {
    let dimens = terminal.size().unwrap();
    let width = dimens.width;
    let height = dimens.height;
    terminal.draw(|mut f| {
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
            Paragraph::new(
                [Text::raw(format!(
                    "Terminal width and height too small!\nwidth: {}\nheight: {}\n\nIt is strongly recommended to play this with a width of at least: {} and a height of at least: {}\nConsider making your terminal fullscreen!\n\nCheck again <ENTER>, Ignore check: ^D, Exit: ^C",
                    width,
                    height,
                    recommended_width,
                    recommended_height,
                ))]
                .iter(),
            )
            .block(passage_block.clone().title("Checking bounds"))
            .wrap(true)
            .alignment(Alignment::Left)
            .render(&mut f, chunks[0]);
    }).expect("Failed to write to terminal");
}
