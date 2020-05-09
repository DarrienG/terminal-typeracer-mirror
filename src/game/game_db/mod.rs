use rusqlite::types::ToSql;
use rusqlite::{params, Connection, Result};

use std::path::PathBuf;

use crate::dirs::setup_dirs::get_quote_dir;
use crate::passage_controller::PassageInfo;
use crate::stats::Stats;
use std::convert::TryFrom;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn store_stats(
    db_path: &PathBuf,
    game_stats: &Stats,
    passage_info: &PassageInfo,
    instant_death: bool,
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
            instant_death,
            when_played_secs
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            local_path,
            ToSql::to_sql(&i64::try_from(game_stats.get_wpm()).unwrap())?,
            game_stats.get_typing_accuracy(),
            ToSql::to_sql(&i64::try_from(game_stats.get_highest_combo()).unwrap())?,
            instant_death,
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
        .trim_start_matches(&get_quote_dir().to_string_lossy().into_owned())
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properly_trims_to_local_path() {
        let quote_dir = get_quote_dir();

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
