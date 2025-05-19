use crate::Symbol;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct CalculateResult {
    pub balance: f32,
    pub min_balance: f32,
    pub opened_orders: usize,
    pub executed_orders: usize,
    pub assets_available: HashMap<Symbol, f32>,
    pub assets_frozen: HashMap<Symbol, f32>,
}
