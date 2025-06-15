use std::collections::HashMap;
use std::sync::Mutex;
use std::{env, fs};

use serde_json::Value;
use tracing::info;

use new_york_calculate_core::{
    buy_market, sell_market, Activate, Calculate, CalculateAgent, CalculateCommand,
    CalculateResult, CandleTrait, Order, Symbol,
};

#[derive(Debug)]
struct CalculateIterActivate {
    score: Mutex<f32>,
    step: Mutex<u32>,
}

#[derive(Clone, Debug)]
pub struct Candle {
    pub start_time: u64,
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
        "test".to_string()
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

impl Activate<Candle> for &CalculateIterActivate {
    fn activate(
        &self,
        candles: &[Candle],
        prices: &HashMap<Symbol, f32>,
        stats: &CalculateResult,
        _active: &HashMap<Symbol, Vec<Order>>,
    ) -> Vec<CalculateCommand> {
        let mut step: std::sync::MutexGuard<'_, u32> = self.step.lock().unwrap();
        *step += 1;

        let mut score = self.score.lock().unwrap();
        *score = stats.balance
            + stats
                .assets_frozen
                .iter()
                .map(|r| prices.get(r.0).unwrap_or(&0f32) * r.1)
                .sum::<f32>()
            + stats
                .assets_available
                .iter()
                .map(|r| prices.get(r.0).unwrap_or(&0f32) * r.1)
                .sum::<f32>();

        info!(
            step = *step,
            score = *score,
            balance = stats.balance,
            assets_frozen = ?stats.assets_frozen,
            assets_available =  ?stats.assets_available,
            "score"
        );

        let Some(candle) = candles.last() else {
            return vec![];
        };

        match *step % 8u32 {
            0 => {
                info!("BuyMarket");
                vec![buy_market!(candle.get_symbol(), 100.0)]
            }
            4 => {
                info!("SellMarket");
                vec![sell_market!(candle.get_symbol(), 100.0)]
            }
            _ => vec![CalculateCommand::None],
        }
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
            open: row[1].as_str().unwrap().parse().unwrap(),
            high: row[2].as_str().unwrap().parse().unwrap(),
            low: row[3].as_str().unwrap().parse().unwrap(),
            close: row[4].as_str().unwrap().parse().unwrap(),
        })
    }

    let activate = CalculateIterActivate {
        score: Mutex::new(0f32),
        step: Mutex::new(0),
    };

    let mut agents = vec![];

    let agent = CalculateAgent::new(3000.0, 0.0001, Box::new(&activate));
    agents.push(agent);

    let mut calculate_iter = Calculate::new(&candles, agents);

    let mut cont = Some(());

    while cont.is_some() {
        cont = calculate_iter.next();
    }

    calculate_iter.on_end();

    info!("activate: {activate:?}",);
}
