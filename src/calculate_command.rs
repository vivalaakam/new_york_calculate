pub enum CalculateCommand {
    Unknown,
    /* score */
    None(f64),
    /* gain, stake, score */
    BuyProfit(f64, f64, f64),
}
