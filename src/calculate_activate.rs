use crate::calculate_stats::CalculateStats;
use crate::{CalculateCommand, Candle};

pub trait CalculateActivate {
    fn activate(
        &self,
        candle: &Candle,
        position: usize,
        stats: &CalculateStats,
    ) -> CalculateCommand;
}
