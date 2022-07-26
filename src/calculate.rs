use crate::candle::Candle;
use crate::order::Order;
use crate::score::get_score;
use crate::utils::{ceil_to_nearest, floor_to_nearest};

pub struct Calculate {
    candles: Vec<Candle>,
    initial_balance: f64,
    stake: f64,
    gain: f64,
    profit: f64,
    step_lot: f64,
    step_price: f64,
    interval: u64,
}

impl Calculate {
    pub fn new(
        candles: Vec<Candle>,
        initial_balance: Option<f64>,
        stake: Option<f64>,
        gain: Option<f64>,
        profit: Option<f64>,
        interval: Option<u64>,
    ) -> Self {
        Calculate {
            candles,
            initial_balance: initial_balance.unwrap_or(3000f64),
            stake: stake.unwrap_or(10f64),
            gain: gain.unwrap_or(1f64) / 100f64 + 1f64,
            step_lot: 1f64,
            step_price: 0.0001f64,
            profit: profit.unwrap_or(0.5f64),
            interval: interval.unwrap_or(15),
        }
    }

    pub fn calculate(
        &self,
        results: Vec<u8>,
    ) -> (f64, f64, f64, f64, f64, f64, usize, usize, f64, f64, f64) {
        let mut balance = self.initial_balance;
        let mut opened_orders = vec![];
        let mut executed_orders = vec![];
        let mut wallet = 0f64;
        let mut min_balance = self.initial_balance;

        for (ind, candle) in self.candles.iter().enumerate() {
            min_balance = min_balance.min(balance);

            if ind < results.len() {
                if results[ind] == 1 && balance > self.stake {
                    let curr_stake = floor_to_nearest(self.stake / candle.open, self.step_lot);
                    let order_sum = curr_stake * candle.open;
                    balance -= order_sum;
                    balance -= order_sum * 0.001;

                    opened_orders.push(Order {
                        start_time: candle.start_time,
                        end_time: 0,
                        buy_price: candle.open,
                        sell_price: ceil_to_nearest(candle.open * self.gain, self.step_price),
                        qty: curr_stake,
                        commission: order_sum * 0.001,
                    });

                    opened_orders.sort_by(|a, b| b.sell_price.partial_cmp(&a.sell_price).unwrap());
                }
            }

            let mut cont = true;

            while cont {
                let order = opened_orders.last();

                if order.is_none() || order.unwrap().sell_price > candle.high {
                    cont = false;
                    continue;
                }

                let mut order = opened_orders.pop().unwrap();

                let order_sum = order.sell_price * order.qty;

                balance += order_sum;
                balance -= order_sum * 0.001;
                order.commission += order_sum * 0.001;

                order.end_time = candle.start_time;

                let profit_size = ((order.sell_price - order.buy_price) * order.qty
                    - order.commission)
                    * self.profit;

                balance -= profit_size;
                wallet += profit_size;

                executed_orders.push(order);

                // println!("ru order close: {} {} {}", candle.start_time, balance, wallet);
            }
        }

        let mut base_count = 0f64;
        let mut base_expected = 0f64;
        let mut avg_wait = 0;
        let mut successful_orders = 0;
        for order in &opened_orders {
            base_count += order.qty;
            base_expected += order.qty * order.sell_price;
        }

        let last_candle = self.candles.last().unwrap();

        let base_real = base_count * last_candle.close;

        for order in &executed_orders {
            let time = (order.end_time - order.start_time) + (self.interval * 60 - 1);

            avg_wait += time;

            if time < 12 * 60 * 60 {
                successful_orders += 1
            }
        }

        let drawdown = if opened_orders.len() > 0 {
            (base_real + balance) / (base_expected + balance)
        } else {
            1f64
        };

        let avg_wait = if executed_orders.len() > 0 {
            avg_wait as f64 / executed_orders.len() as f64
        } else {
            0f64
        };

        let successful_ratio = if executed_orders.len() > 0 {
            successful_orders as f64 / executed_orders.len() as f64
        } else {
            0f64
        };

        /*
            wallet, balance, base_real, base_expected, min_balance, drawdown, opened_orders, executed_orders, avg_wait, score, successful_ratio
        */
        (
            wallet,
            balance,
            base_real,
            base_expected,
            min_balance,
            drawdown,
            opened_orders.len(),
            executed_orders.len(),
            avg_wait,
            get_score(wallet, drawdown, successful_ratio),
            successful_ratio,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candle_response::CandleResponse;
    use crate::get_candles::perform_candles;
    use serde_json::from_reader;
    use std::fs::File;

    #[test]
    fn should_work() {
        let file = File::open("tests/candles.json").expect("file should open read only");
        let json = from_reader::<_, Vec<CandleResponse>>(file);

        assert_eq!(json.is_ok(), true);

        let candles = json
            .unwrap()
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<Candle>>();

        let candles = perform_candles(candles, 1655769600, 12, Some(vec![]));

        let results = candles
            .clone()
            .into_iter()
            .map(|candle| if candle.max_profit_12 > 1.0 { 1 } else { 0 })
            .collect::<Vec<u8>>();

        let calculate = Calculate::new(candles, None, None, None, None, None);

        let resp = calculate.calculate(results);

        /* wallet */
        assert_eq!(resp.0, 8.26311035000005);
        /* balance */
        assert_eq!(resp.1, 2998.380237350004);
        /* base_real */
        assert_eq!(resp.2, 9.882000000000001);
        /* base_expected */
        assert_eq!(resp.3, 9.972000000000001);
        /* min_balance */
        assert_eq!(resp.4, 2560.199113750004);
        /* drawdown */
        assert_eq!(resp.5, 0.9999700832904862);
        /* opened_orders */
        assert_eq!(resp.6, 1);
        /* executed_orders */
        assert_eq!(resp.7, 207);
        /* avg_wait  */
        assert_eq!(resp.8, 7909.144927536232);
        /* score */
        assert_eq!(resp.9, 8.262863144928028);
        /* successful_ratio */
        assert_eq!(resp.10, 1.0);
    }
}
