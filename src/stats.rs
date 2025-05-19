use crate::symbol::Symbol;
use std::collections::HashMap;

#[derive(Debug)]
pub struct CalculateStats<'a> {
    pub balance: f32,
    pub orders: usize,
    pub count: f32,
    pub expected: f32,
    pub real: f32,
    pub assets_available: &'a HashMap<Symbol, f32>,
    pub assets_frozen: &'a HashMap<Symbol, f32>,
}
