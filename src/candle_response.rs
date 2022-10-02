use crate::Candle;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CandleResponse {
    start_time: u64,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: String,
    end_time: u64,
    quote: String,
    trades: u64,
    buy_base: String,
    buy_quote: String,
    _ign: String,
}

impl Into<Candle> for CandleResponse {
    fn into(self) -> Candle {
        Candle {
            start_time: self.start_time / 1000,
            end_time: self.end_time / 1000,
            open: self.open.parse::<f64>().unwrap(),
            high: self.high.parse::<f64>().unwrap(),
            low: self.low.parse::<f64>().unwrap(),
            close: self.close.parse::<f64>().unwrap(),
            volume: self.volume.parse::<f64>().unwrap(),
            quote: self.quote.parse::<f64>().unwrap(),
            trades: self.trades as f64,
            buy_base: self.buy_base.parse::<f64>().unwrap(),
            buy_quote: self.buy_quote.parse::<f64>().unwrap(),
            interval: ((self.end_time as f64 - self.start_time as f64) / 60000f64).ceil() as u64,
            ..Candle::default()
        }
    }
}
