pub fn num_to_day(day_num: f64) -> String {
    if day_num == 0.0 {
        "last play".to_owned()
    } else {
        format!("{:.2}", day_num)
    }
}
