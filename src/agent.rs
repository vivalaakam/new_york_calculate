use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::activate::Activate;
use crate::candle::CandleTrait;
use crate::order::{Order, OrderSide, OrderStatus, OrderType};
use crate::types::{OrderId, Symbol, TimeStamp, UserId};
use crate::{CalculateCommand, CalculateResult, CalculateResultRef, CalculateStats};
use errors::CalculateAgentError;
use tracing::{debug, instrument};
use uuid::Uuid;

mod errors;

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
    pub fn activate(&self, candles: &[C], prices: &HashMap<&str, f32>) -> Vec<CalculateCommand> {
        let result = self.get_result_ref();
        self.activate
            .activate(candles, prices, result, &self.queue_orders)
    }

    /// Get the stats of the agent
    #[instrument(level = "debug", skip(self))]
    pub fn get_stats(&self, candle: &C) -> CalculateStats<'_> {
        let count = self
            .portfolio_available
            .get(candle.get_symbol())
            .unwrap_or(&0.0);
        let orders = self
            .queue_orders
            .get(candle.get_symbol())
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
    #[allow(clippy::too_many_arguments)]
    #[instrument(level = "debug", skip(self))]
    pub fn buy_order(
        &mut self,
        candle: &C,
        price: f32,
        qty: f32,
        order_type: OrderType,
        expiration: Option<TimeStamp>,
        id: Option<OrderId>,
        user_id: Option<UserId>,
    ) -> Result<Order, CalculateAgentError> {
        let order_sum = qty * price;

        if self.balance < order_sum {
            return Err(CalculateAgentError::InsufficientBalance {
                available: self.balance,
                required: order_sum,
            });
        }

        let mut order = Order {
            created_at: candle.get_start_time(),
            finished_at: 0,
            price,
            qty,
            symbol: candle.get_symbol().to_owned(),
            id: id.unwrap_or(Uuid::new_v4()),
            commission: order_sum * self.commission,
            status: OrderStatus::Open,
            side: OrderSide::Buy,
            order_type,
            expiration,
            user_id,
        };

        self.balance -= order_sum;

        self.activate.on_order(candle.get_start_time(), &order);

        match order_type {
            OrderType::Market => {
                self.execute_buy_order(&mut order, candle);
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

    /// Sell an order
    #[allow(clippy::too_many_arguments)]
    #[instrument(level = "debug", skip(self))]
    pub fn sell_order(
        &mut self,
        candle: &C,
        price: f32,
        qty: f32,
        order_type: OrderType,
        expiration: Option<TimeStamp>,
        id: Option<OrderId>,
        user_id: Option<UserId>,
    ) -> Result<Order, CalculateAgentError> {
        let portfolio_amount = self
            .portfolio_available
            .get(candle.get_symbol())
            .unwrap_or(&0.0);

        if qty > *portfolio_amount {
            return Err(CalculateAgentError::InsufficientAssetBalance {
                symbol: candle.get_symbol().to_owned(),
                available: *portfolio_amount,
                required: qty,
            });
        }

        self.portfolio_available
            .entry(candle.get_symbol().to_owned())
            .and_modify(|v| *v -= qty)
            .or_insert(0.0);

        self.portfolio_frozen
            .entry(candle.get_symbol().to_owned())
            .and_modify(|v| *v += qty)
            .or_insert(qty);

        let order_sum = qty * price;

        let mut order = Order {
            id: id.unwrap_or(Uuid::new_v4()),
            created_at: candle.get_start_time(),
            finished_at: 0,
            symbol: candle.get_symbol().to_owned(),
            price,
            qty,
            commission: order_sum * self.commission,
            status: OrderStatus::Open,
            side: OrderSide::Sell,
            order_type,
            expiration,
            user_id,
        };

        self.activate.on_order(candle.get_start_time(), &order);

        match order_type {
            OrderType::Market => {
                self.execute_sell_order(&mut order, candle);
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

    /// Perform an order
    #[instrument(level = "debug", skip(self))]
    pub fn perform_order(
        &mut self,
        command: CalculateCommand,
        candle: &C,
    ) -> Result<Option<Order>, CalculateAgentError> {
        match command {
            CalculateCommand::BuyMarket { stake, user_id, .. } => self
                .buy_order(
                    candle,
                    candle.get_open(),
                    stake,
                    OrderType::Market,
                    None,
                    None,
                    user_id,
                )
                .map(Some),
            CalculateCommand::SellMarket { stake, user_id, .. } => self
                .sell_order(
                    candle,
                    candle.get_open(),
                    stake,
                    OrderType::Market,
                    None,
                    None,
                    user_id,
                )
                .map(Some),
            CalculateCommand::BuyLimit {
                stake,
                price,
                expiration,
                user_id,
                ..
            } => self
                .buy_order(
                    candle,
                    price,
                    stake,
                    OrderType::Limit,
                    expiration,
                    None,
                    user_id,
                )
                .map(Some),
            CalculateCommand::SellLimit {
                stake,
                price,
                expiration,
                user_id,
                ..
            } => self
                .sell_order(
                    candle,
                    price,
                    stake,
                    OrderType::Limit,
                    expiration,
                    None,
                    user_id,
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
        let Some(orders) = self.queue_orders.get_mut(candle.get_symbol()) else {
            return;
        };

        // Pass 1: identify which orders to execute (immutable borrow of orders + candle).
        let to_execute: Vec<usize> = orders
            .iter()
            .enumerate()
            .filter(|(_, order)| {
                let buy_hit = order.side == OrderSide::Buy && order.price > candle.get_low();
                let sell_hit = order.side == OrderSide::Sell && order.price < candle.get_high();
                let expired = order
                    .expiration
                    .is_some_and(|exp| order.created_at + exp < candle.get_start_time());
                buy_hit || sell_hit || expired
            })
            .map(|(i, _)| i)
            .collect();

        if to_execute.is_empty() {
            return;
        }

        // Pass 2: extract executed orders via swap_remove (preserves order in remaining).
        let mut executed: Vec<Order> = Vec::with_capacity(to_execute.len());
        for &i in to_execute.iter().rev() {
            executed.push(orders.swap_remove(i));
        }

        // Pass 3: process extracted orders with full &mut self access.
        for order in &mut executed {
            let buy_hit = order.side == OrderSide::Buy && order.price > candle.get_low();
            let sell_hit = order.side == OrderSide::Sell && order.price < candle.get_high();

            if buy_hit {
                self.execute_buy_order(order, candle);
            } else if sell_hit {
                self.execute_sell_order(order, candle);
            } else {
                self.execute_cancel_order(order, candle);
            }
        }

        self.executed_orders.extend(executed);

        debug!(
            symbol = candle.get_symbol(),
            portfolio_available = ?self.portfolio_available.get(candle.get_symbol()),
            portfolio_frozen = ?self.portfolio_frozen.get(candle.get_symbol()),
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

        let Some(idx) = orders.iter().position(|o| o.id == id) else {
            debug!(symbol = symbol, id = ?id, "cancel order not found");
            return;
        };

        // Extract the order at idx, process it, then re-insert.
        let mut order = orders.swap_remove(idx);
        self.execute_cancel_order(&mut order, candle);
        self.executed_orders.push(order);
    }

    /// Get the result of the agent
    #[instrument(level = "debug", skip(self))]
    pub fn get_result(&self) -> CalculateResult {
        CalculateResult::from(self.get_result_ref())
    }

    /// Get a borrowed view of the agent's state — no cloning.
    pub fn get_result_ref(&self) -> CalculateResultRef<'_> {
        let opened_orders = self.queue_orders.values().map(|v| v.len()).sum();

        debug!(
            balance = self.balance,
            queue = opened_orders,
            "Agent get result"
        );

        CalculateResultRef {
            balance: self.balance,
            min_balance: self.min_balance,
            opened_orders,
            executed_orders: self.executed_orders.len(),
            assets_available: &self.portfolio_available,
            assets_frozen: &self.portfolio_frozen,
        }
    }

    /// Final action after all rounds finished
    #[instrument(level = "debug", skip(self))]
    pub fn on_end(&mut self) {
        let result = self.get_result();
        self.activate.on_end(result)
    }

    /// Action after a round finished
    #[instrument(level = "debug", skip(self))]
    pub fn on_end_round(&mut self, _ts: u64, _candles: &[C]) {
        self.min_balance = self.min_balance.min(self.balance);
    }

    /// Execute a buy order: update balance, portfolio, mark as closed
    #[instrument(level = "debug", skip(self, order, candle))]
    fn execute_buy_order(&mut self, order: &mut Order, candle: &C) {
        self.balance -= order.commission;

        self.portfolio_available
            .entry(candle.get_symbol().to_owned())
            .and_modify(|v| *v += order.qty)
            .or_insert(order.qty);

        order.status = OrderStatus::Close;
        order.finished_at = candle.get_start_time();

        self.activate.on_order(candle.get_start_time(), order);

        debug!(balance = self.balance, order = ?order, "buy order execution completed");
    }

    /// Execute a sell order: update balance, portfolio, mark as closed
    #[instrument(level = "debug", skip(self, order, candle))]
    fn execute_sell_order(&mut self, order: &mut Order, candle: &C) {
        self.balance += order.price * order.qty;
        self.balance -= order.commission;

        self.portfolio_frozen
            .entry(candle.get_symbol().to_owned())
            .and_modify(|v| *v -= order.qty);

        order.status = OrderStatus::Close;
        order.finished_at = candle.get_start_time();

        self.activate.on_order(candle.get_start_time(), order);

        debug!(balance = self.balance, order = ?order, "sell order execution completed");
    }

    /// Cancel an order: revert balance/portfolio changes, mark as cancelled
    #[instrument(level = "debug", skip(self, order, candle))]
    fn execute_cancel_order(&mut self, order: &mut Order, candle: &C) {
        match order.side {
            OrderSide::Buy => {
                self.balance += order.price * order.qty;
            }
            OrderSide::Sell => {
                self.portfolio_available
                    .entry(candle.get_symbol().to_owned())
                    .and_modify(|v| *v += order.qty);

                self.portfolio_frozen
                    .entry(candle.get_symbol().to_owned())
                    .and_modify(|v| *v -= order.qty);
            }
        }

        order.status = OrderStatus::Cancel;
        order.finished_at = candle.get_start_time();

        self.activate.on_order(candle.get_start_time(), order);
    }
}

#[cfg(test)]
mod tests {
    use crate::order::Order;
    use crate::test_utils::{init_tracing, Candle};
    use crate::{
        assert_agent_state, buy_limit, buy_market, sell_limit, sell_market, Activate,
        CalculateAgent, CalculateCommand, CalculateResultRef, Symbol,
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
            _prices: &HashMap<&str, f32>,
            _stats: CalculateResultRef<'_>,
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

        let result =
            agent.perform_order(buy_market!(symbol, 5.0, user_id = "buy_market"), &candle_1);

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_1);

        agent.on_end_round(0, &[candle_1]);

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
            sell_market!(symbol, 5.0, user_id = "sell_market"),
            &candle_2,
        );

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_2);

        agent.on_end_round(1, &[candle_2]);

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
            buy_limit!(symbol, 5.0, 85.0, user_id = "buy_limit"),
            &candle_1,
        );

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_1);

        agent.on_end_round(candle_1.start_time, &[candle_1]);

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

        agent.on_end_round(candle_2.start_time, &[candle_2]);

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
            sell_limit!(symbol, 5.0, 135.0, user_id = "sell_limit"),
            &candle_3,
        );

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_3);

        agent.on_end_round(candle_3.start_time, &[candle_3]);

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

        agent.on_end_round(candle_4.start_time, &[candle_4]);

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
                user_id: None,
            },
            &candle_1,
        );

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_1);

        agent.on_end_round(candle_1.start_time, &[candle_1]);

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

        agent.on_end_round(candle_2.start_time, &[candle_2]);

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

        agent.on_end_round(candle_3.start_time, &[candle_3]);

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
                user_id: None,
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
                user_id: None,
            },
            &candle_1,
        );

        assert!(matches!(result, Ok(Some(_))));

        agent.perform_candle(&candle_1);

        agent.on_end_round(candle_1.start_time, &[candle_1]);

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

        agent.on_end_round(candle_2.start_time, &[candle_2]);

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

        agent.on_end_round(candle_3.start_time, &[candle_3]);

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
                user_id: None,
            },
            &candle_1,
        );

        assert!(matches!(result, Ok(Some(_))));

        let Ok(Some(Order { id, .. })) = result else {
            panic!("Order not found");
        };

        agent.perform_candle(&candle_1);

        agent.on_end_round(candle_1.start_time, &[candle_1]);

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

        agent.on_end_round(candle_2.start_time, &[candle_2]);

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

        agent.on_end_round(candle_3.start_time, &[candle_3]);

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
                user_id: None,
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
                user_id: None,
            },
            &candle_1,
        );

        assert!(matches!(result, Ok(Some(_))));

        let Ok(Some(Order { id, .. })) = result else {
            panic!("Order not found");
        };

        agent.perform_candle(&candle_1);

        agent.on_end_round(candle_1.start_time, &[candle_1]);

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

        agent.on_end_round(candle_2.start_time, &[candle_2]);

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

        agent.on_end_round(candle_3.start_time, &[candle_3]);

        let results = agent.get_result();

        info!(result = ?results, "candle_3");

        assert_agent_state!(results, 499.95, 0, 2, activate.orders, 4);
    }
}
