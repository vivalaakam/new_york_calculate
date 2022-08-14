use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
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
    pub max_profit: Vec<f64>,
    pub max_profit_1: f64,
    pub max_profit_3: f64,
    pub max_profit_6: f64,
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

impl Ord for Candle {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_time.partial_cmp(&other.start_time).unwrap()
    }
}

impl Eq for Candle {}

impl PartialEq for Candle {
    fn eq(&self, other: &Self) -> bool {
        self.start_time == other.start_time
    }
}

impl PartialOrd for Candle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
