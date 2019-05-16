use std::io::{stdin, stdout, Error, Write};
use termion::cursor::Goto;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Terminal;
use unicode_width::UnicodeWidthStr;

// TODO: Calculate constraints based on terminal size
// e.g. smaller terminal means smaller padding on top and bottom

fn main() -> Result<(), Error> {
    let stdout = stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut user_input = String::new();

    loop {
        let stdin = stdin();
        terminal
            .draw(|mut f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(10)
                    .constraints(
                        [
                            Constraint::Percentage(20),
                            Constraint::Percentage(30),
                            Constraint::Percentage(30),
                            Constraint::Percentage(20),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());
                Block::default()
                    .title("The text passage be here")
                    .borders(Borders::ALL)
                    .render(&mut f, chunks[2]);

                let block = Block::default()
                    .borders(Borders::ALL)
                    .title_style(Style::default().modifier(Modifier::BOLD));
                Paragraph::new([Text::raw(&user_input)].iter())
                    .block(block.clone().title("Fellas type here"))
                    .alignment(Alignment::Left)
                    .render(&mut f, chunks[3]);
            })
            .unwrap();

        write!(
            terminal.backend_mut(),
            "{}",
            Goto(4 + user_input.width() as u16, 4)
        )?;

        for c in stdin.keys() {
            let key_code = c.unwrap();
            if key_code == Key::Ctrl('c') {
                return Ok(());
            } else if key_code == Key::Backspace {
                user_input.pop().unwrap();
            } else {
                //user_input.push(key_code(k));
            }
        }
    }
}
