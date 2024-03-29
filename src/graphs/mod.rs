use crossbeam_channel::Receiver;
use rusqlite::{Connection, Result};
use std::collections::HashMap;
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

    let mut results_map = HashMap::new();
    let mut terminal_size = (0, 0);
    let mut should_re_render = true;
    loop {
        let results = match results_map.get(&game_mode) {
            Some(results) => results,
            None => {
                results_map.insert(
                    game_mode,
                    user_result_mapper::as_user_results(&graphs_db::aggregrate_graph_data(
                        &conn, game_mode,
                    )?),
                );
                &results_map[&game_mode]
            }
        };

        // terminal size is updated so we should re-render
        let updated_terminal_size = terminal_length_width();
        if terminal_size != updated_terminal_size {
            should_re_render = true;
            terminal_size = updated_terminal_size;
        }

        // rendering graphs is a little extra expensive, let's not re-render if we don't have to
        if should_re_render {
            graphs_render::render(terminal, results, game_mode, &MODES[current_mode]);
            should_re_render = false;
        }

        // slow re-render. Graphs are a little more intensive to render so we should only do it every so often.
        // With that said we want to support terminal re-size
        let recv_result = input_receiver.recv_timeout(Duration::from_secs(1));
        if recv_result.is_err() {
            continue;
        }

        let key = recv_result.unwrap();
        // the user entered something of importance, let's re-render
        match key {
            Key::Up | Key::Down | Key::Left | Key::Right => should_re_render = true,
            _ => (),
        }
        match key {
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

fn terminal_length_width() -> (u16, u16) {
    termion::terminal_size().expect("Unable to get terminal size")
}
