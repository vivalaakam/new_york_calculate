#[derive(Clone)]
pub enum CalculateCommand {
    Unknown,
    None,
    /* profit, stake */
    BuyProfit(f64, f64),
}
