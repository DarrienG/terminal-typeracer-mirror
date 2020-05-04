use rusqlite::{Connection, Result};
use std::io::stdin;
use std::path::PathBuf;
use termion::event::Key;
use termion::input::TermRead;
use tui::backend::Backend;
use tui::terminal::Terminal;

mod graphs_render;
mod user_result_mapper;

pub mod graphs_db;

#[derive(Clone)]
pub struct RawUserResults {
    wpm: i64,
    accuracy: f64,
    highest_combo: i64,
    when_played_secs: i64,
}

#[derive(Clone, Debug, Default)]
pub struct UserResults {
    wpm: i64,
    accuracy: f64,
    highest_combo: i64,
    days_back_played: f64,
}

pub enum Mode {
    Accuracy,
    Combo,
    Wpm,
}

const MODES: [Mode; 3] = [Mode::Wpm, Mode::Accuracy, Mode::Combo];

pub fn show_graphs<B: Backend>(
    terminal: &mut Terminal<B>,
    db_path: &PathBuf,
    instant_death_from_game: bool,
) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(db_path)?;

    let mut instant_death = instant_death_from_game;
    let mut current_mode = 0;

    loop {
        let stdin = stdin();
        let mut user_results = user_result_mapper::as_user_results(
            &graphs_db::aggregrate_graph_data(&conn, instant_death)?,
        );

        graphs_render::render(
            terminal,
            &mut user_results,
            instant_death,
            &MODES[current_mode],
        );

        for c in stdin.keys() {
            let keypress = c.unwrap();
            if keypress == Key::Ctrl('c') {
                return Ok(());
            } else if keypress == Key::Up || keypress == Key::Down {
                instant_death = !instant_death;
            } else if keypress == Key::Left {
                current_mode = decrement_current_mode(current_mode);
            } else if keypress == Key::Right {
                current_mode = increment_current_mode(current_mode);
            }

            break;
        }
    }
}

fn increment_current_mode(current_mode: usize) -> usize {
    (current_mode + 1) % MODES.len()
}

fn decrement_current_mode(current_mode: usize) -> usize {
    if current_mode == 0 {
        MODES.len() - 1
    } else {
        current_mode - 1
    }
}
