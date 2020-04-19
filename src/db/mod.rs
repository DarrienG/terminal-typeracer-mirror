use rusqlite::{params, Connection, Result};

use std::path::PathBuf;

use crate::dirs::setup_dirs;
use std::fs::read_dir;
use std::path::Path;

/// See if the stats db exists
pub fn check_stats_db() -> bool {
    let db_dir = setup_dirs::get_db_dir();
    Path::new(&db_dir).exists()
        && dbg!(read_dir(&db_dir))
            .unwrap()
            .filter(|item| item.is_ok() && item.as_ref().unwrap().file_name() == "stats.db")
            .count()
            > 0
}

/// Database does not exist, so let's make it
pub fn create_database(path: &PathBuf) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_info (
            schema_version INTEGER PRIMARY KEY
        )",
        params![],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS passages (
            passage TEXT PRIMARY KEY,
            passage_len INTEGER
        )",
        params![],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS passage_stats (
            row_id INTEGER PRIMARY KEY,
            passage TEXT,
            wpm INTEGER,
            accuracy REAL,
            highest_combo INTEGER,
            FOREIGN KEY(passage) REFERENCES passages(passage)
        )",
        params![],
    )?;

    Ok(())
}

pub fn db_path(base_path: &PathBuf) -> PathBuf {
    base_path.join("stats.db")
}
