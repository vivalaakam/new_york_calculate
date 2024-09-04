use crate::stats::CalculateStats;
use crate::{CalculateCommand, CalculateResult, Candle};

pub trait Activate {
    fn activate(&self, candle: &Candle, stats: &CalculateStats) -> CalculateCommand;

    fn on_end(&mut self, result: CalculateResult);
}
