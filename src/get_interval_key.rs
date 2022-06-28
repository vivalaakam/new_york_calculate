pub fn get_interval_key(period: usize) -> &'static str {
    match period {
        5 => "5m",
        _ => "1d",
    }
}
