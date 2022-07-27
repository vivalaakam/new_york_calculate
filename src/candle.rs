#[derive(Clone, Default, Debug)]
pub struct Candle {
    pub start_time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub quote: f64,
    pub trades: f64,
    pub buy_base: f64,
    pub buy_quote: f64,
    pub history: Vec<f64>,
    pub shape: Vec<usize>,
    pub max_profit_12: f64,
    pub max_profit_24: f64,
    pub score: f64,
}

impl Candle {
    pub fn get_data(&self) -> Vec<f64> {
        vec![
            self.open,
            self.high,
            self.low,
            self.close,
            self.volume,
            self.quote,
            self.trades,
            self.buy_base,
            self.buy_quote,
        ]
    }
}
