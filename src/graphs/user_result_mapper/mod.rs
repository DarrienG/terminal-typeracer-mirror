use crate::graphs::{RawUserResults, UserResults};

/// Takes ordered list of raw user results and maps to a list of
/// UserResults ready for consumption by the graph render.
pub fn as_user_results(ordered_user_results: &[RawUserResults]) -> Vec<UserResults> {
    let days_played_for = &days_played_for(ordered_user_results);

    let first_played_time = match (*ordered_user_results).first() {
        Some(v) => v.when_played_secs,
        None => 0,
    };

    let mut mapped_results = (*ordered_user_results)
        .iter()
        .map(|raw_user_result| UserResults {
            wpm: raw_user_result.wpm,
            accuracy: raw_user_result.accuracy,
            highest_combo: raw_user_result.highest_combo,
            days_back_played: days_played_back(
                *days_played_for,
                first_played_time,
                raw_user_result.when_played_secs,
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

fn days_played_back(days_played_for: f64, first_played_time: i64, when_played_secs: i64) -> f64 {
    let normalized_play_time = ((when_played_secs - first_played_time) as f64) / 86400.0;
    let played_percent_days = normalized_play_time / days_played_for;
    let inverted_percent = 1.0 - played_percent_days;

    inverted_percent * days_played_for
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::izip;

    #[test]
    fn days_back_set_correctly() {
        let raw_user_results = vec![
            RawUserResults {
                wpm: 80,
                accuracy: 100.0,
                highest_combo: 100,
                when_played_secs: 1588464000,
            },
            RawUserResults {
                wpm: 80,
                accuracy: 100.0,
                highest_combo: 100,
                when_played_secs: 1588550400,
            },
            RawUserResults {
                wpm: 80,
                accuracy: 100.0,
                highest_combo: 100,
                when_played_secs: 1588636800,
            },
        ];

        let user_results = as_user_results(&raw_user_results);
        let expected_days = vec![2.0, 1.0, 0.0];

        for (result, expected) in izip!(user_results.iter(), expected_days.iter()) {
            assert_eq_float(result.days_back_played, *expected);
        }
    }

    fn assert_eq_float(v1: f64, v2: f64) {
        let error_margin = f64::EPSILON;
        assert!((v1 - v2).abs() < error_margin);
    }
}
