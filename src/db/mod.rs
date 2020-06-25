use rusqlite::{Connection, Result, NO_PARAMS};

use std::path::PathBuf;

use crate::dirs::setup_dirs;
use std::fs::read_dir;
use std::path::Path;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("src/db/migrations");
}

static DB_VERSION: i64 = 2;

/// See if the stats db exists
pub fn check_stats_db() -> bool {
    let db_dir = setup_dirs::get_db_dir();
    Path::new(&db_dir).exists()
        && read_dir(&db_dir)
            .unwrap()
            .filter(|item| item.is_ok() && item.as_ref().unwrap().file_name() == "stats.db")
            .count()
            > 0
}

pub fn check_for_migration(path: &PathBuf) -> bool {
    let conn = Connection::open(path).expect("Unreachable DB");
    let db_version: Result<i64, rusqlite::Error> =
        conn.query_row("PRAGMA user_version", NO_PARAMS, |row| row.get(0));
    if let Ok(version) = db_version {
        version == DB_VERSION
    } else {
        false
    }
}

pub fn do_migration(path: &PathBuf) -> Result<(), rusqlite::Error> {
    // If our user_version is 0 then we have to do work to migrate to migration
    // package
    let mut conn = Connection::open(path)?;
    let db_version: i64 = conn.query_row("PRAGMA user_version", NO_PARAMS, |row| row.get(0))?;
    // We also have to check if the original table exists before commiting
    // to migrating to the migration package. Work around since sqlite is
    // garbage.
    let mut stmt = conn.prepare(
        "SELECT
            sql
        FROM
            sqlite_master
        WHERE
            name = 'passage_stats'
        AND
            type = 'table'",
    )?;
    let table_exists = stmt.exists(NO_PARAMS)?;
    drop(stmt);

    // If both conditions are true, then we have to deal with migrating
    // old data to new migration system
    if db_version == 0 && table_exists {
        conn.execute(
            "ALTER TABLE passage_stats RENAME TO ancient_passage_stats",
            NO_PARAMS,
        )?;
    }

    if embedded::migrations::runner().run(&mut conn).is_err() {
        return Err(rusqlite::Error::InvalidPath(path.to_path_buf()));
    }

    // UPDATE THIS MIGRATION CODE AS NEEDED. This should migrate data from
    // ancient_passage_stats to the most current schema
    if db_version == 0 && table_exists {
        conn.execute(
            "INSERT INTO passage_stats (
                row_id,
                passage,
                wpm,
                accuracy,
                highest_combo,
                game_mode,
                when_played_secs
            ) SELECT
                row_id,
                passage,
                wpm,
                accuracy,
                highest_combo,
                instant_death,
                when_played_secs
            FROM ancient_passage_stats",
            NO_PARAMS,
        )?;
        conn.execute("DROP TABLE ancient_passage_stats", NO_PARAMS)?;
    }
    Ok(())
}

/// Database does not exist, so let's make it
pub fn create_database(path: &PathBuf) -> Result<(), rusqlite::Error> {
    println!("CREATING DB");
    let mut conn = Connection::open(path)?;
    if embedded::migrations::runner().run(&mut conn).is_err() {
        return Err(rusqlite::Error::InvalidPath(path.to_path_buf()));
    }
    Ok(())
}

pub fn db_path(base_path: &PathBuf) -> PathBuf {
    base_path.join("stats.db")
}