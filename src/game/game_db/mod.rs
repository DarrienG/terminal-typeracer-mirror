use std::collections::HashSet;
use std::time::Duration;
use std::{
    convert::TryFrom,
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use rand::Rng;
use rusqlite::{params, types::ToSql, Connection, Result};

use crate::{
    db, dirs::setup_dirs::get_quote_dirs, game, passage_controller::PassageInfo, stats::Stats,
};

#[derive(Debug)]
pub enum DbRecreationError {
    // Right now we only care if we failed or we didn't.
    // We can get more fine grained later if we care.
    Failure,
    DbTooYoung,
    DbNotAFile,
}

pub fn store_stats(
    db_path: &Path,
    game_stats: &Stats,
    passage_info: &PassageInfo,
    game_mode: game::GameMode,
) -> Result<(), rusqlite::Error> {
    if !should_persist(passage_info) {
        return Ok(());
    }

    let local_path = local_passage_path(passage_info.passage_path.clone());

    let conn = Connection::open(db_path)?;
    conn.execute(
        "INSERT OR REPLACE INTO passages (passage, passage_len) VALUES (?1, ?2)",
        params![
            local_path,
            ToSql::to_sql(&i64::try_from(passage_info.passage.len()).unwrap())?
        ],
    )?;

    conn.execute(
        "INSERT INTO passage_stats (
            passage,
            wpm,
            accuracy,
            highest_combo,
            game_mode,
            when_played_secs
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            local_path,
            ToSql::to_sql(&i64::try_from(game_stats.get_wpm()).unwrap())?,
            game_stats.get_typing_accuracy(),
            ToSql::to_sql(&i64::try_from(game_stats.get_highest_combo()).unwrap())?,
            game_mode as i64,
            ToSql::to_sql(
                &i64::try_from(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_secs()
                )
                .expect("Failed play_time_secs conversion to sqlite type")
            )?
        ],
    )?;

    Ok(())
}

pub fn store_mistaken_words(
    db_path: &Path,
    mistaken_words: &HashSet<String>,
) -> Result<(), rusqlite::Error> {
    let mut conn = Connection::open(db_path)?;
    let tx = conn.transaction()?;

    for word in mistaken_words {
        tx.execute(
            "INSERT OR REPLACE INTO mistaken_words (word) VALUES (?1)",
            params![word,],
        )?;
    }

    tx.commit()
}

/// This function is intended to run at the end of a training mode game, to give a chance to
/// remove some words from the mistaken_words table in the database.
///
/// We do this by seeing if words in the typed passage also appear in the mistaken_words set
/// for that passage (words the user made an error on). If a word was typed incorrectly, it doesn't
/// get a chance to be removed from the mistaken_words table. Otherwise, we 'roll a dice' to
/// determine if a correctly typed word should be removed.
pub fn roll_to_delete_mistaken_words_typed_correctly(
    db_path: &Path,
    words: &[&str],
    mistaken_words: &HashSet<String>,
) -> Result<(), rusqlite::Error> {
    let mut conn = Connection::open(db_path)?;
    let mut rng = rand::thread_rng();

    let tx = conn.transaction()?;

    for word in words {
        match mistaken_words.get(&word.to_string()) {
            Some(_a) => (),
            _ => {
                let random_val: f32 = rng.gen();
                if random_val < 0.33 {
                    // 1/3 chance of removing the word from the db
                    tx.execute(
                        "DELETE FROM mistaken_words where word = (?1)",
                        params![word,],
                    )?;
                }
            }
        }
    }

    tx.commit()
}

/// Determines if we should persist data for the passage or not
/// user given passages vary in length and content, so we do not want to persist
/// any data about them.
fn should_persist(passage_info: &PassageInfo) -> bool {
    passage_info.passage_path != "User input" && passage_info.passage_path != "FALLBACK_PATH"
}

/// Trims out all parts of passage path outside of typeracer data dir
/// e.g. /home/darrien/.local/share/typeracer/lang_packs/default/1.txt
/// becomes -> /default/1.txt
fn local_passage_path(absolute_passage: String) -> String {
    absolute_passage
        .trim_start_matches(
            &get_quote_dirs()
                .main_pack_dir
                .to_string_lossy()
                .into_owned(),
        )
        .to_owned()
}

/// Delete and rebuild stats database
/// This is not a common error, but has happened more than a few
/// times over the last few years. It appears on very old migrations a
/// migration may not have run, causing us to be left in a corrupted DB state.
/// "New" installations (within the last year) do not appear to have this problem.
/// When we run into this error, we are best off deleting and rebuilding the database.
/// To make sure this does not run too readily, we want to make sure we only
/// run on old database installations (e.g. created prior to January 2023).
/// For more info, see here: https://gitlab.com/ttyperacer/terminal-typeracer/-/issues/39
pub fn rebuild_stats_db_if_ancient(db_path: &Path) -> Result<(), DbRecreationError> {
    // Somehow we got a bad db path. This should never happen, but let's
    // make sure we do not delete directories.
    if db_path.is_dir() {
        return Err(DbRecreationError::DbNotAFile);
    }
    let jan_2023 = SystemTime::UNIX_EPOCH + Duration::from_secs(1672444800);
    match fs::metadata(db_path) {
        Ok(metadata) => {
            if metadata.created().unwrap() < jan_2023 {
                rebuild_stats_db(&db_path);
                return Ok(());
            }
            Err(DbRecreationError::DbTooYoung)
        }
        Err(_) => Err(DbRecreationError::Failure),
    }
}

fn rebuild_stats_db(db_path: &Path) {
    match fs::remove_file(db_path) {
        Ok(_) => {}
        Err(e) => eprintln!("Failed to delete database during recreation, something bad is happening. Show this to a maintainer: {}", e)
    }
    match db::create_database(db_path) {
        Ok(_) => {}
        Err(e) => eprintln!("Failed to recreate database, something bad is happening. Show this to a maintainer: {}", e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properly_trims_to_local_path() {
        let quote_dir = get_quote_dirs().main_pack_dir;

        assert_eq!(
            local_passage_path(
                quote_dir
                    .join("/default/b7448c1c-c70b-4183-86f9-94049376926e")
                    .to_string_lossy()
                    .into_owned()
            ),
            "/default/b7448c1c-c70b-4183-86f9-94049376926e"
        );

        assert_eq!(
            local_passage_path(
                quote_dir
                    .join("/default/itsnotover/broooooooo/extrapaths")
                    .to_string_lossy()
                    .into_owned()
            ),
            "/default/itsnotover/broooooooo/extrapaths"
        );
    }
}
