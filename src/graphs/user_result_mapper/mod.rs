use crate::graphs::{RawUserResults, UserResults};

/// Takes ordered list of raw user results and maps to a list of
/// UserResults ready for consumption by the graph render.
pub fn as_user_results(ordered_user_results: &[RawUserResults]) -> Vec<UserResults> {
    let latest_play_time = match ordered_user_results.last() {
        Some(v) => v.when_played_secs,
        None => 0,
    };

    let days_played_for = &days_played_for(ordered_user_results);

    let mut mapped_results = (*ordered_user_results)
        .iter()
        .map(|raw_user_result| UserResults {
            wpm: raw_user_result.wpm,
            accuracy: raw_user_result.accuracy,
            highest_combo: raw_user_result.highest_combo,
            days_back_played: days_back_played(
                *days_played_for,
                percentage_back_played(latest_play_time, raw_user_result.when_played_secs),
            ),
        })
        .collect::<Vec<UserResults>>();

    let first_result = match mapped_results.first() {
        Some(v) => UserResults {
            days_back_played: *days_played_for,
            ..*v
        },
        None => UserResults::default(),
    };

    let last_result = match mapped_results.last() {
        Some(v) => UserResults {
            days_back_played: 0.0,
            ..*v
        },
        None => UserResults::default(),
    };

    if mapped_results.len() < 2 {
        mapped_results.push(first_result);
        mapped_results.push(last_result)
    } else {
        let last_idx = mapped_results.len() - 1;

        mapped_results[0] = first_result;
        mapped_results[last_idx] = last_result;
    }

    mapped_results
}

fn days_played_for(ordered_user_results: &[RawUserResults]) -> f64 {
    let first_day = match ordered_user_results.first() {
        Some(v) => v.when_played_secs,
        None => 0,
    };

    let last_day = match ordered_user_results.last() {
        Some(v) => v.when_played_secs,
        None => 0,
    };

    (last_day - first_day) as f64 / 86400.0
}

fn percentage_back_played(latest_play_time: i64, when_played: i64) -> f64 {
    (latest_play_time as f64 - when_played as f64) / latest_play_time as f64
}

fn days_back_played(max_days_back: f64, percentage_days_back_played: f64) -> f64 {
    max_days_back * percentage_days_back_played
}

/*
 * (LATEST_PLAY_TIME - WHEN_PLAYED) / LATEST_PLAY_TIME => PERCENTAGE_DAYS_BACK_PLAYED
 *
 * MAX_DAYS_PLAYED_BACK * PERCENTAGE_DAYS_BACK_PLAYED = DAYS_BACK_PLAYED
 */
