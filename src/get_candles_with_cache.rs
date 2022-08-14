use new_york_utils::{exists_file, read_from_file, write_to_file};

use crate::{get_candles, Candle, Indicators};

pub async fn get_candles_with_cache(
    ticker: String,
    period: usize,
    start_time: u64,
    look_back: usize,
    indicators: Option<Vec<Indicators>>,
) -> Vec<Candle> {
    let filename = format!("tmp/{ticker}_{start_time}_{period}_{look_back}.cbor");
    if exists_file(filename.as_str()) == false {
        let new_candles = get_candles(ticker, period, start_time, look_back, indicators).await;
        write_to_file(filename.as_str(), new_candles);
    }

    read_from_file(filename.as_str())
}
