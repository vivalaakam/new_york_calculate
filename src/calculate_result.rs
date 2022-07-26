use crate::calculate_iter::CalculateIter;
use crate::score::get_score;

pub struct CalculateResult {
    pub wallet: f64,
    pub balance: f64,
    pub base_real: f64,
    pub base_expected: f64,
    pub min_balance: f64,
    pub drawdown: f64,
    pub opened_orders: usize,
    pub executed_orders: usize,
    pub avg_wait: f64,
    pub score: f64,
    pub successful_ratio: f64,
}

impl From<CalculateIter<'_>> for CalculateResult {
    fn from(calculate: CalculateIter) -> Self {
        let mut base_count = 0f64;
        let mut base_expected = 0f64;
        let mut avg_wait = 0;
        let mut successful_orders = 0;
        for order in &calculate.opened_orders {
            base_count += order.qty;
            base_expected += order.qty * order.sell_price;
        }

        let last_candle = calculate.last_candle().unwrap();

        let base_real = base_count * last_candle.close;

        for order in &calculate.executed_orders {
            let time = (order.end_time - order.start_time) + (calculate.interval * 60 - 1);

            avg_wait += time;

            if time < 12 * 60 * 60 {
                successful_orders += 1
            }
        }

        let drawdown = if calculate.opened_orders.len() > 0 {
            (base_real + calculate.balance) / (base_expected + calculate.balance)
        } else {
            1f64
        };

        let avg_wait = if calculate.executed_orders.len() > 0 {
            avg_wait as f64 / calculate.executed_orders.len() as f64
        } else {
            0f64
        };

        let successful_ratio = if calculate.executed_orders.len() > 0 {
            successful_orders as f64 / calculate.executed_orders.len() as f64
        } else {
            0f64
        };

        CalculateResult {
            wallet: calculate.wallet,
            balance: calculate.balance,
            base_real,
            base_expected,
            min_balance: calculate.min_balance,
            drawdown,
            opened_orders: calculate.opened_orders.len(),
            executed_orders: calculate.executed_orders.len(),
            avg_wait,
            score: get_score(calculate.wallet, drawdown, successful_ratio),
            successful_ratio,
        }
    }
}
