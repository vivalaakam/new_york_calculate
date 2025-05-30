use crate::types::{Symbol, TimeStamp};

pub trait CandleTrait {
    fn get_start_time(&self) -> TimeStamp;
    fn get_symbol(&self) -> Symbol;
    fn get_open(&self) -> f32;
    fn get_high(&self) -> f32;
    fn get_low(&self) -> f32;
    fn get_close(&self) -> f32;
}
