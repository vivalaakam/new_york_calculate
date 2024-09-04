use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{env, fs};

use new_york_calculate_core::{
    Activate, Calculate, CalculateAgent, CalculateCommand, CalculateResult, CalculateStats, Candle,
};
use serde_json::Value;
use tracing::info;

#[derive(Debug)]
struct CalculateIterActivate {
    score: Mutex<f32>,
    step: Arc<Mutex<u32>>,
}

impl Activate for &CalculateIterActivate {
    fn activate(&self, candle: &Candle, stats: &CalculateStats) -> CalculateCommand {
        let mut step = self.step.lock().unwrap();
        *step += 1;

        info!("step: {step:?} {stats:?} {candle:?}");

        return match *step % 8u32 {
            0 => {
                info!("BuyMarket");
                CalculateCommand::BuyMarket { stake: 10.0 }
            }
            4 => {
                info!("SellMarket");
                CalculateCommand::SellMarket { stake: 10.0 }
            }
            _ => CalculateCommand::None,
        };
    }

    fn on_end(&mut self, result: CalculateResult) {
        info!("on_end: {result:?}");
        let mut score = self.score.lock().unwrap();
        *score = result.balance
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let mut candles = HashMap::new();

    let json_path = env::current_dir().unwrap().join("./tests/candles.json");

    let file = fs::File::open(json_path).expect("file should open read only");
    let resp = serde_json::from_reader::<_, Vec<Value>>(file).expect("file should be proper JSON");

    for row in resp {
        let ts = candles.entry(row[0].as_u64().unwrap()).or_insert(vec![]);
        ts.push(Candle {
            start_time: row[0].as_u64().unwrap(),
            end_time: row[6].as_u64().unwrap(),
            symbol: "test".to_string(),
            open: row[1].as_str().unwrap().parse().unwrap(),
            high: row[2].as_str().unwrap().parse().unwrap(),
            low: row[3].as_str().unwrap().parse().unwrap(),
            close: row[4].as_str().unwrap().parse().unwrap(),
            volume: row[5].as_str().unwrap().parse().unwrap(),
            trades: row[8].as_u64().unwrap() as f32,
        })
    }

    let mut activates = vec![];

    activates.push(CalculateIterActivate {
        score: Mutex::new(0f32),
        step: Arc::new(Mutex::new(0)),
    });

    let mut agents = vec![];

    for activate in activates.iter().enumerate() {
        let agent = CalculateAgent::new(activate.0, 3000.0, 0.0001, Box::new(activate.1));
        agents.push(agent);
    }

    let mut calculate_iter = Calculate::new(&candles, 10, agents);

    let mut cont = Some(());

    while cont.is_some() {
        cont = calculate_iter.next();
    }

    calculate_iter.on_end();

    for activate in activates.iter_mut() {
        info!("activate: {activate:?}",);
    }
}
