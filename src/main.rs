use std::io::{stdin, stdout, Error, Write};
use termion::cursor::{Left, Right};
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
fn get_bounds() -> [Constraint; 4] {
    [
        Constraint::Percentage(20),
        Constraint::Percentage(30),
        Constraint::Percentage(30),
        Constraint::Percentage(20),
    ]
}

// TODO: Read in passage from somewhere
fn get_passage() -> String {
    "The quick brown fox jumps over the lazy dog".to_owned()
}

fn main() -> Result<(), Error> {
    let stdout = stdout().into_raw_mode()?;
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);
    let mut terminal = Terminal::new(backend)?;

    let mut user_input = String::new();
    let mut raw_passage = get_passage();

    loop {
        let stdin = stdin();

        terminal
            .draw(|mut f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(10)
                    .constraints(get_bounds().as_ref())
                    .split(f.size());
                let passage_block = Block::default()
                    .borders(Borders::ALL)
                    .title_style(Style::default());
                Paragraph::new([Text::raw(&raw_passage)].iter())
                    .block(passage_block.clone().title("Passage to type"))
                    .alignment(Alignment::Left)
                    .render(&mut f, chunks[2]);

                let typing_block = Block::default()
                    .borders(Borders::ALL)
                    .title_style(Style::default().modifier(Modifier::BOLD));
                Paragraph::new([Text::raw(&user_input)].iter())
                    .block(typing_block.clone().title("Fellas type here"))
                    .alignment(Alignment::Left)
                    .render(&mut f, chunks[3]);
            })
            .unwrap();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Ctrl('c') => return Ok(()),
                Key::Backspace => {
                    user_input.pop();
                    if user_input.chars().count() > 0 {
                        write!(terminal.backend_mut(), "{}", Left(1))?;
                    }
                    break;
                }
                Key::Char(c) => {
                    user_input.push(c);
                    write!(terminal.backend_mut(), "{}", Right(1))?;
                    break;
                }
                _ => {
                    break;
                }
            }
        }
    }
}
