use reqwest;
use reqwest::Client;

use crate::candle_response::CandleResponse;
use crate::get_interval_key::get_interval_key;
use crate::utils::range;
use crate::Candle;

pub async fn get_candles(
    ticker: String,
    period: usize,
    start_time: u64,
    look_back: usize,
) -> Vec<Candle> {
    let mut from_time = (start_time - 86400).min(start_time - (look_back * period * 2) as u64);
    let to_time = start_time + 86400 * 2;
    let end_time = start_time + 86400;

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

    let in_day = candles_cache
        .clone()
        .into_iter()
        .filter(|k| k.start_time >= start_time && k.start_time < end_time)
        .collect::<Vec<_>>();

    let variants_min = [1, 4, 5, 6, 7, 8];
    let variants_max = [2, 4, 5, 6, 7, 8];
    let hist_map = [
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 1),
        (5, 2),
        (6, 3),
        (7, 4),
        (8, 5),
    ];

    let mut candles = vec![];

    for row in &in_day {
        let prev_time = row.start_time - 86400;
        let next_12_time = row.start_time + 43200;
        let next_24_time = row.start_time + 86400;

        let prev = candles_cache
            .iter()
            .filter(|k| k.start_time >= prev_time && k.start_time < row.start_time)
            .collect::<Vec<_>>();

        let mut deltas_min = [f64::MAX; 6];
        let mut deltas_max = [f64::MIN; 6];
        let mut prev_size = 0usize;
        for candle in &prev {
            let hist_data = candle.get_data();
            for i in 0..6 {
                if hist_data[variants_max[i]] > deltas_max[i] {
                    deltas_max[i] = hist_data[variants_max[i]]
                }

                if hist_data[variants_min[i]] < deltas_min[i] {
                    deltas_min[i] = hist_data[variants_min[i]]
                }
            }

            prev_size += 1;
        }

        let mut hist = vec![];

        for p in prev_size - look_back..prev_size {
            let candle = &prev[p];
            let hist_data = candle.get_data();
            for h in &hist_map {
                hist.push(range(
                    deltas_min[h.1],
                    deltas_max[h.1],
                    0f64,
                    1f64,
                    hist_data[h.0],
                ));
            }

            for i in 0..6 {
                hist.push(deltas_max[i] / deltas_min[i] - 1f64);
            }
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
        candle.history = hist;
        candle.shape = vec![look_back as i32, 15];
        candle.max_profit_12 = (max_12 - 1f64) * 100f64;
        candle.max_profit_24 = (max_24 - 1f64) * 100f64;

        candles.push(candle.clone());
    }

    candles
}

#[cfg(test)]
mod tests {
    use tokio;

    use super::*;

    #[tokio::test]
    async fn test_get_candles() {
        let candles = get_candles("XRPUSDT".to_string(), 5, 1655769600, 12).await;

        assert_eq!(288, candles.len());
        assert_eq!(
            format!("{:?}", candles[0]),
            r#"Candle { start_time: 1655769600, open: 0.3226, high: 0.3232, low: 0.3217, close: 0.3223, volume: 1047813.0, quote: 337795.6648, trades: 491.0, buy_base: 502532.0, buy_quote: 161987.0611, history: [0.6287878787878773, 0.674242424242422, 0.6136363636363639, 0.6590909090909085, 0.054492106699697195, 0.055632664907512216, 0.04469727752945957, 0.08183721925376695, 0.08369696897424625, 0.04199809099586371, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.6590909090909085, 0.6590909090909085, 0.6060606060606029, 0.6060606060606029, 0.0, 0.0, 0.0004063388866314506, 0.008871964016942867, 0.009196089159155357, 0.04199809099586371, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.6060606060606029, 0.6060606060606029, 0.47727272727272557, 0.537878787878788, 0.05181271128562022, 0.05246007760850539, 0.07110930516050386, 0.026963115385634916, 0.02735228421705913, 0.04199809099586371, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.530303030303027, 0.537878787878788, 0.47727272727272557, 0.49242424242423904, 0.034678686742829444, 0.03502789440075473, 0.04185290532303942, 0.02719615118080694, 0.02757719741910689, 0.04199809099586371, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.49242424242423904, 0.5454545454545446, 0.4848484848484823, 0.5227272727272703, 0.01898510688818661, 0.01911238779803759, 0.017878911011783828, 0.0, 0.0, 0.04199809099586371, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.530303030303027, 0.5530303030303014, 0.4166666666666632, 0.5530303030303014, 0.13934141308312384, 0.14098848465475683, 0.12393336042259244, 0.11603842643744489, 0.11750453806131823, 0.04199809099586371, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.5454545454545446, 0.7272727272727276, 0.530303030303027, 0.7121212121212099, 0.11448896112956566, 0.11676950124621537, 0.20438845997561966, 0.13854764518785453, 0.14151883111689875, 0.04199809099586371, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.7121212121212099, 0.8030303030303035, 0.6666666666666653, 0.7878787878787858, 0.13465166788654406, 0.13792943426377186, 0.24258431531897603, 0.1257962177707854, 0.12912421408830838, 0.04199809099586371, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.780303030303029, 0.818181818181817, 0.7348484848484844, 0.7954545454545426, 0.13186073936809536, 0.1354766775721419, 0.20682649329540836, 0.09435124145450456, 0.09716936633968401, 0.04199809099586371, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.7954545454545426, 0.8409090909090915, 0.7348484848484844, 0.7499999999999979, 0.11846192636183603, 0.12169007821738478, 0.18407151564404714, 0.09201971832380848, 0.09475871054320441, 0.04199809099586371, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.7499999999999979, 0.7499999999999979, 0.7121212121212099, 0.7348484848484844, 0.05732189586082137, 0.05881871122553912, 0.07557903291344982, 0.054404245446116446, 0.05599912101930392, 0.04199809099586371, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616, 0.7499999999999979, 0.7499999999999979, 0.6136363636363639, 0.636363636363634, 0.0520082384562472, 0.05321408655826363, 0.10524177163754571, 0.025277101407565332, 0.026047438015938652, 0.04199809099586371, 33.392132420557886, 32.76109809815581, 14.648809523809524, 31.125155265424546, 30.680000727121616], shape: [12, 15], max_profit_12: 2.9758214507129566, max_profit_24: 4.587724736515808 }"#
        );

        let success = candles
            .into_iter()
            .filter(|x| x.max_profit_12 > 1f64)
            .collect::<Vec<_>>();
        assert_eq!(208, success.len());
    }
}
