use crate::symbol::Symbol;

pub trait CandleTrait {
    fn get_start_time(&self) -> u64;
    fn get_symbol(&self) -> Symbol;
    fn get_open(&self) -> f32;
    fn get_high(&self) -> f32;
    fn get_low(&self) -> f32;
    fn get_close(&self) -> f32;
}
