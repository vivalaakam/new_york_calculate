use crate::{CalculateCommand, CalculateResult, CandleTrait};

pub trait Activate<C> {
    fn activate(&self, candles: &[C], results: &CalculateResult) -> Vec<CalculateCommand>
    where
        C: CandleTrait;

    fn on_end(&mut self, _result: CalculateResult) {}

    fn on_end_round(&mut self, _ts: u64, _result: CalculateResult, _candles: &[C]) {}
}
