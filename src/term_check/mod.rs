use std::io::{stdin, stdout, Error};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::Rect;
use tui::Terminal;

mod term_check_render;

fn need_input(rect: Rect, recommended_width: u16, recommended_height: u16) -> bool {
    rect.height < recommended_height || rect.width < recommended_width
}

pub fn resolution_check() -> Result<(), Error> {
    let stdout = stdout().into_raw_mode().expect("Unable to capture stdout");
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);
    let mut terminal = Terminal::new(backend)?;

    let recommended_height = 30;
    let recommended_width = 80;

    while need_input(
        terminal.size().unwrap(),
        recommended_width,
        recommended_height,
    ) {
        term_check_render::render(&mut terminal, recommended_width, recommended_height);
        let stdin = stdin();
        for c in stdin.keys() {
            let checked = c.unwrap();
            if checked == Key::Ctrl('c') {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "User wants to exit",
                ));
            }
            if checked == Key::Ctrl('d') {
                return Ok(());
            } else if checked == Key::Char('\n') {
                break;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::term_check;
    use tui::layout::Rect;

    #[test]
    fn test_need_input() {
        // Check with matching values
        assert!(!term_check::need_input(Rect::new(0, 0, 30, 80), 30, 80));

        // Check with one lower height but higher width
        assert!(term_check::need_input(Rect::new(0, 0, 29, 82), 30, 80));

        // Check with higher width, but lower height
        assert!(term_check::need_input(Rect::new(0, 0, 31, 79), 30, 80));

        // Check with higher recommended width, but lower recommended height
        assert!(term_check::need_input(Rect::new(0, 0, 30, 80), 29, 81));

        // Check with higher recommended height, but lower recommended height
        assert!(term_check::need_input(Rect::new(0, 0, 30, 80), 31, 79));
    }
}
