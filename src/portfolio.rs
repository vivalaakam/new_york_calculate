use log::debug;

use crate::utils::{floor_to_nearest, round_to_nearest};
use crate::Candle;

pub struct Portfolio {
    candles: Vec<Candle>,
    initial_balance: f64,
    stake: f64,
    step_lot: f64,
    // step_price: f64,
    step_sum: f64,
    fee: f64,
}

impl Portfolio {
    pub fn new(candles: Vec<Candle>, initial_balance: Option<f64>, stake: Option<f64>) -> Self {
        Portfolio {
            candles,
            initial_balance: initial_balance.unwrap_or(3000f64),
            stake: stake.unwrap_or(10f64),
            fee: 0.001,
            step_lot: 1f64,
            // step_price: 0.0001,
            step_sum: 0.0000001,
        }
    }

    pub fn calculate(&self, results: Vec<u8>) -> (f64, f64, f64) {
        let mut base = 0f64;
        let mut quote = self.initial_balance;

        for (ind, candle) in self.candles.iter().enumerate() {
            if ind >= results.len() {
                continue;
            }

            match results[ind] {
                0 => {
                    let curr_stake = floor_to_nearest(self.stake / candle.open, self.step_lot);
                    if curr_stake >= base {
                        let order_sum = round_to_nearest(curr_stake * candle.open, self.step_sum);
                        quote += order_sum;
                        quote -= round_to_nearest(order_sum * self.fee, self.step_sum);
                        base -= curr_stake;

                        debug!(
                            "SELL {} by {} ({}, {})",
                            curr_stake, candle.open, quote, base
                        );
                    }
                }
                2 => {
                    if self.stake < quote {
                        let curr_stake = floor_to_nearest(self.stake / candle.open, self.step_lot);
                        let order_sum = round_to_nearest(curr_stake * candle.open, self.step_sum);
                        quote -= order_sum;
                        quote -= round_to_nearest(order_sum * self.fee, self.step_sum);
                        base += curr_stake;

                        debug!(
                            "BUY {} by {} ({}, {})",
                            curr_stake, candle.open, quote, base
                        );
                    }
                }
                _ => {}
            }
        }

        let last_candle = self.candles.last().unwrap();

        (quote, base, base * last_candle.close)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::BufReader;

    use env_logger;
    use serde_json::from_reader;

    use crate::candle::Candle;
    use crate::candle_response::CandleResponse;

    use super::*;

    fn init_logger() {
        let _ = env_logger::builder()
            // Include all events in tests
            .filter_level(log::LevelFilter::max())
            // Ensure events are captured by `cargo test`
            .is_test(true)
            // Ignore errors initializing the logger if tests race to configure it
            .try_init();
    }

    #[test]
    fn should_work() {
        init_logger();

        let file = fs::File::open("tests/candles.json").expect("file should open read only");

        let json = from_reader::<_, Vec<CandleResponse>>(BufReader::new(file)).unwrap();
        let candles = json
            .into_iter()
            .map(|candle| candle.into())
            .collect::<Vec<Candle>>();

        let portfolio = Portfolio::new(candles, Some(3000.0), None);

        let result = portfolio.calculate(vec![1, 2, 1, 0, 1, 2, 0, 2, 2, 1, 1, 1, 0, 2, 0]);

        assert_eq!(result.0, 2970.205544299999);
        assert_eq!(result.1, 92.0);
        assert_eq!(result.2, 30.176000000000002);
    }
}
