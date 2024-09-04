use crate::symbol::Symbol;
use std::collections::HashMap;

#[derive(Debug)]
pub struct CalculateStats {
    pub balance: f32,
    pub orders: usize,
    pub count: f32,
    pub expected: f32,
    pub real: f32,
    pub assets_stock: HashMap<Symbol, f32>,
    pub assets_fiat: HashMap<Symbol, f32>,
}
