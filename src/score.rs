pub fn get_score(wallet: f64, drawdown: f64, successful_ratio: f64) -> f64 {
    wallet * drawdown * successful_ratio
}
