use new_york_utils::Matrix;
use reqwest;
use reqwest::Client;

use crate::candle_response::CandleResponse;
use crate::get_interval_key::get_interval_key;
use crate::{Candle, Indicators};

pub async fn get_candles(
    ticker: String,
    period: usize,
    start_time: u64,
    look_back: usize,
    indicators: Option<Vec<Indicators>>,
) -> Vec<Candle> {
    let mut from_time = (start_time - 86400).min(start_time - (look_back * period * 2) as u64);
    let to_time = start_time + 86400 * 2;

    let mut candles_cache = vec![];

    while from_time < to_time {
        let body = Client::new()
            .get(format!(
                "https://api.binance.com/api/v3/klines?symbol={}&interval={}&startTime={}&limit=1000",
                ticker, get_interval_key(period), from_time * 1000
            ))
            .send()
            .await
            .unwrap()
            .json::<Vec<CandleResponse>>()
            .await
            .unwrap()
            .into_iter()
            .map(|x| x.into()).collect::<Vec<Candle>>();

        for row in body {
            from_time = from_time.max(row.start_time);

            if from_time < to_time {
                candles_cache.push(row);
            }
        }
    }

    perform_candles(candles_cache, start_time, look_back, indicators)
}

pub fn perform_candles(
    candles_cache: Vec<Candle>,
    start_time: u64,
    look_back: usize,
    indicators: Option<Vec<Indicators>>,
) -> Vec<Candle> {
    let end_time = start_time + 86400;

    let in_day = candles_cache
        .clone()
        .into_iter()
        .filter(|k| k.start_time >= start_time && k.start_time < end_time)
        .collect::<Vec<_>>();

    let indicators = match indicators {
        None => vec![
            Indicators::Open24,
            Indicators::High24,
            Indicators::Low24,
            Indicators::Close24,
            Indicators::Volume24,
            Indicators::QuoteAsset24,
            Indicators::Trades24,
            Indicators::BuyBase24,
            Indicators::BuyQuote24,
            Indicators::Candle24Delta,
            Indicators::Volume24Delta,
            Indicators::QuoteAsset24Delta,
            Indicators::Trades24Delta,
            Indicators::BuyBase24Delta,
            Indicators::BuyQuote24Delta,
        ],
        Some(v) => v,
    };

    let mut deltas = (false, false, false, false, false, false);

    for indicator in &indicators {
        if deltas.0 == false && Indicators::candle_24().contains(&indicator) {
            deltas.0 = true;
        }

        if deltas.1 == false && Indicators::volume_24().contains(&indicator) {
            deltas.1 = true;
        }

        if deltas.2 == false && Indicators::quote_asset_24().contains(&indicator) {
            deltas.2 = true;
        }

        if deltas.3 == false && Indicators::trades_24().contains(&indicator) {
            deltas.3 = true;
        }

        if deltas.4 == false && Indicators::buy_base_24().contains(&indicator) {
            deltas.4 = true;
        }

        if deltas.5 == false && Indicators::buy_quote_24().contains(&indicator) {
            deltas.5 = true;
        }
    }

    let mut candles = vec![];

    for row in &in_day {
        let prev_time = row.start_time - 86400;
        let next_12_time = row.start_time + 43200;
        let next_24_time = row.start_time + 86400;

        let prev = candles_cache
            .iter()
            .filter(|k| k.start_time >= prev_time && k.start_time < row.start_time)
            .collect::<Vec<_>>();

        let mut deltas_data = vec![(f64::MAX, f64::MIN); 6];

        for candle in &prev {
            if deltas.0 == true {
                deltas_data[0] = (
                    deltas_data[0].0.min(candle.low),
                    deltas_data[0].1.max(candle.high),
                )
            }

            if deltas.1 == true {
                deltas_data[1] = (
                    deltas_data[1].0.min(candle.volume),
                    deltas_data[1].1.max(candle.volume),
                )
            }

            if deltas.2 == true {
                deltas_data[2] = (
                    deltas_data[2].0.min(candle.quote),
                    deltas_data[2].1.max(candle.quote),
                )
            }

            if deltas.3 == true {
                deltas_data[3] = (
                    deltas_data[3].0.min(candle.trades),
                    deltas_data[3].1.max(candle.trades),
                )
            }

            if deltas.4 == true {
                deltas_data[4] = (
                    deltas_data[4].0.min(candle.buy_base),
                    deltas_data[4].1.max(candle.buy_base),
                )
            }

            if deltas.5 == true {
                deltas_data[5] = (
                    deltas_data[5].0.min(candle.buy_quote),
                    deltas_data[5].1.max(candle.buy_quote),
                )
            }
        }

        let mut hist = Matrix::new(indicators.len(), look_back);

        for i in 0..indicators.len() {
            println!("{:?}", indicators[i]);
            let data = indicators[i].get_data(&prev, look_back, &deltas_data);
            let _ = hist.add_column(i, data);
        }

        let mut max_12 = 0f64;

        let next_12 = candles_cache
            .iter()
            .filter(|k| k.start_time >= row.start_time && k.start_time < next_12_time)
            .collect::<Vec<_>>();

        for candle in next_12 {
            max_12 = max_12.max(candle.high / row.open)
        }

        let mut max_24 = max_12;

        let next_24 = candles_cache
            .iter()
            .filter(|k| k.start_time >= next_12_time && k.start_time < next_24_time)
            .collect::<Vec<_>>();

        for candle in next_24 {
            max_24 = max_24.max(candle.high / row.open)
        }

        let mut candle = row.clone();
        candle.history = hist.get_data().unwrap();
        candle.shape = vec![hist.get_rows(), hist.get_columns()];
        candle.max_profit_12 = (max_12 - 1f64) * 100f64;
        candle.max_profit_24 = (max_24 - 1f64) * 100f64;

        candles.push(candle.clone());
    }

    candles
}

#[cfg(test)]
mod tests {
    use serde_json::from_reader;
    use std::fs::File;
    use tokio;

    use super::*;

    fn init_logger() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::max())
            .is_test(true)
            .try_init();
    }

    #[tokio::test]
    async fn test_get_candles() {
        init_logger();

        let candles = get_candles("XRPUSDT".to_string(), 5, 1655769600, 12, None).await;

        assert_eq!(288, candles.len());
        assert_eq!(
            format!("{:?}", candles[0]),
            r#"Candle { start_time: 1655769600, open: 0.3226, high: 0.3232, low: 0.3217, close: 0.3223, volume: 1047813.0, quote: 337795.6648, trades: 491.0, buy_base: 502532.0, buy_quote: 161987.0611, history: [0.5909090909090897, 0.629870129870128, 0.5779220779220783, 0.6168831168831165, 0.054492106699697195, 0.055632664907512216, 0.04469727752945957, 0.08183721925376695, 0.08369696897424625, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.6168831168831165, 0.6168831168831165, 0.5714285714285688, 0.5714285714285688, 0.0, 0.0, 0.0004063388866314506, 0.008871964016942867, 0.009196089159155357, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.5714285714285688, 0.5714285714285688, 0.46103896103895997, 0.5129870129870133, 0.05181271128562022, 0.05246007760850539, 0.07110930516050386, 0.026963115385634916, 0.02735228421705913, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.506493506493504, 0.5129870129870133, 0.46103896103895997, 0.4740259740259715, 0.034678686742829444, 0.03502789440075473, 0.04185290532303942, 0.02719615118080694, 0.02757719741910689, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.4740259740259715, 0.5194805194805191, 0.46753246753246575, 0.4999999999999982, 0.01898510688818661, 0.01911238779803759, 0.017878911011783828, 0.0, 0.0, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.506493506493504, 0.5259740259740249, 0.4090909090909066, 0.5259740259740249, 0.13934141308312384, 0.14098848465475683, 0.12393336042259244, 0.11603842643744489, 0.11750453806131823, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.5194805194805191, 0.6753246753246755, 0.506493506493504, 0.6623376623376604, 0.11448896112956566, 0.11676950124621537, 0.20438845997561966, 0.13854764518785453, 0.14151883111689875, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.6623376623376604, 0.7402597402597404, 0.6233766233766223, 0.7272727272727253, 0.13465166788654406, 0.13792943426377186, 0.24258431531897603, 0.1257962177707854, 0.12912421408830838, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.7207792207792195, 0.7532467532467519, 0.6818181818181813, 0.7337662337662311, 0.13186073936809536, 0.1354766775721419, 0.20682649329540836, 0.09435124145450456, 0.09716936633968401, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.7337662337662311, 0.7727272727272729, 0.6818181818181813, 0.6948051948051929, 0.11846192636183603, 0.12169007821738478, 0.18407151564404714, 0.09201971832380848, 0.09475871054320441, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.6948051948051929, 0.6948051948051929, 0.6623376623376604, 0.6818181818181813, 0.05732189586082137, 0.05881871122553912, 0.07557903291344982, 0.054404245446116446, 0.05599912101930392, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.6948051948051929, 0.6948051948051929, 0.5779220779220783, 0.5974025974025955, 0.0520082384562472, 0.05321408655826363, 0.10524177163754571, 0.025277101407565332, 0.026047438015938652, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616], shape: [12, 15], max_profit_12: 2.9758214507129566, max_profit_24: 4.587724736515808, score: 0.0 }"#
        );

        let success = candles
            .into_iter()
            .filter(|x| x.max_profit_12 > 1f64)
            .collect::<Vec<_>>();
        assert_eq!(208, success.len());
    }

    #[tokio::test]
    async fn test_perform_candles() {
        let file = File::open("tests/candles.json").expect("file should open read only");
        let json = from_reader::<_, Vec<CandleResponse>>(file);

        assert_eq!(json.is_ok(), true);

        let candles = json
            .unwrap()
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<Candle>>();

        let candles = perform_candles(candles, 1655769600, 12, None);

        assert_eq!(288, candles.len());
        assert_eq!(
            format!("{:?}", candles[0]),
            r#"Candle { start_time: 1655769600, open: 0.3226, high: 0.3232, low: 0.3217, close: 0.3223, volume: 1047813.0, quote: 337795.6648, trades: 491.0, buy_base: 502532.0, buy_quote: 161987.0611, history: [0.5909090909090897, 0.629870129870128, 0.5779220779220783, 0.6168831168831165, 0.054492106699697195, 0.055632664907512216, 0.04469727752945957, 0.08183721925376695, 0.08369696897424625, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.6168831168831165, 0.6168831168831165, 0.5714285714285688, 0.5714285714285688, 0.0, 0.0, 0.0004063388866314506, 0.008871964016942867, 0.009196089159155357, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.5714285714285688, 0.5714285714285688, 0.46103896103895997, 0.5129870129870133, 0.05181271128562022, 0.05246007760850539, 0.07110930516050386, 0.026963115385634916, 0.02735228421705913, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.506493506493504, 0.5129870129870133, 0.46103896103895997, 0.4740259740259715, 0.034678686742829444, 0.03502789440075473, 0.04185290532303942, 0.02719615118080694, 0.02757719741910689, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.4740259740259715, 0.5194805194805191, 0.46753246753246575, 0.4999999999999982, 0.01898510688818661, 0.01911238779803759, 0.017878911011783828, 0.0, 0.0, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.506493506493504, 0.5259740259740249, 0.4090909090909066, 0.5259740259740249, 0.13934141308312384, 0.14098848465475683, 0.12393336042259244, 0.11603842643744489, 0.11750453806131823, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.5194805194805191, 0.6753246753246755, 0.506493506493504, 0.6623376623376604, 0.11448896112956566, 0.11676950124621537, 0.20438845997561966, 0.13854764518785453, 0.14151883111689875, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.6623376623376604, 0.7402597402597404, 0.6233766233766223, 0.7272727272727253, 0.13465166788654406, 0.13792943426377186, 0.24258431531897603, 0.1257962177707854, 0.12912421408830838, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.7207792207792195, 0.7532467532467519, 0.6818181818181813, 0.7337662337662311, 0.13186073936809536, 0.1354766775721419, 0.20682649329540836, 0.09435124145450456, 0.09716936633968401, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.7337662337662311, 0.7727272727272729, 0.6818181818181813, 0.6948051948051929, 0.11846192636183603, 0.12169007821738478, 0.18407151564404714, 0.09201971832380848, 0.09475871054320441, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.6948051948051929, 0.6948051948051929, 0.6623376623376604, 0.6818181818181813, 0.05732189586082137, 0.05881871122553912, 0.07557903291344982, 0.054404245446116446, 0.05599912101930392, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.6948051948051929, 0.6948051948051929, 0.5779220779220783, 0.5974025974025955, 0.0520082384562472, 0.05321408655826363, 0.10524177163754571, 0.025277101407565332, 0.026047438015938652, 0.04912280701754401, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616], shape: [12, 15], max_profit_12: 2.9758214507129566, max_profit_24: 4.587724736515808, score: 0.0 }"#
        );

        let success = candles
            .into_iter()
            .filter(|x| x.max_profit_12 > 1f64)
            .collect::<Vec<_>>();
        assert_eq!(208, success.len());
    }

    #[tokio::test]
    async fn test_perform_candles_without_indicators() {
        let file = File::open("tests/candles.json").expect("file should open read only");
        let json = from_reader::<_, Vec<CandleResponse>>(file);

        assert_eq!(json.is_ok(), true);

        let candles = json
            .unwrap()
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<Candle>>();

        let candles = perform_candles(candles, 1655769600, 12, Some(vec![]));

        assert_eq!(288, candles.len());
        assert_eq!(
            format!("{:?}", candles[0]),
            r#"Candle { start_time: 1655769600, open: 0.3226, high: 0.3232, low: 0.3217, close: 0.3223, volume: 1047813.0, quote: 337795.6648, trades: 491.0, buy_base: 502532.0, buy_quote: 161987.0611, history: [], shape: [12, 0], max_profit_12: 2.9758214507129566, max_profit_24: 4.587724736515808, score: 0.0 }"#
        );

        let success = candles
            .into_iter()
            .filter(|x| x.max_profit_12 > 1f64)
            .collect::<Vec<_>>();
        assert_eq!(208, success.len());
    }
}
