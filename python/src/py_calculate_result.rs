use new_york_calculate_core::CalculateResult;

#[pyclass]
#[derive(Default, FromPyObject)]
pub struct PyCalculateResult {
    #[pyo3(get)]
    pub wallet: f64,
    #[pyo3(get)]
    pub balance: f64,
    #[pyo3(get)]
    pub base_real: f64,
    #[pyo3(get)]
    pub base_expected: f64,
    #[pyo3(get)]
    pub min_balance: f64,
    #[pyo3(get)]
    pub drawdown: f64,
    #[pyo3(get)]
    pub opened_orders: usize,
    #[pyo3(get)]
    pub executed_orders: usize,
    #[pyo3(get)]
    pub avg_wait: f64,
    #[pyo3(get)]
    pub score: f64,
    #[pyo3(get)]
    pub successful_ratio: f64,
}

impl Into<PyCalculateResult> for CalculateResult {
    fn into(self) -> PyCalculateResult {
        PyCalculateResult {
            wallet: self.wallet,
            balance: self.balance,
            base_real: self.base_real,
            base_expected: self.base_expected,
            min_balance: self.min_balance,
            drawdown: self.drawdown,
            opened_orders: self.opened_orders,
            executed_orders: self.executed_orders,
            avg_wait: self.avg_wait,
            score: self.score,
            successful_ratio: self.successful_ratio
        }
    }
}