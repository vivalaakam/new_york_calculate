use std::collections::HashMap;

use crate::candle::Candle;
use crate::utils::{ceil_to_nearest, floor_to_nearest};

#[pyclass]
pub struct Calculate {
    candles: HashMap<u64, Candle>,
    keys: Vec<u64>,
    initial_balance: f32,
    stake: f32,
    gain: f32,
    inner_gain: f32,
    profit: f32,
    step_lot: f32,
    step_price: f32,
    interval: u64,
}

impl Calculate {
    pub fn new(
        candles: HashMap<u64, Candle>,
        initial_balance: Option<f32>,
        stake: Option<f32>,
        gain: Option<f32>,
        profit: Option<f32>,
        interval: Option<u64>,
    ) -> Self {
        let mut keys: Vec<u64> = candles.keys().collect();

        keys.sort();

        Calculate {
            candles,
            keys,
            initial_balance: initial_balance.unwrap_or(3000f32),
            stake: stake.unwrap_or(10f32),
            gain: gain.unwrap_or(1f32),
            inner_gain: gain.unwrap_or(1f32) / 100f32 + 1f32,
            step_lot: 1f32,
            step_price: 0.0001f32,
            profit: profit.unwrap_or(0.5f32),
            interval: interval.unwrap_or(15),
        }
    }

    pub fn get_keys(&self) -> Vec<u64> {
        self.keys.to_vec()
    }

    pub fn get_first(&self) -> u64 {
        *self.keys.get(0).unwrap()
    }

    pub fn calculate(&self, results: HashMap<u64, bool>) -> (f32, f32, f32, f32, f32, f32, usize, usize, f32) {
        let mut balance = self.initial_balance;
        let mut opened_orders = vec![];
        let mut executed_orders = vec![];
        let mut wallet = 0f32;
        let mut min_balance = self.initial_balance;

        for key in &self.keys {
            min_balance = min_balance.min(balance);

            let candle = self.candles.get(key).unwrap();
            let result = *results.get(key).unwrap_or(&false);

            if result == true && balance > self.stake {
                let curr_stake = floor_to_nearest(self.stake / candle.open, self.step_lot);
                let order_sum = curr_stake * candle.open;
                balance -= order_sum;
                balance -= order_sum * 0.001;

                opened_orders.push(Order {
                    start_time: candle.start_time,
                    end_time: 0,
                    buy_price: candle.open,
                    sell_price: ceil_to_nearest(candle.open * self.inner_gain, self.step_price),
                    qty: curr_stake,
                    commission: order_sum * 0.001,
                });

                opened_orders.sort_by(|a, b| b.sell_price.partial_cmp(&a.sell_price).unwrap());
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
            }
        }

        let mut base_count = 0f32;
        let mut base_sum = 0f32;
        let mut avg_wait = 0f32;

        for order in &opened_orders {
            base_count += order.qty;
            base_sum += order.qty * order.sell_price;
        }

        let total = executed_orders.len();

        for order in &executed_orders {
            avg_wait += ((order.end_time - order.start_time) + (self.interval * 60 - 1)) as f32;
        }

        let last_candle = self.candles.get(self.keys.last().unwrap()).unwrap();

        let drawdown = if total > 0 {
            (base_count * last_candle.close) / base_sum
        } else {
            1f32
        };

        let avg_wait = if total > 0 {
            avg_wait / total as f32
        } else {
            0f32
        };

        /*
            wallet, balance, base_real, base_expected, min_balance, drawdown, opened_orders, executed_orders, avg_wait
        */
        (
            wallet,
            balance,
            base_count * last_candle.close,
            base_sum,
            min_balance,
            drawdown,
            total,
            executed_orders.len(),
            avg_wait,
        )
    }
}
