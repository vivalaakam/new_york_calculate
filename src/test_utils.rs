use crate::{CandleTrait, Symbol};
use std::env;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn init_tracing() {
    INIT.call_once(|| {
        let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
        let log_level = log_level
            .parse::<tracing::Level>()
            .unwrap_or(tracing::Level::INFO);

        tracing_subscriber::fmt().with_max_level(log_level).init();
    });
}

#[derive(Clone, Debug)]
pub struct Candle {
    pub start_time: u64,
    pub symbol: Symbol,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
}

impl CandleTrait for Candle {
    fn get_start_time(&self) -> u64 {
        self.start_time
    }

    fn get_symbol(&self) -> Symbol {
        self.symbol.clone()
    }

    fn get_open(&self) -> f32 {
        self.open
    }

    fn get_high(&self) -> f32 {
        self.high
    }

    fn get_low(&self) -> f32 {
        self.low
    }

    fn get_close(&self) -> f32 {
        self.close
    }
}
