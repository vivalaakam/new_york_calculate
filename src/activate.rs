use crate::order::Order;
use crate::{CalculateCommand, CalculateResult, CandleTrait, Symbol};
use std::collections::HashMap;

pub trait Activate<C> {
    fn activate(
        &self,
        candles: &[C],
        results: &CalculateResult,
        active: &HashMap<Symbol, Vec<Order>>,
    ) -> Vec<CalculateCommand>
    where
        C: CandleTrait;

    fn on_end(&mut self, _result: CalculateResult) {}

    fn on_end_round(
        &mut self,
        _ts: u64,
        _result: CalculateResult,
        _candles: &[C],
        _executed_orders: &[Order],
    ) {
    }
}
