use std::collections::HashMap;

use thiserror::Error;
use tracing::info;
use uuid::Uuid;

use crate::activate::Activate;
use crate::order::{Order, OrderSide, OrderStatus, OrderType};
use crate::symbol::Symbol;
use crate::{CalculateCommand, CalculateResult, CalculateStats, Candle};

pub struct CalculateAgent<T: Activate + ?Sized> {
    index: usize,
    balance: f32,
    commission: f32,
    balance_assets: f32,
    min_balance: f32,
    portfolio_stock: HashMap<Symbol, f32>,
    portfolio_fiat: HashMap<Symbol, f32>,
    activate: Box<T>,
    queue_orders: HashMap<(Symbol, OrderSide), Vec<Order>>,
    executed_orders: Vec<Order>,
}

#[derive(Debug, Error)]
pub enum CalculateAgentError {
    #[error("Not enough balance: {0} < {1}")]
    NotEnoughBalance(f32, f32),
    #[error("Not enough asset balance {0}: {1} < {2}")]
    NotAssetEnoughBalance(Symbol, f32, f32),
    #[error("Unknown command")]
    UnknownCommand,
}

impl<T> CalculateAgent<T>
where
    T: Activate + ?Sized,
{
    pub fn new(index: usize, balance: f32, commission: f32, activate: Box<T>) -> CalculateAgent<T> {
        CalculateAgent {
            index,
            balance,
            activate,
            commission,
            min_balance: balance,
            balance_assets: 0.0,
            executed_orders: Default::default(),
            queue_orders: Default::default(),
            portfolio_stock: Default::default(),
            portfolio_fiat: Default::default(),
        }
    }

    pub fn activate(&self, candle: &Candle) -> CalculateCommand {
        self.activate.activate(candle, &self.get_stats(candle))
    }

    pub fn get_stats(&self, candle: &Candle) -> CalculateStats {
        let count = self.portfolio_stock.get(&candle.symbol).unwrap_or(&0.0);
        let orders = self
            .queue_orders
            .get(&(candle.symbol.clone(), OrderSide::Buy))
            .unwrap_or(&vec![])
            .len();

        CalculateStats {
            balance: self.balance,
            orders,
            count: *count,
            expected: 0f32,
            real: count * candle.open,
            assets_stock: self.portfolio_stock.clone(),
            assets_fiat: self.portfolio_fiat.clone(),
        }
    }

    pub fn market_buy(
        &mut self,
        candle: &Candle,
        price: f32,
        amount: f32,
        id: Option<Uuid>,
    ) -> Result<Order, CalculateAgentError> {
        let order_sum = amount * price;

        if self.balance < order_sum {
            return Err(CalculateAgentError::NotEnoughBalance(self.balance, amount));
        }

        self.balance -= order_sum;
        self.balance -= order_sum * self.commission;

        self.portfolio_stock
            .entry(candle.symbol.clone())
            .and_modify(|v| *v += amount)
            .or_insert(amount);

        let order = Order {
            uid: Uuid::new_v4(),
            ts: candle.start_time,
            price,
            qty: amount,
            symbol: candle.symbol.clone(),
            id,
            commission: order_sum * self.commission,
            agent: self.index,
            status: OrderStatus::Close,
            side: OrderSide::Buy,
            order_type: OrderType::Market,
        };

        self.executed_orders.push(order.clone());

        Ok(order)
    }

    pub fn limit_buy(
        &mut self,
        candle: &Candle,
        price: f32,
        amount: f32,
        id: Option<Uuid>,
    ) -> Result<Order, CalculateAgentError> {
        let order_sum = amount * price;

        if self.balance < order_sum {
            return Err(CalculateAgentError::NotEnoughBalance(
                self.balance,
                order_sum,
            ));
        }

        self.balance -= order_sum;
        self.balance -= order_sum * self.commission;

        let order = Order {
            uid: Uuid::new_v4(),
            ts: candle.start_time,
            price,
            qty: amount,
            id,
            symbol: candle.symbol.clone(),
            commission: order_sum * self.commission,
            agent: self.index,
            status: OrderStatus::Open,
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
        };

        let queue = self
            .queue_orders
            .entry((order.symbol.clone(), order.side.clone()))
            .or_default();

        queue.push(order.clone());

        Ok(order)
    }

    pub fn market_sell(
        &mut self,
        candle: &Candle,
        price: f32,
        amount: f32,
        id: Option<Uuid>,
    ) -> Result<Order, CalculateAgentError> {
        let portfolio_amount = self.portfolio_stock.get(&candle.symbol).unwrap_or(&0.0);

        if amount > *portfolio_amount {
            return Err(CalculateAgentError::NotAssetEnoughBalance(
                candle.symbol.clone(),
                *portfolio_amount,
                amount,
            ));
        }

        let order_sum = amount * price;

        self.balance += order_sum;
        self.balance -= order_sum * 0.001;

        self.portfolio_stock
            .entry(candle.symbol.clone())
            .and_modify(|v| *v -= amount);

        let order = Order {
            uid: Uuid::new_v4(),
            ts: candle.start_time,
            price,
            symbol: candle.symbol.clone(),
            id,
            qty: amount,
            commission: order_sum * self.commission,
            agent: self.index,
            status: OrderStatus::Close,
            side: OrderSide::Buy,
            order_type: OrderType::Market,
        };

        self.executed_orders.push(order.clone());

        Ok(order)
    }

    pub fn limit_sell(
        &mut self,
        candle: &Candle,
        price: f32,
        amount: f32,
        id: Option<Uuid>,
    ) -> Result<Order, CalculateAgentError> {
        let portfolio_amount = self.portfolio_stock.get(&candle.symbol).unwrap_or(&0.0);

        if amount > *portfolio_amount {
            return Err(CalculateAgentError::NotAssetEnoughBalance(
                candle.symbol.clone(),
                *portfolio_amount,
                amount,
            ));
        }

        let order_sum = amount * price;

        info!("portfolio before: {:?}", self.portfolio_stock);

        self.portfolio_stock
            .entry(candle.symbol.clone())
            .and_modify(|v| *v -= amount);

        info!("portfolio after: {:?}", self.portfolio_stock);

        let order = Order {
            uid: Uuid::new_v4(),
            ts: candle.start_time,
            price,
            id,
            symbol: candle.symbol.clone(),
            qty: amount,
            commission: order_sum * 0.001,
            agent: self.index,
            status: OrderStatus::Open,
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
        };

        let queue = self
            .queue_orders
            .entry((order.symbol.clone(), order.side.clone()))
            .or_default();
        queue.push(order.clone());

        Ok(order)
    }

    pub fn buy_profit(
        &mut self,
        candle: &Candle,
        price: f32,
        amount: f32,
        gain: f32,
        id: Option<Uuid>,
    ) -> Result<Order, CalculateAgentError> {
        let buy_order = self.market_buy(candle, price, amount, id)?;
        self.limit_sell(candle, buy_order.price * gain, buy_order.qty, id)
    }

    pub fn perform_order(
        &mut self,
        command: CalculateCommand,
        candle: &Candle,
    ) -> Result<Option<Order>, CalculateAgentError> {
        match command {
            CalculateCommand::BuyMarket { stake } => self
                .market_buy(candle, candle.open, stake, Some(Uuid::new_v4()))
                .map(Some),
            CalculateCommand::SellMarket { stake } => self
                .market_sell(candle, candle.open, stake, Some(Uuid::new_v4()))
                .map(Some),
            CalculateCommand::BuyLimit { stake, price } => self
                .limit_buy(candle, price, stake, Some(Uuid::new_v4()))
                .map(Some),
            CalculateCommand::SellLimit { stake, price } => self
                .limit_sell(candle, price, stake, Some(Uuid::new_v4()))
                .map(Some),
            CalculateCommand::BuyProfit { stake, profit } => self
                .buy_profit(candle, candle.open, stake, profit, Some(Uuid::new_v4()))
                .map(Some),
            CalculateCommand::None | CalculateCommand::Unknown => Ok(None),
        }
    }

    pub fn perform_candle(&mut self, candle: &Candle) {
        if let Some(orders) = self
            .queue_orders
            .get_mut(&(candle.symbol.clone(), OrderSide::Buy))
        {
            let mut to_remove = vec![];
            for order in orders.iter_mut() {
                if order.price > candle.low {
                    order.status = OrderStatus::Close;
                    self.portfolio_stock
                        .entry(candle.symbol.clone())
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
            .get_mut(&(candle.symbol.clone(), OrderSide::Sell))
        {
            let mut to_remove = vec![];

            for order in orders.iter_mut() {
                if order.price < candle.high {
                    order.status = OrderStatus::Close;
                    to_remove.push(order.uid);

                    self.balance += order.price * order.qty;
                    self.balance -= (order.price * order.qty) * 0.001;

                    self.executed_orders.push(order.clone());
                }
            }

            orders.retain(|o| !to_remove.contains(&o.uid));
        }

        if let Some(order_sum) = self.portfolio_stock.get(&candle.symbol) {
            self.portfolio_fiat
                .insert(candle.symbol.clone(), order_sum * candle.close);
        }

        info!(
            "perform_candle Agent {} portfolio: {:?}, fiat: {:?}",
            self.index, self.portfolio_stock, self.portfolio_fiat
        );
    }

    pub fn get_result(&self) -> CalculateResult {
        info!(
            "Agent {} result: {:?} queue: {:?}",
            self.index, self.balance, self.queue_orders
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
        }
    }

    pub fn on_end(&mut self) {
        self.activate.on_end(self.get_result())
    }

    pub fn on_end_round(&mut self) {
        self.min_balance = self.min_balance.min(self.balance);
        self.balance_assets = self.portfolio_fiat.values().sum();
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Activate, CalculateAgent, CalculateCommand, CalculateResult, CalculateStats, Candle,
    };

    #[test]
    fn test_calculate_agent_market() {
        // tracing_subscriber::fmt::init();

        struct CalculateIterActivate {}

        impl Activate for CalculateIterActivate {
            fn activate(&self, _candle: &Candle, _stats: &CalculateStats) -> CalculateCommand {
                CalculateCommand::None
            }

            fn on_end(&mut self, result: CalculateResult) {}
        }

        let mut agent = CalculateAgent::new(0, 1000.0, 0.0001, Box::new(CalculateIterActivate {}));

        let candle_1 = Candle {
            symbol: "BTC".to_string(),
            start_time: 0,
            open: 100.0,
            high: 120.0,
            low: 90.0,
            close: 110.0,
            volume: 100.0,
            end_time: 0,
            trades: 0.0,
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
            volume: 100.0,
            end_time: 0,
            trades: 0.0,
        };

        let result = agent.perform_order(CalculateCommand::SellMarket { stake: 5.0 }, &candle_2);

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_2);

        agent.on_end_round();

        let results = agent.get_result();

        println!("{:?}", agent.get_result());

        assert_eq!(results.balance, 1099.35);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 0);
        assert_eq!(results.executed_orders, 2);
    }

    #[test]
    fn test_calculate_agent_limit() {
        struct CalculateIterActivate {}

        impl Activate for CalculateIterActivate {
            fn activate(&self, _candle: &Candle, _stats: &CalculateStats) -> CalculateCommand {
                CalculateCommand::None
            }

            fn on_end(&mut self, result: CalculateResult) {}
        }

        let mut agent = CalculateAgent::new(0, 1000.0, 0.0001, Box::new(CalculateIterActivate {}));

        let candle_1 = Candle {
            symbol: "BTC".to_string(),
            start_time: 0,
            open: 100.0,
            high: 120.0,
            low: 90.0,
            close: 110.0,
            volume: 100.0,
            end_time: 0,
            trades: 0.0,
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
            volume: 100.0,
            end_time: 0,
            trades: 0.0,
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
            volume: 100.0,
            end_time: 0,
            trades: 0.0,
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
            volume: 100.0,
            end_time: 0,
            trades: 0.0,
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
