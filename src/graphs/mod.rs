use rusqlite::{Connection, Result};
use std::io::stdin;
use std::path::PathBuf;
use termion::event::Key;
use termion::input::TermRead;
use tui::backend::Backend;
use tui::terminal::Terminal;

mod graphs_render;

pub mod graphs_db;

#[derive(Clone)]
pub struct UserResults {
    wpm: i64,
    accuracy: f64,
    highest_combo: i64,
    when_played_secs: i64,
}

pub fn show_graphs<B: Backend>(
    terminal: &mut Terminal<B>,
    db_path: &PathBuf,
    instant_death: bool,
) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(db_path)?;

    let mut user_results = graphs_db::aggregrate_graph_data(&conn, instant_death)?;

    graphs_render::render(terminal, &mut user_results);

    loop {
        let stdin = stdin();
        for c in stdin.keys() {
            if c.unwrap() == Key::Ctrl('c') {
                return Ok(());
            }
        }
    }
}
