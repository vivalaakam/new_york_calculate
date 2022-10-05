use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use log::LevelFilter;

use new_york_calculate_core::{
    get_candles_with_cache, CalculateActivate, CalculateAgent, CalculateAgentIter,
    CalculateCommand, CalculateResult, CalculateStats, Candle,
};

#[derive(Debug)]
struct CalculateIterActivate {
    gain: f64,
    profit: f64,
    target: usize,
    score: Mutex<f64>,
}

impl CalculateActivate for &CalculateIterActivate {
    fn activate(
        &self,
        candle: &Candle,
        position: usize,
        _stats: &CalculateStats,
    ) -> CalculateCommand {
        if position >= self.target {
            return CalculateCommand::None;
        }

        if (candle.max_profit.last().unwrap_or(&0f64) / 100f64) + 1f64 > self.gain {
            return CalculateCommand::BuyProfit(self.gain, self.profit);
        }

        CalculateCommand::None
    }

    fn on_end(&mut self, result: CalculateResult) {
        println!("{:?}", result);
        let mut data = self.score.lock().unwrap();
        *data = result.score
    }
}

#[tokio::main]
async fn main() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .is_test(true)
        .try_init();

    let mut candles = vec![];

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    let next = (now / 86400000) as u64;
    let mut keys = vec![];

    let mut from = (next - 95) * 86400;
    let to = from + 92 * 86400;

    while from <= to {
        keys.push(from);
        from += 86400;
    }

    keys.push(from);

    for key in keys {
        let new_candles = get_candles_with_cache("XRPBUSD".to_string(), 15, key, 12, None).await;
        candles = [candles, new_candles].concat();
    }

    candles.sort();

    let gain = [1.03, 1.0125, 1.01, 1.0075, 1.005];
    let profit = [500.0, 200.0, 100.0, 50.0, 25.0];
    let target = candles.len() - 288;

    let mut activates = vec![];

    for g in gain {
        for p in profit {
            activates.push(CalculateIterActivate {
                profit: p,
                gain: g,
                target,
                score: Mutex::new(0f64),
            });
        }
    }

    let mut agents = vec![];

    for activate in activates.iter().enumerate() {
        let agent = CalculateAgent::new(activate.0, 3000f64, false, Box::new(activate.1));

        agents.push(agent);
    }

    let mut calculate_iter = CalculateAgentIter::new(&candles, 0.5, 1f64, 0.0001f64, agents);

    let mut cont = Ok(());

    while cont.is_ok() {
        cont = calculate_iter.next();
    }

    for activate in activates {
        println!("{:?}", activate);
    }
}
