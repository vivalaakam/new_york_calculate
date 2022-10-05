use crate::calculate_stats::CalculateStats;
use crate::{CalculateCommand, CalculateResult, Candle};

pub trait CalculateActivate {
    fn activate(
        &self,
        candle: &Candle,
        position: usize,
        stats: &CalculateStats,
    ) -> CalculateCommand;

    fn on_end(&mut self, result: CalculateResult);
}
