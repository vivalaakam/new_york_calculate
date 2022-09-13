use std::time::{SystemTime, UNIX_EPOCH};

use log::LevelFilter;

use new_york_calculate_core::{
    get_candles_with_cache, CalculateCommand, CalculateIter, CalculateResult,
};

#[tokio::main]
async fn main() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .is_test(true)
        .try_init();

    let mut candles = vec![];

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    let next = (now / 86400000) as u64;
    let mut keys = vec![];

    let mut from = (next - 95) * 86400;
    let to = from + 92 * 86400;

    while from <= to {
        keys.push(from);
        from += 86400;
    }

    keys.push(from);

    for key in keys {
        let new_candles = get_candles_with_cache("XRPBUSD".to_string(), 15, key, 12, None).await;
        candles = [candles, new_candles].concat();
    }

    candles.sort();

    let targets = [3.0, 1.25, 1.0, 0.75, 0.5];
    let gain = [1.03, 1.0125, 1.01, 1.0075, 1.005];
    let stake = [500.0, 200.0, 100.0, 50.0, 25.0];
    let target = candles.len() - 288;
    for i1 in 0..5 {
        for i2 in i1..5 {
            let mut calculate_iter = CalculateIter::new(
                &candles,
                3000f64,
                0.5,
                15,
                1f64,
                0.0001f64,
                Box::new(move |candle, ind, stats| {
                    if ind >= target {
                        return CalculateCommand::Unknown;
                    }

                    println!("{:?}", stats);

                    for j in i1..i2 + 1 {
                        if candle.max_profit[4 - j] > targets[j] {
                            return CalculateCommand::BuyProfit(gain[j], stake[j], 1f64);
                        }
                    }

                    CalculateCommand::None(0f64)
                }),
            );

            let mut cont = Ok(());

            while cont.is_ok() {
                cont = calculate_iter.next();
            }

            let result: CalculateResult = calculate_iter.into();
            println!("{i1} - {i2}: {:?}", result);
        }
    }
}
