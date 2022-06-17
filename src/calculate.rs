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

    pub fn calculate(&self, results: Vec<u8>) -> (f64, f64, f64, f64, f64, f64, usize, usize, f64, f64) {
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

                    // println!(
                    //     "ru order create: {} {} {} {}",
                    //     candle.start_time, curr_stake, order_sum, balance
                    // )
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
        let mut base_sum = 0f64;
        let mut avg_wait = 0f64;

        for order in &opened_orders {
            base_count += order.qty;
            base_sum += order.qty * order.sell_price;
        }

        for order in &executed_orders {
            avg_wait += ((order.end_time - order.start_time) + (self.interval * 60 - 1)) as f64;
        }

        let last_candle = self.candles.last().unwrap();

        let drawdown = if opened_orders.len() > 0 {
            (base_count * last_candle.close) / base_sum
        } else {
            1f64
        };

        let avg_wait = if executed_orders.len() > 0 {
            avg_wait / executed_orders.len() as f64
        } else {
            0f64
        };

        /*
            wallet, balance, base_real, base_expected, min_balance, drawdown, opened_orders, executed_orders, avg_wait, score
        */
        (
            wallet,
            balance,
            base_count * last_candle.close,
            base_sum,
            min_balance,
            drawdown,
            opened_orders.len(),
            executed_orders.len(),
            avg_wait,
            get_score(wallet, drawdown)
        )
    }
}
