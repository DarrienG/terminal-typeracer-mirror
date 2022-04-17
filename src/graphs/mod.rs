use crossbeam_channel::Receiver;
use rusqlite::{Connection, Result};
use std::path::Path;
use std::time::Duration;
use termion::event::Key;
use tui::{backend::Backend, terminal::Terminal};

use crate::game::GameMode;

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
    input_receiver: &Receiver<Key>,
    db_path: &Path,
    game_mode_from_game: GameMode,
) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(db_path)?;

    let mut game_mode = game_mode_from_game;
    let mut current_mode = 0;

    loop {
        let user_results = user_result_mapper::as_user_results(&graphs_db::aggregrate_graph_data(
            &conn, game_mode,
        )?);

        graphs_render::render(terminal, &user_results, game_mode, &MODES[current_mode]);

        // async, but only in name. We don't care to do any extra renders while waiting on user input
        let recv_result = input_receiver.recv_timeout(Duration::from_secs(180));
        if recv_result.is_err() {
            continue;
        }

        match recv_result.unwrap() {
            Key::Ctrl('c') => return Ok(()),
            Key::Up => game_mode = game_mode.prev(),
            Key::Down => game_mode = game_mode.next(),
            Key::Left => current_mode = decrement_current_mode(current_mode),
            Key::Right => current_mode = increment_current_mode(current_mode),
            _ => (),
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
