use crate::order::Order;
use crate::result::{CalculateResult, CalculateResultRef};
use crate::types::TimeStamp;
use crate::{CalculateCommand, CandleTrait, Symbol};
use std::collections::HashMap;

pub trait Activate<C> {
    fn activate(
        &self,
        candles: &[C],
        prices: &HashMap<&str, f32>,
        stats: CalculateResultRef<'_>,
        active: &HashMap<Symbol, Vec<Order>>,
    ) -> Vec<CalculateCommand>
    where
        C: CandleTrait;

    fn on_order(&mut self, _ts: TimeStamp, _order: &Order) {}

    fn on_end(&mut self, _result: CalculateResult) {}
}
