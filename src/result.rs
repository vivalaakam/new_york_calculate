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

/// A lightweight view into the agent's state, borrowed from the agent's
/// internal storage. Use this in hot paths where cloning the owned
/// `CalculateResult` would be wasteful.
#[derive(Debug, Clone, Copy)]
pub struct CalculateResultRef<'a> {
    pub balance: f32,
    pub min_balance: f32,
    pub opened_orders: usize,
    pub executed_orders: usize,
    pub assets_available: &'a HashMap<Symbol, f32>,
    pub assets_frozen: &'a HashMap<Symbol, f32>,
}

impl From<CalculateResultRef<'_>> for CalculateResult {
    fn from(r: CalculateResultRef<'_>) -> Self {
        CalculateResult {
            balance: r.balance,
            min_balance: r.min_balance,
            opened_orders: r.opened_orders,
            executed_orders: r.executed_orders,
            assets_available: r.assets_available.clone(),
            assets_frozen: r.assets_frozen.clone(),
        }
    }
}
