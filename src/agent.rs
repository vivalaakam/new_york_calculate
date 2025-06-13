use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::activate::Activate;
use crate::candle::CandleTrait;
use crate::order::{Order, OrderSide, OrderStatus, OrderType};
use crate::types::{OrderId, Symbol, TimeStamp};
use crate::{
    handle_buy_executed_order, handle_cancel_order, handle_sell_executed_order, CalculateCommand,
    CalculateResult, CalculateStats,
};
use errors::CalculateAgentError;
use tracing::{debug, instrument};
use uuid::Uuid;

mod errors;
mod macros;

pub struct CalculateAgent<T: Activate<C> + ?Sized, C: CandleTrait> {
    balance: f32,
    commission: f32,
    min_balance: f32,
    portfolio_available: HashMap<Symbol, f32>,
    portfolio_frozen: HashMap<Symbol, f32>,
    activate: Box<T>,
    queue_orders: HashMap<Symbol, Vec<Order>>,
    executed_orders: Vec<Order>,
    candle: PhantomData<C>,
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
            executed_orders: Default::default(),
            queue_orders: Default::default(),
            portfolio_available: Default::default(),
            portfolio_frozen: Default::default(),
            candle: PhantomData,
        }
    }

    /// Activate the agent
    #[instrument(level = "debug", skip(self))]
    pub fn activate(&self, candles: &[C], prices: &HashMap<Symbol, f32>) -> Vec<CalculateCommand> {
        self.activate
            .activate(candles, prices, &self.get_result(), &self.queue_orders)
    }

    /// Get the stats of the agent
    #[instrument(level = "debug", skip(self))]
    pub fn get_stats(&self, candle: &C) -> CalculateStats {
        let count = self
            .portfolio_available
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
            assets_available: &self.portfolio_available,
            assets_frozen: &self.portfolio_frozen,
        }
    }

    /// Buy an order
    #[instrument(level = "warn", skip(self))]
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
            return Err(CalculateAgentError::InsufficientBalance {
                available: self.balance,
                required: order_sum,
            });
        }

        let order = Order {
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

        self.activate.on_order(candle.get_start_time(), &order);

        match order_type {
            OrderType::Market => {
                let executed_order = handle_buy_executed_order!(self, order, candle);
                self.executed_orders.push(executed_order);
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

    /// Sell an order
    #[instrument(level = "debug", skip(self))]
    pub fn sell_order(
        &mut self,
        candle: &C,
        price: f32,
        qty: f32,
        order_type: OrderType,
        expiration: Option<TimeStamp>,
        id: Option<OrderId>,
    ) -> Result<Order, CalculateAgentError> {
        let portfolio_amount = self
            .portfolio_available
            .get(&candle.get_symbol())
            .unwrap_or(&0.0);

        if qty > *portfolio_amount {
            return Err(CalculateAgentError::InsufficientAssetBalance {
                symbol: candle.get_symbol(),
                available: *portfolio_amount,
                required: qty,
            });
        }

        self.portfolio_available
            .entry(candle.get_symbol())
            .and_modify(|v| *v -= qty)
            .or_insert(0.0);

        self.portfolio_frozen
            .entry(candle.get_symbol())
            .and_modify(|v| *v += qty)
            .or_insert(qty);

        let order_sum = qty * price;

        let order = Order {
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

        self.activate.on_order(candle.get_start_time(), &order);

        match order_type {
            OrderType::Market => {
                let executed_order = handle_sell_executed_order!(self, order, candle);
                self.executed_orders.push(executed_order);
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

    /// Perform an order
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
                self.cancel_order(symbol, id, candle);
                Ok(None)
            }
        }
    }

    /// Perform a candle
    #[instrument(level = "debug", skip(self))]
    pub fn perform_candle(&mut self, candle: &C) {
        if let Some(orders) = self.queue_orders.get_mut(&candle.get_symbol()) {
            let mut executed_ids = HashSet::new();

            for order in orders.iter_mut() {
                let mut executed_order = match order.side {
                    OrderSide::Buy => {
                        if order.price > candle.get_low() {
                            Some(handle_buy_executed_order!(self, order, candle))
                        } else {
                            None
                        }
                    }
                    OrderSide::Sell => {
                        if order.price < candle.get_high() {
                            Some(handle_sell_executed_order!(self, order, candle))
                        } else {
                            None
                        }
                    }
                };

                if executed_order.is_none() {
                    if let Some(expiration) = order.expiration {
                        if order.created_at + expiration < candle.get_start_time() {
                            executed_order = Some(handle_cancel_order!(self, order, candle));
                        }
                    }
                }

                if let Some(executed_order) = executed_order {
                    executed_ids.insert(executed_order.id);
                    self.executed_orders.push(executed_order);
                }
            }

            orders.retain(|o| !executed_ids.contains(&o.id));
        }

        debug!(
            symbol = candle.get_symbol(),
            portfolio_available = ?self.portfolio_available.get(&candle.get_symbol()),
            portfolio_frozen = ?self.portfolio_frozen.get(&candle.get_symbol()),
            "perform_candle done"
        );
    }

    /// Perform a cancel order
    #[instrument(level = "debug", skip(self))]
    fn cancel_order(&mut self, symbol: Symbol, id: OrderId, candle: &C) {
        let Some(orders) = self.queue_orders.get_mut(&symbol) else {
            debug!(symbol = symbol, "cancel order symbol not found");
            return;
        };

        let Some(order) = orders.iter().find(|o| o.id == id) else {
            debug!(symbol = symbol, id = ?id, "cancel order not found");
            return;
        };

        let executed_order = handle_cancel_order!(self, order, candle);

        self.executed_orders.push(executed_order);

        orders.retain(|o| o.id != id);
    }

    /// Get the result of the agent
    #[instrument(level = "debug", skip(self))]
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
            assets_available: self.portfolio_available.clone(),
            assets_frozen: self.portfolio_frozen.clone(),
        }
    }

    /// Final action after all rounds finished
    #[instrument(level = "debug", skip(self))]
    pub fn on_end(&mut self) {
        self.activate.on_end(self.get_result())
    }

    /// Action after a round finished
    #[instrument(level = "debug", skip(self))]
    pub fn on_end_round(&mut self, ts: u64, candles: &[C]) {
        self.min_balance = self.min_balance.min(self.balance);
    }
}

#[cfg(test)]
mod tests {
    use crate::order::Order;
    use crate::test_utils::{init_tracing, Candle};
    use crate::{
        assert_agent_state, Activate, CalculateAgent, CalculateCommand, CalculateResult, Symbol,
    };
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tracing::info;

    #[derive(Debug, Default)]
    struct CalculateIterActivate {
        orders: Mutex<Vec<Order>>,
    }

    impl Activate<Candle> for &CalculateIterActivate {
        fn activate(
            &self,
            _candles: &[Candle],
            _prices: &HashMap<Symbol, f32>,
            _stats: &CalculateResult,
            _active: &HashMap<Symbol, Vec<Order>>,
        ) -> Vec<CalculateCommand> {
            vec![CalculateCommand::None]
        }

        fn on_order(&mut self, _ts: u64, order: &Order) {
            self.orders.lock().unwrap().push(order.clone());
        }
    }

    #[test]
    fn test_calculate_agent_market() {
        init_tracing();

        let activate = CalculateIterActivate::default();

        let mut agent = CalculateAgent::new(1000.0, 0.0001, Box::new(&activate));

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

        assert_agent_state!(results, 499.95, 0, 1, activate.orders, 2);

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
        assert_agent_state!(results, 1099.8899, 0, 2, activate.orders, 4);
    }

    #[test]
    fn test_calculate_agent_limit() {
        init_tracing();
        let activate = CalculateIterActivate::default();

        let mut agent = CalculateAgent::new(1000.0, 0.0001, Box::new(&activate));

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

        assert_agent_state!(results, 575.0, 1, 0, activate.orders, 1);

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

        assert_agent_state!(results, 574.9575, 0, 1, activate.orders, 2);

        assert_eq!(
            results.assets_available,
            HashMap::from_iter(vec![(symbol.to_string(), 5.0)])
        );

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

        assert_agent_state!(results, 574.9575, 1, 1, activate.orders, 3);

        assert_eq!(
            results.assets_available,
            HashMap::from_iter(vec![(symbol.to_string(), 0.0)])
        );
        assert_eq!(
            results.assets_frozen,
            HashMap::from_iter(vec![(symbol.to_string(), 5.0)])
        );

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

        assert_agent_state!(results, 1249.89, 0, 2, activate.orders, 4);

        assert_eq!(
            results.assets_available,
            HashMap::from_iter(vec![(symbol.to_string(), 0.0)])
        );
        assert_eq!(
            results.assets_frozen,
            HashMap::from_iter(vec![(symbol.to_string(), 0.0)])
        );
    }

    #[test]
    fn test_calculate_agent_buy_expiration() {
        init_tracing();

        let activate = CalculateIterActivate::default();

        let mut agent = CalculateAgent::new(1000.0, 0.0001, Box::new(&activate));

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

        assert_agent_state!(results, 575.0, 1, 0, activate.orders, 1);

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

        assert_agent_state!(results, 575.0, 1, 0, activate.orders, 1);

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

        assert_agent_state!(results, 1000.0, 0, 1, activate.orders, 2);
    }

    #[test]
    fn test_calculate_agent_sell_expiration() {
        init_tracing();
        let activate = CalculateIterActivate::default();
        let mut agent = CalculateAgent::new(1000.0, 0.0001, Box::new(&activate));

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

        assert_agent_state!(results, 499.95, 1, 1, activate.orders, 3);

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

        assert_agent_state!(results, 499.95, 1, 1, activate.orders, 3);

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
        assert_eq!(results.opened_orders, 0);
        assert_eq!(results.executed_orders, 2);
    }

    #[test]
    fn test_calculate_agent_buy_cancel() {
        init_tracing();
        let activate = CalculateIterActivate::default();
        let mut agent = CalculateAgent::new(1000.0, 0.0001, Box::new(&activate));

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

        assert_agent_state!(results, 575.0, 1, 0, activate.orders, 1);

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

        assert_agent_state!(results, 1000.0, 0, 1, activate.orders, 2);
    }

    #[test]
    fn test_calculate_agent_sell_cancel() {
        init_tracing();
        let activate = CalculateIterActivate::default();
        let mut agent = CalculateAgent::new(1000.0, 0.0001, Box::new(&activate));

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

        assert_agent_state!(results, 499.95, 1, 1, activate.orders, 3);

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

        assert_agent_state!(results, 499.95, 1, 1, activate.orders, 3);

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

        assert_agent_state!(results, 499.95, 0, 2, activate.orders, 4);
    }
}
