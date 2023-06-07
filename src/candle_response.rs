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

impl From<CandleResponse> for Candle {
    fn from(val: CandleResponse) -> Self {
        Candle {
            start_time: val.start_time / 1000,
            end_time: val.end_time / 1000,
            open: val.open.parse::<f64>().unwrap(),
            high: val.high.parse::<f64>().unwrap(),
            low: val.low.parse::<f64>().unwrap(),
            close: val.close.parse::<f64>().unwrap(),
            volume: val.volume.parse::<f64>().unwrap(),
            quote: val.quote.parse::<f64>().unwrap(),
            trades: val.trades as f64,
            buy_base: val.buy_base.parse::<f64>().unwrap(),
            buy_quote: val.buy_quote.parse::<f64>().unwrap(),
            interval: ((val.end_time as f64 - val.start_time as f64) / 60000f64).ceil() as u64,
            ..Candle::default()
        }
    }
}
