#[derive(Debug, Default, Clone)]
pub struct CalculateResult {
    pub balance: f32,
    pub min_balance: f32,
    pub balance_assets: f32,
    pub opened_orders: usize,
    pub executed_orders: usize,
}
