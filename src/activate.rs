use crate::stats::CalculateStats;
use crate::{CalculateCommand, CalculateResult, CandleTrait};

pub trait Activate<C> {
    fn activate(&self, candle: &C, candles: &Vec<C>, stats: &CalculateStats) -> CalculateCommand
    where
        C: CandleTrait;

    fn on_end(&mut self, _result: CalculateResult) {}

    fn on_end_round(&mut self, _result: CalculateResult) {}
}
