use std::collections::{HashMap, HashSet};

use crate::calculate_iter::CalculateIter;
use crate::score::get_score;

#[derive(Debug, Default)]
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
    pub gain_ratio: HashMap<String, f64>,
    pub profit_ratio: HashMap<String, f64>,
}

impl From<CalculateIter<'_>> for CalculateResult {
    fn from(calculate: CalculateIter) -> Self {
        let executed_orders = calculate.agent.get_executed_orders();
        let mut base_count = 0f64;
        let mut base_expected = 0f64;

        for order in &calculate.sell_orders {
            base_count += order.qty;
            base_expected += order.qty * order.sell_price;
        }

        let last_candle = calculate.last_candle().unwrap();

        let base_real = base_count * last_candle.close;

        let mut ratios = HashSet::new();
        let mut gain_ratio = HashMap::new();
        let mut profit_ratio = HashMap::new();

        for order in &executed_orders {
            ratios.insert(order.gain.to_string());
            gain_ratio
                .entry(order.gain.to_string())
                .and_modify(|counter| *counter += 1.0)
                .or_insert(1.0);
            profit_ratio
                .entry(order.gain.to_string())
                .and_modify(|counter| *counter += order.profit)
                .or_insert(order.profit);
        }

        for value in ratios {
            gain_ratio
                .entry(value.to_string())
                .and_modify(|counter| *counter /= executed_orders.len() as f64)
                .or_insert(0.0);
            profit_ratio
                .entry(value.to_string())
                .and_modify(|counter| *counter /= calculate.agent.get_wallet())
                .or_insert(0.0);
        }

        let drawdown = if calculate.sell_orders.len() > 0 {
            (base_real + calculate.agent.get_balance())
                / (base_expected + calculate.agent.get_balance())
        } else {
            1f64
        };

        CalculateResult {
            wallet: calculate.agent.get_wallet(),
            balance: calculate.agent.get_balance(),
            base_real,
            base_expected,
            min_balance: calculate.agent.get_min_balance(),
            drawdown,
            opened_orders: calculate.agent.get_orders(),
            executed_orders: calculate.agent.get_executed_orders().len(),
            avg_wait: calculate.agent.get_average_waiting(),
            score: get_score(
                calculate.agent.get_wallet(),
                drawdown,
                calculate.agent.get_successful_ratio(),
            ),
            successful_ratio: calculate.agent.get_successful_ratio(),
            gain_ratio,
            profit_ratio,
        }
    }
}
