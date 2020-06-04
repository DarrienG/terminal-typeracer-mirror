use rusqlite::{params, Connection, Result};

use crate::game::GameMode;
use crate::graphs::RawUserResults;

pub fn aggregrate_graph_data(
    conn: &Connection,
    game_mode: GameMode,
) -> Result<Vec<RawUserResults>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT wpm, accuracy, highest_combo, when_played_secs
            FROM passage_stats
            WHERE game_mode = ?1
            ORDER BY when_played_secs ASC",
    )?;

    let user_results_iter = stmt.query_map(params![game_mode as i64], |row| {
        Ok(RawUserResults {
            wpm: row.get(0)?,
            accuracy: row.get(1)?,
            highest_combo: row.get(2)?,
            when_played_secs: row.get(3)?,
        })
    })?;

    Ok(user_results_iter
        .map(|result| result.unwrap())
        .collect::<Vec<RawUserResults>>())
}
