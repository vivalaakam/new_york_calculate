use std::cmp::Ordering;

use crate::symbol::Symbol;

#[derive(Clone, Debug)]
pub struct Candle {
    pub start_time: u64,
    pub end_time: u64,
    pub symbol: Symbol,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub volume: f32,
    pub trades: f32,
}

impl Candle {
    pub fn get_data(&self) -> Vec<f32> {
        vec![
            self.open,
            self.high,
            self.low,
            self.close,
            self.volume,
            self.trades,
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
