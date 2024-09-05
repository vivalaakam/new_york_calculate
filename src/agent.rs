use std::cmp::PartialEq;
use std::collections::HashMap;
use std::marker::PhantomData;

use thiserror::Error;
use tracing::debug;
use uuid::Uuid;

use crate::activate::Activate;
use crate::candle::CandleTrait;
use crate::order::{Order, OrderSide, OrderStatus, OrderType};
use crate::symbol::Symbol;
use crate::{CalculateCommand, CalculateResult, CalculateStats};

pub struct CalculateAgent<T: Activate<C> + ?Sized, C: CandleTrait> {
    balance: f32,
    commission: f32,
    balance_assets: f32,
    min_balance: f32,
    portfolio_stock: HashMap<Symbol, f32>,
    portfolio_fiat: HashMap<Symbol, f32>,
    activate: Box<T>,
    queue_orders: HashMap<(Symbol, OrderSide), Vec<Order>>,
    executed_orders: Vec<Order>,
    candle: PhantomData<C>,
}

#[derive(Debug, Error, PartialEq)]
pub enum CalculateAgentError {
    #[error("Not enough balance: {0} < {1}")]
    NotEnoughBalance(f32, f32),
    #[error("Not enough asset balance {0}: {1} < {2}")]
    NotAssetEnoughBalance(Symbol, f32, f32),
    #[error("Unknown command")]
    UnknownCommand,
}

impl<T, C> CalculateAgent<T, C>
where
    T: Activate<C> + ?Sized,
    C: CandleTrait,
{
    pub fn new(balance: f32, commission: f32, activate: Box<T>) -> CalculateAgent<T, C> {
        CalculateAgent {
            balance,
            activate,
            commission,
            min_balance: balance,
            balance_assets: 0.0,
            executed_orders: Default::default(),
            queue_orders: Default::default(),
            portfolio_stock: Default::default(),
            portfolio_fiat: Default::default(),
            candle: PhantomData,
        }
    }

    pub fn activate(&self, candle: &C, candles: &Vec<C>) -> CalculateCommand {
        self.activate
            .activate(candle, candles, &self.get_stats(candle))
    }

    pub fn get_stats(&self, candle: &C) -> CalculateStats {
        let count = self
            .portfolio_stock
            .get(&candle.get_symbol())
            .unwrap_or(&0.0);
        let orders = self
            .queue_orders
            .get(&(candle.get_symbol(), OrderSide::Buy))
            .unwrap_or(&vec![])
            .len();

        CalculateStats {
            balance: self.balance,
            orders,
            count: *count,
            expected: 0f32,
            real: count * candle.get_open(),
            assets_stock: self.portfolio_stock.clone(),
            assets_fiat: self.portfolio_fiat.clone(),
        }
    }

    pub fn buy_order(
        &mut self,
        candle: &C,
        price: f32,
        qty: f32,
        order_type: OrderType,
        id: Option<Uuid>,
    ) -> Result<Order, CalculateAgentError> {
        let order_sum = qty * price;

        if self.balance < order_sum {
            return Err(CalculateAgentError::NotEnoughBalance(self.balance, qty));
        }

        self.balance -= order_sum;
        self.balance -= order_sum * self.commission;

        let status = match order_type {
            OrderType::Market => OrderStatus::Close,
            OrderType::Limit => OrderStatus::Open,
        };

        let order = Order {
            uid: Uuid::new_v4(),
            ts: candle.get_start_time(),
            price,
            qty,
            symbol: candle.get_symbol(),
            id,
            commission: order_sum * self.commission,
            status,
            side: OrderSide::Buy,
            order_type,
        };

        if matches!(order_type, OrderType::Market) {
            self.portfolio_stock
                .entry(candle.get_symbol())
                .and_modify(|v| *v += qty)
                .or_insert(qty);

            self.executed_orders.push(order.clone());
        } else if matches!(order_type, OrderType::Limit) {
            let queue = self
                .queue_orders
                .entry((order.symbol.clone(), order.side.clone()))
                .or_default();

            queue.push(order.clone());
        }

        Ok(order)
    }

    pub fn sell_order(
        &mut self,
        candle: &C,
        price: f32,
        qty: f32,
        order_type: OrderType,
        id: Option<Uuid>,
    ) -> Result<Order, CalculateAgentError> {
        let portfolio_amount = self
            .portfolio_stock
            .get(&candle.get_symbol())
            .unwrap_or(&0.0);

        if qty > *portfolio_amount {
            return Err(CalculateAgentError::NotAssetEnoughBalance(
                candle.get_symbol(),
                *portfolio_amount,
                qty,
            ));
        }

        let order_sum = qty * price;

        let status = match order_type {
            OrderType::Market => OrderStatus::Close,
            OrderType::Limit => OrderStatus::Open,
        };

        let order = Order {
            uid: Uuid::new_v4(),
            ts: candle.get_start_time(),
            price,
            symbol: candle.get_symbol(),
            id,
            qty,
            commission: order_sum * self.commission,
            status,
            side: OrderSide::Sell,
            order_type,
        };

        if matches!(order_type, OrderType::Market) {
            self.balance += order_sum;
            self.balance -= order_sum * self.commission;

            self.executed_orders.push(order.clone());
        } else if matches!(order_type, OrderType::Limit) {
            let queue = self
                .queue_orders
                .entry((order.symbol.clone(), order.side.clone()))
                .or_default();
            queue.push(order.clone());
        }

        self.portfolio_stock
            .entry(candle.get_symbol())
            .and_modify(|v| *v -= qty);

        Ok(order)
    }

    pub fn buy_profit_order(
        &mut self,
        candle: &C,
        price: f32,
        amount: f32,
        gain: f32,
        id: Option<Uuid>,
    ) -> Result<Order, CalculateAgentError> {
        let buy_order = self.buy_order(candle, price, amount, OrderType::Market, id)?;
        self.sell_order(
            candle,
            buy_order.price * gain,
            buy_order.qty,
            OrderType::Limit,
            id,
        )
    }

    pub fn perform_order(
        &mut self,
        command: CalculateCommand,
        candle: &C,
    ) -> Result<Option<Order>, CalculateAgentError> {
        match command {
            CalculateCommand::BuyMarket { stake } => self
                .buy_order(
                    candle,
                    candle.get_open(),
                    stake,
                    OrderType::Market,
                    Some(Uuid::new_v4()),
                )
                .map(Some),
            CalculateCommand::SellMarket { stake } => self
                .sell_order(
                    candle,
                    candle.get_open(),
                    stake,
                    OrderType::Market,
                    Some(Uuid::new_v4()),
                )
                .map(Some),
            CalculateCommand::BuyLimit { stake, price } => self
                .buy_order(candle, price, stake, OrderType::Limit, Some(Uuid::new_v4()))
                .map(Some),
            CalculateCommand::SellLimit { stake, price } => self
                .sell_order(candle, price, stake, OrderType::Limit, Some(Uuid::new_v4()))
                .map(Some),
            CalculateCommand::BuyProfit { stake, profit } => self
                .buy_profit_order(
                    candle,
                    candle.get_open(),
                    stake,
                    profit,
                    Some(Uuid::new_v4()),
                )
                .map(Some),
            CalculateCommand::None | CalculateCommand::Unknown => Ok(None),
        }
    }

    pub fn perform_candle(&mut self, candle: &C) {
        if let Some(orders) = self
            .queue_orders
            .get_mut(&(candle.get_symbol(), OrderSide::Buy))
        {
            let mut to_remove = vec![];
            for order in orders.iter_mut() {
                if order.price > candle.get_low() {
                    order.status = OrderStatus::Close;
                    self.portfolio_stock
                        .entry(candle.get_symbol())
                        .and_modify(|v| *v += order.qty)
                        .or_insert(order.qty);
                    to_remove.push(order.uid);
                    self.executed_orders.push(order.clone());
                }
            }

            orders.retain(|o| !to_remove.contains(&o.uid));
        }

        if let Some(orders) = self
            .queue_orders
            .get_mut(&(candle.get_symbol(), OrderSide::Sell))
        {
            let mut to_remove = vec![];

            for order in orders.iter_mut() {
                if order.price < candle.get_high() {
                    order.status = OrderStatus::Close;
                    to_remove.push(order.uid);

                    self.balance += order.price * order.qty;
                    self.balance -= (order.price * order.qty) * 0.001;

                    self.executed_orders.push(order.clone());
                }
            }

            orders.retain(|o| !to_remove.contains(&o.uid));
        }

        if let Some(order_sum) = self.portfolio_stock.get(&candle.get_symbol()) {
            self.portfolio_fiat
                .insert(candle.get_symbol(), order_sum * candle.get_close());
        }

        debug!(
            "perform_candle Agent portfolio: {:?}, fiat: {:?}",
            self.portfolio_stock, self.portfolio_fiat
        );
    }

    pub fn get_result(&self) -> CalculateResult {
        debug!(
            "Agent result: {:?} queue: {:?}",
            self.balance, self.queue_orders
        );

        CalculateResult {
            balance: self.balance,
            min_balance: self.min_balance,
            opened_orders: self
                .queue_orders
                .iter()
                .fold(0, |acc, (_, v)| acc + v.len()),
            executed_orders: self.executed_orders.len(),
            balance_assets: self.balance_assets,
            assets_stock: self.portfolio_stock.clone(),
            assets_fiat: self.portfolio_fiat.clone(),
        }
    }

    pub fn on_end(&mut self) {
        self.activate.on_end(self.get_result())
    }

    pub fn on_end_round(&mut self) {
        self.min_balance = self.min_balance.min(self.balance);
        self.balance_assets = self.portfolio_fiat.values().sum();
        self.activate.on_end_round(self.get_result());
    }
}

#[cfg(test)]
mod tests {
    use crate::candle::CandleTrait;
    use crate::symbol::Symbol;
    use crate::{Activate, CalculateAgent, CalculateCommand, CalculateStats};

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

    struct CalculateIterActivate {}

    impl Activate<Candle> for CalculateIterActivate {
        fn activate(
            &self,
            _candle: &Candle,
            _candles: &Vec<Candle>,
            _stats: &CalculateStats,
        ) -> CalculateCommand {
            CalculateCommand::None
        }
    }

    #[test]
    fn test_calculate_agent_market() {
        // tracing_subscriber::fmt::init();

        let mut agent = CalculateAgent::new(1000.0, 0.0001, Box::new(CalculateIterActivate {}));

        let candle_1 = Candle {
            symbol: "BTC".to_string(),
            start_time: 0,
            open: 100.0,
            high: 120.0,
            low: 90.0,
            close: 110.0,
        };

        let result = agent.perform_order(CalculateCommand::BuyMarket { stake: 5.0 }, &candle_1);

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_1);

        agent.on_end_round();

        let results = agent.get_result();

        println!("{:?}", agent.get_result());

        assert_eq!(results.balance, 499.95);
        assert_eq!(results.balance_assets, 550.0);
        assert_eq!(results.opened_orders, 0);
        assert_eq!(results.executed_orders, 1);

        let candle_2 = Candle {
            symbol: "BTC".to_string(),
            start_time: 1,
            open: 120.0,
            high: 130.0,
            low: 90.0,
            close: 110.0,
        };

        let result = agent.perform_order(CalculateCommand::SellMarket { stake: 5.0 }, &candle_2);

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_2);

        agent.on_end_round();

        let results = agent.get_result();

        println!("{:?}", agent.get_result());

        assert_eq!(results.balance, 1099.8899);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 0);
        assert_eq!(results.executed_orders, 2);
    }

    #[test]
    fn test_calculate_agent_limit() {
        let mut agent = CalculateAgent::new(1000.0, 0.0001, Box::new(CalculateIterActivate {}));

        let candle_1 = Candle {
            symbol: "BTC".to_string(),
            start_time: 0,
            open: 100.0,
            high: 120.0,
            low: 90.0,
            close: 110.0,
        };

        let result = agent.perform_order(
            CalculateCommand::BuyLimit {
                price: 85.0,
                stake: 5.0,
            },
            &candle_1,
        );

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_1);

        agent.on_end_round();

        let results = agent.get_result();

        println!("{:?}", agent.get_result());

        assert_eq!(results.balance, 574.9575);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 1);
        assert_eq!(results.executed_orders, 0);

        let candle_2 = Candle {
            symbol: "BTC".to_string(),
            start_time: 1,
            open: 120.0,
            high: 130.0,
            low: 80.0,
            close: 110.0,
        };

        agent.perform_candle(&candle_2);

        agent.on_end_round();

        let results = agent.get_result();

        println!("{:?}", agent.get_result());

        assert_eq!(results.balance, 574.9575);
        assert_eq!(results.balance_assets, 550.0);
        assert_eq!(results.opened_orders, 0);
        assert_eq!(results.executed_orders, 1);

        let candle_3 = Candle {
            symbol: "BTC".to_string(),
            start_time: 3,
            open: 120.0,
            high: 130.0,
            low: 90.0,
            close: 110.0,
        };

        let result = agent.perform_order(
            CalculateCommand::SellLimit {
                stake: 5.0,
                price: 135.0,
            },
            &candle_3,
        );

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_3);

        agent.on_end_round();

        let results = agent.get_result();

        println!("{:?}", agent.get_result());

        assert_eq!(results.balance, 574.9575);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 1);
        assert_eq!(results.executed_orders, 1);

        let candle_4 = Candle {
            symbol: "BTC".to_string(),
            start_time: 3,
            open: 120.0,
            high: 140.0,
            low: 90.0,
            close: 110.0,
        };

        agent.perform_candle(&candle_4);

        agent.on_end_round();

        let results = agent.get_result();

        println!("{:?}", agent.get_result());

        assert_eq!(results.balance, 1249.2825);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 0);
        assert_eq!(results.executed_orders, 2);
    }
}
