use crate::types::TimeStamp;

pub trait CandleTrait {
    fn get_start_time(&self) -> TimeStamp;
    fn get_symbol(&self) -> &str;
    fn get_open(&self) -> f32;
    fn get_high(&self) -> f32;
    fn get_low(&self) -> f32;
    fn get_close(&self) -> f32;
}
