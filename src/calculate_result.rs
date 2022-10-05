use crate::calculate_iter::CalculateIter;

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
}

impl From<CalculateIter<'_>> for CalculateResult {
    fn from(calculate: CalculateIter) -> Self {
        calculate.agent.get_result()
    }
}
