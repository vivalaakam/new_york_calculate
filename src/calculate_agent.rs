use crate::calculate_activate::CalculateActivate;
use crate::calculate_stats::CalculateStats;
use crate::utils::{ceil_to_nearest, floor_to_nearest};
use crate::{CalculateCommand, Candle, Order};

pub struct CalculateAgent<T> {
    balance: f64,
    executed_orders: Vec<Order>,
    wallet: f64,
    min_balance: f64,
    orders: usize,
    count: f64,
    expected: f64,
    waiting: u64,
    successful: u64,
    activate: T,
}

impl<T> CalculateAgent<T>
where
    T: CalculateActivate,
{
    pub fn new(balance: f64, activate: T) -> CalculateAgent<T> {
        CalculateAgent {
            balance,
            activate,
            min_balance: balance,
            executed_orders: vec![],
            wallet: 0.0,
            orders: 0,
            count: 0.0,
            expected: 0.0,
            waiting: 0,
            successful: 0,
        }
    }

    pub fn activate(&self, candle: &Candle, position: usize) -> CalculateCommand {
        self.activate
            .activate(candle, position, &self.get_stats(candle))
    }

    pub fn get_stats(&self, candle: &Candle) -> CalculateStats {
        CalculateStats {
            balance: self.balance,
            orders: self.orders,
            count: self.count,
            expected: self.expected,
            real: self.count * candle.open,
        }
    }

    pub fn buy_profit_open(
        &mut self,
        candle: &Candle,
        stake: f64,
        gain: f64,
        step_lot: f64,
        step_price: f64,
    ) -> Option<Order> {
        if self.balance < stake {
            return None;
        }
        let curr_stake = floor_to_nearest(stake / candle.open, step_lot);
        let order_sum = curr_stake * candle.open;
        self.balance -= order_sum;
        self.balance -= order_sum * 0.001;

        let order = Order {
            start_time: candle.start_time,
            end_time: 0,
            buy_price: candle.open,
            sell_price: ceil_to_nearest(candle.open * gain, step_price),
            qty: curr_stake,
            commission: order_sum * 0.001,
            gain,
            stake,
            profit: 0.0,
        };

        self.orders += 1;
        self.count += order.qty;
        self.expected += order.sell_price * order.qty;

        Some(order)
    }

    pub fn buy_profit_close(&mut self, order: Order, candle: &Candle, profit: f64) {
        let mut order = order.clone();
        let order_sum = order.sell_price * order.qty;

        self.balance += order_sum;
        self.balance -= order_sum * 0.001;
        order.commission += order_sum * 0.001;

        order.end_time = candle.end_time;

        let profit_size =
            ((order.sell_price - order.buy_price) * order.qty - order.commission) * profit;

        order.profit = profit_size;

        self.balance -= profit_size;
        self.wallet += profit_size;

        self.orders -= 1;
        self.count -= order.qty;
        self.expected -= order.sell_price * order.qty;

        let waiting = order.end_time - order.start_time;

        self.waiting += waiting;

        if waiting < 12 * 60 * 60 {
            self.successful += 1
        }

        self.executed_orders.push(order);
    }

    pub fn on_end_round(&mut self) {
        self.min_balance = self.min_balance.min(self.balance)
    }

    pub fn get_executed_orders(&self) -> Vec<Order> {
        self.executed_orders.to_vec()
    }

    pub fn get_wallet(&self) -> f64 {
        self.wallet
    }

    pub fn get_balance(&self) -> f64 {
        self.balance
    }

    pub fn get_orders(&self) -> usize {
        self.orders
    }

    pub fn get_waiting(&self) -> u64 {
        self.waiting
    }

    pub fn get_successful(&self) -> u64 {
        self.successful
    }

    pub fn get_min_balance(&self) -> f64 {
        self.min_balance
    }

    pub fn get_average_waiting(&self) -> f64 {
        if self.executed_orders.len() > 0 {
            self.get_waiting() as f64 / self.executed_orders.len() as f64
        } else {
            0f64
        }
    }

    pub fn get_successful_ratio(&self) -> f64 {
        if self.executed_orders.len() > 0 {
            self.get_successful() as f64 / self.executed_orders.len() as f64
        } else {
            0f64
        }
    }
}
