use new_york_calculate_core::{
    Activate, Calculate, CalculateAgent, CalculateCommand, CalculateResult, CandleTrait, Order,
    OrderSide, OrderStatus, OrderType, Symbol, TimeStamp,
};
use rand::Rng;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use std::{env, fs};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Default)]
struct CalculateIterData {
    score: f32,
    sell_orders: HashMap<TimeStamp, Vec<Uuid>>,
}

#[derive(Debug, Default)]
struct CalculateIterActivate {
    data: Mutex<CalculateIterData>,
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
    fn get_start_time(&self) -> TimeStamp {
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
        let Some(candle) = candles.last() else {
            return vec![];
        };

        let mut actions = vec![];
        let mut data = self.data.lock().unwrap();

        data.score = stats.balance
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
            step = candle.get_start_time(),
            score = data.score,
            balance = stats.balance,
            assets_frozen = ?stats.assets_frozen,
            assets_available =  ?stats.assets_available,
            "score"
        );

        let exists = data
            .sell_orders
            .keys()
            .into_iter()
            .filter(|k| *k + 1200 < candle.get_start_time())
            .collect::<Vec<_>>();

        for key in exists {
            if let Some(orders) = data.sell_orders.get(key) {
                for order_id in orders.iter() {
                    info!(order_id = ?order_id, "cancel order");
                    actions.push(CalculateCommand::CancelLimit {
                        id: *order_id,
                        symbol: candle.get_symbol(),
                    });

                    actions.push(CalculateCommand::SellMarket {
                        symbol: candle.get_symbol(),
                        stake: 100.0,
                    })
                }
            }
        }

        if candle.start_time % 1800 == 0 {
            actions.push(CalculateCommand::BuyMarket {
                symbol: candle.get_symbol(),
                stake: 100.0,
            });

            actions.push(CalculateCommand::SellLimit {
                symbol: candle.get_symbol(),
                stake: 100.0,
                price: prices.get(&candle.get_symbol()).unwrap_or(&0.0) * 1.01,
                expiration: None,
            })
        }

        actions
    }

    fn on_order(&mut self, ts: TimeStamp, order: &Order) {
        if order.side == OrderSide::Sell && order.order_type == OrderType::Limit {
            let mut data = self.data.lock().unwrap();

            if order.status == OrderStatus::Open {
                data.sell_orders
                    .entry(ts)
                    .or_insert_with(Vec::new)
                    .push(order.id);
                info!(ts, order = ?order, "on_order");
            } else {
                if let Some(orders) = data.sell_orders.get_mut(&ts) {
                    orders.retain(|&id| id != order.id);
                    if orders.is_empty() {
                        data.sell_orders.remove(&ts);
                    }
                }
            }
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
            start_time: row[0].as_u64().unwrap() / 1000,
            open: row[1].as_str().unwrap().parse().unwrap(),
            high: row[2].as_str().unwrap().parse().unwrap(),
            low: row[3].as_str().unwrap().parse().unwrap(),
            close: row[4].as_str().unwrap().parse().unwrap(),
        })
    }

    let activate = CalculateIterActivate::default();

    let mut agents = vec![];

    let agent = CalculateAgent::new(300.0, 0.0001, Box::new(&activate));
    agents.push(agent);

    let mut calculate_iter = Calculate::new(&candles, agents);

    let mut cont = Some(());

    while cont.is_some() {
        cont = calculate_iter.next();
    }

    calculate_iter.on_end();

    info!("activate: {activate:?}",);
}
