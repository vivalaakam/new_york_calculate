#[derive(Debug)]
pub struct Order {
    pub start_time: u64,
    pub end_time: u64,
    pub buy_price: f64,
    pub sell_price: f64,
    pub qty: f64,
    pub commission: f64,
}
