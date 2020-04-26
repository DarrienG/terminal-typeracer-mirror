use std::io::stdin;
use termion::event::Key;
use termion::input::TermRead;
use tui::backend::Backend;
use tui::terminal::Terminal;

mod graphs_render;

pub fn show_graphs<B: Backend>(terminal: &mut Terminal<B>) {
    graphs_render::render(terminal);
    loop {
        let stdin = stdin();
        for c in stdin.keys() {
            if c.unwrap() == Key::Ctrl('c') {
                return;
            }
        }
    }
}
