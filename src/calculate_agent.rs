use crate::calculate_activate::CalculateActivate;
use crate::calculate_stats::CalculateStats;
use crate::score::get_score;
use crate::utils::{ceil_to_nearest, floor_to_nearest};
use crate::{CalculateCommand, CalculateResult, Candle, Order};

pub struct CalculateAgent<T: CalculateActivate + ?Sized> {
    index: usize,
    balance: f64,
    executed_orders: Vec<Order>,
    executed_orders_len: usize,
    wallet: f64,
    min_balance: f64,
    orders: usize,
    count: f64,
    expected: f64,
    waiting: u64,
    successful: u64,
    close_price: f64,
    cache_orders: bool,
    activate: Box<T>,
}

impl<T> CalculateAgent<T>
where
    T: CalculateActivate + ?Sized,
{
    pub fn new(
        index: usize,
        balance: f64,
        cache_orders: bool,
        activate: Box<T>,
    ) -> CalculateAgent<T> {
        CalculateAgent {
            index,
            balance,
            activate,
            min_balance: balance,
            executed_orders: vec![],
            executed_orders_len: 0,
            wallet: 0.0,
            orders: 0,
            count: 0.0,
            expected: 0.0,
            waiting: 0,
            successful: 0,
            close_price: 0.0,
            cache_orders,
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
            agent: self.index,
        };

        self.orders += 1;
        self.count += order.qty;
        self.expected += order.sell_price * order.qty;

        Some(order)
    }

    pub fn buy_profit_close(&mut self, order: Order, candle: &Candle, profit: f64) {
        let mut order = order;
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

        self.executed_orders_len += 1;
        if self.cache_orders {
            self.executed_orders.push(order);
        }
    }

    pub fn on_end_round(&mut self, close_price: f64) {
        self.min_balance = self.min_balance.min(self.balance);
        self.close_price = close_price;
    }

    pub fn get_executed_orders(&self) -> Vec<Order> {
        self.executed_orders.to_vec()
    }

    pub fn get_executed_orders_len(&self) -> usize {
        self.executed_orders_len
    }

    pub fn get_wallet(&self) -> f64 {
        self.wallet
    }

    pub fn get_balance(&self) -> f64 {
        self.balance
    }

    pub fn get_count(&self) -> f64 {
        self.count
    }

    pub fn get_expected(&self) -> f64 {
        self.expected
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

    pub fn get_close_price(&self) -> f64 {
        self.close_price
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_average_waiting(&self) -> f64 {
        if self.get_executed_orders_len() > 0 {
            self.get_waiting() as f64 / self.get_executed_orders_len() as f64
        } else {
            0f64
        }
    }

    pub fn get_successful_ratio(&self) -> f64 {
        if self.get_executed_orders_len() > 0 {
            self.get_successful() as f64 / self.get_executed_orders_len() as f64
        } else {
            0f64
        }
    }

    pub fn get_draw_down(&self) -> f64 {
        if self.get_orders() > 0 {
            (self.get_count() * self.get_close_price() + self.get_balance())
                / (self.get_expected() + self.get_balance())
        } else {
            1f64
        }
    }

    pub fn on_end(&mut self) {
        self.activate.on_end(self.get_result())
    }

    pub fn get_result(&self) -> CalculateResult {
        CalculateResult {
            wallet: self.get_wallet(),
            balance: self.get_balance(),
            base_real: self.get_count() * self.get_close_price(),
            base_expected: self.get_expected(),
            min_balance: self.get_min_balance(),
            drawdown: self.get_draw_down(),
            opened_orders: self.get_orders(),
            executed_orders: self.get_executed_orders_len(),
            avg_wait: self.get_average_waiting(),
            score: get_score(
                self.get_wallet(),
                self.get_draw_down(),
                self.get_successful_ratio(),
            ),
            successful_ratio: self.get_successful_ratio(),
        }
    }
}
