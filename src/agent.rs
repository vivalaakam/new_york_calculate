use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::activate::Activate;
use crate::candle::CandleTrait;
use crate::order::{Order, OrderSide, OrderStatus, OrderType};
use crate::symbol::Symbol;
use crate::{CalculateCommand, CalculateResult, CalculateStats};
use thiserror::Error;
use tracing::{debug, instrument};
use uuid::Uuid;

pub struct CalculateAgent<T: Activate<C> + ?Sized, C: CandleTrait> {
    balance: f32,
    commission: f32,
    balance_assets: f32,
    min_balance: f32,
    portfolio_stock: HashMap<Symbol, f32>,
    portfolio_fiat: HashMap<Symbol, f32>,
    activate: Box<T>,
    queue_orders: HashMap<Symbol, Vec<Order>>,
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
    C: CandleTrait + Debug,
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

    pub fn activate(&self, candles: &[C]) -> Vec<CalculateCommand> {
        self.activate
            .activate(candles, &self.get_result(), &self.queue_orders)
    }

    pub fn get_stats(&self, candle: &C) -> CalculateStats {
        let count = self
            .portfolio_stock
            .get(&candle.get_symbol())
            .unwrap_or(&0.0);
        let orders = self
            .queue_orders
            .get(&candle.get_symbol())
            .unwrap_or(&vec![])
            .len();

        CalculateStats {
            balance: self.balance,
            orders,
            count: *count,
            expected: 0f32,
            real: count * candle.get_open(),
            assets_stock: &self.portfolio_stock,
            assets_fiat: &self.portfolio_fiat,
        }
    }

    #[instrument(level = "debug", skip(self))]
    pub fn buy_order(
        &mut self,
        candle: &C,
        price: f32,
        qty: f32,
        order_type: OrderType,
        expiration: Option<u64>,
        id: Option<Uuid>,
    ) -> Result<Order, CalculateAgentError> {
        let order_sum = qty * price;

        if self.balance < order_sum {
            return Err(CalculateAgentError::NotEnoughBalance(self.balance, qty));
        }

        let mut order = Order {
            created_at: candle.get_start_time(),
            finished_at: 0,
            price,
            qty,
            symbol: candle.get_symbol(),
            id: id.unwrap_or(Uuid::new_v4()),
            commission: order_sum * self.commission,
            status: OrderStatus::Open,
            side: OrderSide::Buy,
            order_type,
            expiration,
        };

        self.balance -= order_sum;

        match order_type {
            OrderType::Market => {
                self.portfolio_stock
                    .entry(candle.get_symbol())
                    .and_modify(|v| *v += qty)
                    .or_insert(qty);

                self.balance -= order.commission;

                debug!(
                    balance = self.balance,
                    order = ?order,
                    portfolio_stock = ?self.portfolio_stock.get(&candle.get_symbol()),
                    "perform_market_buy done"
                );

                order.finished_at = candle.get_start_time();
                order.status = OrderStatus::Close;

                self.executed_orders.push(order.clone());
            }
            OrderType::Limit => {
                self.queue_orders
                    .entry(order.symbol.clone())
                    .or_default()
                    .push(order.clone());
            }
        }

        Ok(order)
    }

    #[instrument(level = "debug", skip(self))]
    pub fn sell_order(
        &mut self,
        candle: &C,
        price: f32,
        qty: f32,
        order_type: OrderType,
        expiration: Option<u64>,
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

        self.portfolio_stock
            .entry(candle.get_symbol())
            .and_modify(|v| *v -= qty);

        let order_sum = qty * price;

        let mut order = Order {
            id: id.unwrap_or(Uuid::new_v4()),
            created_at: candle.get_start_time(),
            finished_at: 0,
            symbol: candle.get_symbol(),
            price,
            qty,
            commission: order_sum * self.commission,
            status: OrderStatus::Open,
            side: OrderSide::Sell,
            order_type,
            expiration,
        };

        match order_type {
            OrderType::Market => {
                self.balance += order_sum;
                self.balance -= order.commission;

                order.finished_at = candle.get_start_time();
                order.status = OrderStatus::Close;

                self.executed_orders.push(order.clone());
            }
            OrderType::Limit => {
                self.queue_orders
                    .entry(order.symbol.clone())
                    .or_default()
                    .push(order.clone());
            }
        }

        Ok(order)
    }

    #[instrument(level = "debug", skip(self))]
    pub fn perform_order(
        &mut self,
        command: CalculateCommand,
        candle: &C,
    ) -> Result<Option<Order>, CalculateAgentError> {
        match command {
            CalculateCommand::BuyMarket { stake, .. } => self
                .buy_order(
                    candle,
                    candle.get_open(),
                    stake,
                    OrderType::Market,
                    None,
                    Some(Uuid::new_v4()),
                )
                .map(Some),
            CalculateCommand::SellMarket { stake, .. } => self
                .sell_order(
                    candle,
                    candle.get_open(),
                    stake,
                    OrderType::Market,
                    None,
                    Some(Uuid::new_v4()),
                )
                .map(Some),
            CalculateCommand::BuyLimit {
                stake,
                price,
                expiration,
                ..
            } => self
                .buy_order(
                    candle,
                    price,
                    stake,
                    OrderType::Limit,
                    expiration,
                    Some(Uuid::new_v4()),
                )
                .map(Some),
            CalculateCommand::SellLimit {
                stake,
                price,
                expiration,
                ..
            } => self
                .sell_order(
                    candle,
                    price,
                    stake,
                    OrderType::Limit,
                    expiration,
                    Some(Uuid::new_v4()),
                )
                .map(Some),
            CalculateCommand::None | CalculateCommand::Unknown => Ok(None),
            CalculateCommand::CancelLimit { symbol, id } => {
                self.perform_cancel_order(symbol, id);
                Ok(None)
            }
        }
    }

    #[instrument(level = "debug", skip(self))]
    pub fn perform_candle(&mut self, candle: &C) {
        let Some(orders) = self.queue_orders.get_mut(&candle.get_symbol()) else {
            if let Some(order_sum) = self.portfolio_stock.get(&candle.get_symbol()) {
                self.portfolio_fiat
                    .insert(candle.get_symbol(), order_sum * candle.get_close());
            }

            debug!("exit perform_candle");
            return;
        };

        let mut executed = vec![];

        for order in orders.into_iter() {
            match order.side {
                OrderSide::Buy => {
                    if order.price > candle.get_low() {
                        self.portfolio_stock
                            .entry(candle.get_symbol())
                            .and_modify(|v| *v += order.qty)
                            .or_insert(order.qty);

                        self.balance -= order.commission;

                        debug!(balance = self.balance,  order = ?order, "perform_buy_order done");

                        let mut order = order.clone();
                        order.status = OrderStatus::Close;
                        order.finished_at = candle.get_start_time();

                        executed.push(order);
                    } else if let Some(expiration) = order.expiration {
                        if order.created_at + expiration < candle.get_start_time() {
                            self.balance += order.price * order.qty;

                            debug!(balance = self.balance,  order = ?order, "perform_buy_order cancelled");

                            let mut order = order.clone();
                            order.status = OrderStatus::Cancel;
                            order.finished_at = candle.get_start_time();
                            executed.push(order.clone());
                        }
                    }
                }
                OrderSide::Sell => {
                    if order.price < candle.get_high() {
                        self.balance += order.price * order.qty;
                        self.balance -= order.commission;

                        debug!(balance = self.balance,  order = ?order, "perform_sell_order done");

                        let mut order = order.clone();
                        order.status = OrderStatus::Close;
                        order.finished_at = candle.get_start_time();

                        executed.push(order.clone());
                    } else if let Some(expiration) = order.expiration {
                        if order.created_at + expiration < candle.get_start_time() {
                            self.portfolio_stock
                                .entry(order.symbol.clone())
                                .and_modify(|v| *v += order.qty);

                            debug!(balance = self.balance,  order = ?order, "perform_sell_order cancelled");

                            let mut order = order.clone();
                            order.status = OrderStatus::Cancel;
                            order.finished_at = candle.get_start_time();
                            executed.push(order.clone());
                        }
                    }
                }
            }
        }

        let executed_ids: Vec<Uuid> = executed.iter().map(|o| o.id).collect();

        orders.retain(|o| !executed_ids.contains(&o.id));

        for order in executed.into_iter() {
            self.executed_orders.push(order);
        }

        if let Some(order_sum) = self.portfolio_stock.get(&candle.get_symbol()) {
            self.portfolio_fiat
                .insert(candle.get_symbol(), order_sum * candle.get_close());
        }

        debug!(
            symbol = candle.get_symbol(),
            portfolio_stock = ?self.portfolio_stock.get(&candle.get_symbol()),
            portfolio_fiat = ?self.portfolio_fiat.get(&candle.get_symbol()),
            "perform_candle done"
        );
    }

    #[instrument(level = "debug", skip(self))]
    fn perform_cancel_order(&mut self, symbol: Symbol, id: Uuid) {
        if let Some(orders) = self.queue_orders.get_mut(&symbol.clone()) {
            if let Some(order) = orders.iter().find(|o| o.id == id) {
                let mut order = order.clone();

                match order.side {
                    OrderSide::Buy => {
                        self.balance += order.price * order.qty;
                    }
                    OrderSide::Sell => {
                        self.portfolio_stock
                            .entry(symbol)
                            .and_modify(|v| *v += order.qty);
                    }
                }

                order.status = OrderStatus::Cancel;

                self.executed_orders.push(order.clone());

                orders.retain(|o| o.id != id);
                return;
            } else {
                debug!(?orders, "Order sell not found");
            }
        };
    }

    pub fn get_result(&self) -> CalculateResult {
        debug!(
            balance = self.balance,
            queue = self
                .queue_orders
                .iter()
                .fold(0, |acc, (_, v)| acc + v.len()),
            "Agent get result"
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

    pub fn on_end_round(&mut self, ts: u64, candles: &[C]) {
        self.min_balance = self.min_balance.min(self.balance);
        self.balance_assets = self.portfolio_fiat.values().sum();
        self.activate
            .on_end_round(ts, self.get_result(), candles, &self.executed_orders);
    }
}

#[cfg(test)]
mod tests {
    use crate::order::Order;
    use crate::test_utils::{init_tracing, Candle};
    use crate::{Activate, CalculateAgent, CalculateCommand, CalculateResult, Symbol};
    use std::collections::HashMap;
    use tracing::info;

    struct CalculateIterActivate {}

    impl Activate<Candle> for CalculateIterActivate {
        fn activate(
            &self,
            _candles: &[Candle],
            _stats: &CalculateResult,
            _active: &HashMap<Symbol, Vec<Order>>,
        ) -> Vec<CalculateCommand> {
            vec![CalculateCommand::None]
        }
    }

    #[test]
    fn test_calculate_agent_market() {
        init_tracing();

        let mut agent = CalculateAgent::new(1000.0, 0.0001, Box::new(CalculateIterActivate {}));

        let symbol = "BTC".to_string();

        let candle_1 = Candle {
            symbol: symbol.clone(),
            start_time: 0,
            open: 100.0,
            high: 120.0,
            low: 90.0,
            close: 110.0,
        };

        let result = agent.perform_order(
            CalculateCommand::BuyMarket {
                symbol: symbol.clone(),
                stake: 5.0,
            },
            &candle_1,
        );

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_1);

        agent.on_end_round(0, &vec![candle_1]);

        let results = agent.get_result();

        info!(results = ?results, "candle_1");

        assert_eq!(results.balance, 499.95);
        assert_eq!(results.balance_assets, 550.0);
        assert_eq!(results.opened_orders, 0);
        assert_eq!(results.executed_orders, 1);

        let candle_2 = Candle {
            symbol: symbol.clone(),
            start_time: 1,
            open: 120.0,
            high: 130.0,
            low: 90.0,
            close: 110.0,
        };

        let result = agent.perform_order(
            CalculateCommand::SellMarket {
                symbol: symbol.clone(),
                stake: 5.0,
            },
            &candle_2,
        );

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_2);

        agent.on_end_round(1, &vec![candle_2]);

        let results = agent.get_result();

        info!(results = ?results, "candle_2");

        assert_eq!(results.balance, 1099.8899);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 0);
        assert_eq!(results.executed_orders, 2);
    }

    #[test]
    fn test_calculate_agent_limit() {
        init_tracing();
        let mut agent = CalculateAgent::new(1000.0, 0.0001, Box::new(CalculateIterActivate {}));

        let symbol = "BTC".to_string();

        let candle_1 = Candle {
            symbol: symbol.clone(),
            start_time: 0,
            open: 100.0,
            high: 120.0,
            low: 90.0,
            close: 110.0,
        };

        let result = agent.perform_order(
            CalculateCommand::BuyLimit {
                symbol: symbol.clone(),
                price: 85.0,
                stake: 5.0,
                expiration: None,
            },
            &candle_1,
        );

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_1);

        agent.on_end_round(candle_1.start_time, &vec![candle_1]);

        let results = agent.get_result();

        info!(result = ?agent.get_result(), "candle_1");

        assert_eq!(results.balance, 575.0);
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

        agent.on_end_round(candle_2.start_time, &vec![candle_2]);

        let results = agent.get_result();

        info!(result = ?agent.get_result(), "candle_2" );

        assert_eq!(results.balance, 574.9575);
        assert_eq!(results.balance_assets, 550.0);
        assert_eq!(results.opened_orders, 0);
        assert_eq!(results.executed_orders, 1);

        let candle_3 = Candle {
            symbol: symbol.clone(),
            start_time: 3,
            open: 120.0,
            high: 130.0,
            low: 90.0,
            close: 110.0,
        };

        let result = agent.perform_order(
            CalculateCommand::SellLimit {
                symbol: symbol.clone(),
                stake: 5.0,
                price: 135.0,
                expiration: None,
            },
            &candle_3,
        );

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_3);

        agent.on_end_round(candle_3.start_time, &vec![candle_3]);

        let results = agent.get_result();

        info!(result = ?agent.get_result(), "candle_3");

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

        agent.on_end_round(candle_4.start_time, &vec![candle_4]);

        let results = agent.get_result();

        info!(result = ?agent.get_result(), "candle_4");

        assert_eq!(results.balance, 1249.89);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 0);
        assert_eq!(results.executed_orders, 2);
    }

    #[test]
    fn test_calculate_agent_buy_expiration() {
        init_tracing();
        let mut agent = CalculateAgent::new(1000.0, 0.0001, Box::new(CalculateIterActivate {}));

        let symbol = "BTC".to_string();

        let candle_1 = Candle {
            symbol: symbol.clone(),
            start_time: 0,
            open: 100.0,
            high: 120.0,
            low: 90.0,
            close: 110.0,
        };

        let result = agent.perform_order(
            CalculateCommand::BuyLimit {
                symbol: symbol.clone(),
                price: 85.0,
                stake: 5.0,
                expiration: Some(1),
            },
            &candle_1,
        );

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_1);

        agent.on_end_round(candle_1.start_time, &vec![candle_1]);

        let results = agent.get_result();

        info!(result = ?agent.get_result(), "candle_1");

        assert_eq!(results.balance, 575.0);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 1);
        assert_eq!(results.executed_orders, 0);

        let candle_2 = Candle {
            symbol: "BTC".to_string(),
            start_time: 1,
            open: 120.0,
            high: 130.0,
            low: 90.0,
            close: 110.0,
        };

        agent.perform_candle(&candle_2);

        agent.on_end_round(candle_2.start_time, &vec![candle_2]);

        let results = agent.get_result();

        info!(result = ?agent.get_result(), "candle_2" );

        assert_eq!(results.balance, 575.0);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 1);
        assert_eq!(results.executed_orders, 0);

        let candle_3 = Candle {
            symbol: symbol.clone(),
            start_time: 3,
            open: 120.0,
            high: 130.0,
            low: 90.0,
            close: 110.0,
        };

        agent.perform_candle(&candle_3);

        agent.on_end_round(candle_3.start_time, &vec![candle_3]);

        let results = agent.get_result();

        info!(result = ?agent.get_result(), "candle_3" );

        assert_eq!(results.balance, 1000.0);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 0);
        assert_eq!(results.executed_orders, 1);
    }

    #[test]
    fn test_calculate_agent_sell_expiration() {
        init_tracing();
        let mut agent = CalculateAgent::new(1000.0, 0.0001, Box::new(CalculateIterActivate {}));

        let symbol = "BTC".to_string();

        let candle_1 = Candle {
            symbol: symbol.clone(),
            start_time: 1,
            open: 100.0,
            high: 120.0,
            low: 90.0,
            close: 110.0,
        };

        let result = agent.perform_order(
            CalculateCommand::BuyMarket {
                symbol: symbol.clone(),
                stake: 5.0,
            },
            &candle_1,
        );

        assert!(matches!(result, Ok(Some(_))));

        let result = agent.perform_order(
            CalculateCommand::SellLimit {
                symbol: symbol.clone(),
                stake: 5.0,
                price: 150.0,
                expiration: Some(1),
            },
            &candle_1,
        );

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_1);

        agent.on_end_round(candle_1.start_time, &vec![candle_1]);

        let results = agent.get_result();

        info!(result = ?results, "candle_1");

        assert_eq!(results.balance, 499.95);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 1);
        assert_eq!(results.executed_orders, 1);

        let candle_2 = Candle {
            symbol: symbol.clone(),
            start_time: 2,
            open: 120.0,
            high: 130.0,
            low: 90.0,
            close: 110.0,
        };

        agent.perform_candle(&candle_2);

        agent.on_end_round(candle_2.start_time, &vec![candle_2]);

        let results = agent.get_result();

        info!(result = ?results, "candle_2");

        assert_eq!(results.balance, 499.95);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 1);
        assert_eq!(results.executed_orders, 1);

        let candle_3 = Candle {
            symbol: "BTC".to_string(),
            start_time: 3,
            open: 120.0,
            high: 130.0,
            low: 90.0,
            close: 110.0,
        };

        agent.perform_candle(&candle_3);

        agent.on_end_round(candle_3.start_time, &vec![candle_3]);

        let results = agent.get_result();

        info!(result = ?agent.get_result(), "candle_3");

        assert_eq!(results.balance, 499.95);
        assert_eq!(results.balance_assets, 550.0);
        assert_eq!(results.opened_orders, 0);
        assert_eq!(results.executed_orders, 2);
    }

    #[test]
    fn test_calculate_agent_buy_cancel() {
        init_tracing();
        let mut agent = CalculateAgent::new(1000.0, 0.0001, Box::new(CalculateIterActivate {}));

        let symbol = "BTC".to_string();

        let candle_1 = Candle {
            symbol: symbol.clone(),
            start_time: 0,
            open: 100.0,
            high: 120.0,
            low: 90.0,
            close: 110.0,
        };

        let result = agent.perform_order(
            CalculateCommand::BuyLimit {
                symbol: symbol.clone(),
                price: 85.0,
                stake: 5.0,
                expiration: None,
            },
            &candle_1,
        );

        assert!(matches!(result, Ok(Some(_))));

        let Ok(Some(Order { id, .. })) = result else {
            panic!("Order not found");
        };

        agent.perform_candle(&candle_1);

        agent.on_end_round(candle_1.start_time, &vec![candle_1]);

        let results = agent.get_result();

        info!(result = ?agent.get_result(), "candle_1");

        assert_eq!(results.balance, 575.0);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 1);
        assert_eq!(results.executed_orders, 0);

        let candle_2 = Candle {
            symbol: "BTC".to_string(),
            start_time: 1,
            open: 120.0,
            high: 130.0,
            low: 90.0,
            close: 110.0,
        };

        agent.perform_candle(&candle_2);

        agent.on_end_round(candle_2.start_time, &vec![candle_2]);

        let results = agent.get_result();

        info!(result = ?agent.get_result(), "candle_2" );

        assert_eq!(results.balance, 575.0);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 1);
        assert_eq!(results.executed_orders, 0);

        let candle_3 = Candle {
            symbol: symbol.clone(),
            start_time: 3,
            open: 120.0,
            high: 130.0,
            low: 90.0,
            close: 110.0,
        };

        let result = agent.perform_order(
            CalculateCommand::CancelLimit {
                symbol: symbol.clone(),
                id,
            },
            &candle_3,
        );

        assert!(matches!(result, Ok(None)));

        agent.perform_candle(&candle_3);

        agent.on_end_round(candle_3.start_time, &vec![candle_3]);

        let results = agent.get_result();

        info!(result = ?agent.get_result(), "candle_3" );

        assert_eq!(results.balance, 1000.0);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 0);
        assert_eq!(results.executed_orders, 1);
    }

    #[test]
    fn test_calculate_agent_sell_cancel() {
        init_tracing();
        let mut agent = CalculateAgent::new(1000.0, 0.0001, Box::new(CalculateIterActivate {}));

        let symbol = "BTC".to_string();

        let candle_1 = Candle {
            symbol: symbol.clone(),
            start_time: 1,
            open: 100.0,
            high: 120.0,
            low: 90.0,
            close: 110.0,
        };

        let result = agent.perform_order(
            CalculateCommand::BuyMarket {
                symbol: symbol.clone(),
                stake: 5.0,
            },
            &candle_1,
        );

        assert!(matches!(result, Ok(Some(_))));

        let result = agent.perform_order(
            CalculateCommand::SellLimit {
                symbol: symbol.clone(),
                stake: 5.0,
                price: 150.0,
                expiration: None,
            },
            &candle_1,
        );

        assert!(matches!(result, Ok(Some(_))));

        let Ok(Some(Order { id, .. })) = result else {
            panic!("Order not found");
        };

        agent.perform_candle(&candle_1);

        agent.on_end_round(candle_1.start_time, &vec![candle_1]);

        let results = agent.get_result();

        info!(result = ?results, "candle_1");

        assert_eq!(results.balance, 499.95);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 1);
        assert_eq!(results.executed_orders, 1);

        let candle_2 = Candle {
            symbol: symbol.clone(),
            start_time: 2,
            open: 120.0,
            high: 130.0,
            low: 90.0,
            close: 110.0,
        };

        agent.perform_candle(&candle_2);

        agent.on_end_round(candle_2.start_time, &vec![candle_2]);

        let results = agent.get_result();

        info!(result = ?results, "candle_2");

        assert_eq!(results.balance, 499.95);
        assert_eq!(results.balance_assets, 0.0);
        assert_eq!(results.opened_orders, 1);
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
            CalculateCommand::CancelLimit {
                symbol: symbol.clone(),
                id,
            },
            &candle_3,
        );

        info!(result = ?result, "perform_order");

        assert!(matches!(result, Ok(None)));

        agent.perform_candle(&candle_3);

        agent.on_end_round(candle_3.start_time, &vec![candle_3]);

        let results = agent.get_result();

        info!(result = ?results, "candle_3");

        assert_eq!(results.balance, 499.95);
        assert_eq!(results.balance_assets, 550.0);
        assert_eq!(results.opened_orders, 0);
        assert_eq!(results.executed_orders, 2);
    }
}
