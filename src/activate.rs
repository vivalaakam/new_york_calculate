use crate::order::Order;
use crate::{CalculateCommand, CalculateResult, CandleTrait, Symbol};
use std::collections::HashMap;

pub trait Activate<C> {
    fn activate(
        &self,
        candles: &[C],
        prices: &HashMap<Symbol, f32>,
        results: &CalculateResult,
        active: &HashMap<Symbol, Vec<Order>>,
    ) -> Vec<CalculateCommand>
    where
        C: CandleTrait;

    fn on_order(&mut self, _ts: u64, _order: &Order) {}

    fn on_end(&mut self, _result: CalculateResult) {}
}
