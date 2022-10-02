use crate::calculate_command::CalculateCommand;
use crate::utils::{ceil_to_nearest, floor_to_nearest};
use crate::{Candle, Order};

#[derive(Debug)]
pub struct CalculateStats {
    pub balance: f64,
    pub orders: usize,
    pub count: f64,
    pub expected: f64,
    pub real: f64,
}

type CalculateActivate = Box<dyn Fn(&Candle, usize, &CalculateStats) -> CalculateCommand>;

pub struct CalculateIter<'a> {
    candles: &'a Vec<Candle>,
    profit: f64,
    step_lot: f64,
    step_price: f64,
    pub(crate) interval: u64,
    pub(crate) balance: f64,
    pub(crate) opened_orders: Vec<Order>,
    pub(crate) executed_orders: Vec<Order>,
    pub(crate) wallet: f64,
    pub(crate) min_balance: f64,
    pointer: usize,
    activate: CalculateActivate,
    stats: CalculateStats,
}

impl<'a> CalculateIter<'a> {
    pub fn new(
        candles: &'a Vec<Candle>,
        balance: f64,
        profit: f64,
        interval: u64,
        step_lot: f64,
        step_price: f64,
        activate: Box<dyn Fn(&Candle, usize, &CalculateStats) -> CalculateCommand>,
    ) -> Self {
        CalculateIter {
            candles,
            step_lot,
            step_price,
            profit,
            balance,
            opened_orders: vec![],
            executed_orders: vec![],
            wallet: 0.0,
            min_balance: balance,
            pointer: 0,
            activate,
            interval,
            stats: CalculateStats {
                orders: 0,
                count: 0.0,
                expected: 0.0,
                real: 0.0,
                balance: 0.0,
            },
        }
    }

    pub fn get_pointer(&self) -> usize {
        self.pointer
    }

    pub fn get_candle(&self, pointer: usize) -> Option<&Candle> {
        self.candles.get(pointer)
    }

    pub fn next(&mut self) -> Result<(), &str> {
        let candle = self.candles.get(self.pointer);

        if candle.is_none() {
            return Err("not found");
        }

        let candle = candle.unwrap();

        self.stats.real = self.stats.count * candle.open;
        self.stats.balance = self.balance;

        match (self.activate)(candle, self.pointer, &self.stats) {
            CalculateCommand::Unknown | CalculateCommand::None => {}
            CalculateCommand::BuyProfit(gain, stake) => {
                if self.balance > stake {
                    let curr_stake = floor_to_nearest(stake / candle.open, self.step_lot);
                    let order_sum = curr_stake * candle.open;
                    self.balance -= order_sum;
                    self.balance -= order_sum * 0.001;

                    let order = Order {
                        start_time: candle.start_time,
                        end_time: 0,
                        buy_price: candle.open,
                        sell_price: ceil_to_nearest(candle.open * gain, self.step_price),
                        qty: curr_stake,
                        commission: order_sum * 0.001,
                        gain,
                        stake,
                        profit: 0.0,
                    };

                    self.stats.orders += 1;
                    self.stats.count += order.qty;
                    self.stats.expected += order.sell_price * order.qty;

                    self.opened_orders.push(order);

                    self.opened_orders
                        .sort_by(|a, b| b.sell_price.partial_cmp(&a.sell_price).unwrap());
                }
            }
        }

        let mut cont = true;

        while cont {
            let order = self.opened_orders.last();

            if order.is_none() || order.unwrap().sell_price > candle.high {
                cont = false;
                continue;
            }

            let mut order = self.opened_orders.pop().unwrap();

            let order_sum = order.sell_price * order.qty;

            self.balance += order_sum;
            self.balance -= order_sum * 0.001;
            order.commission += order_sum * 0.001;

            order.end_time = candle.start_time;

            let profit_size =
                ((order.sell_price - order.buy_price) * order.qty - order.commission) * self.profit;

            order.profit = profit_size;

            self.balance -= profit_size;
            self.wallet += profit_size;

            self.stats.orders -= 1;
            self.stats.count -= order.qty;
            self.stats.expected -= order.sell_price * order.qty;

            self.executed_orders.push(order);
        }

        self.min_balance = self.min_balance.min(self.balance);

        self.pointer += 1;

        Ok(())
    }

    pub fn last_candle(&self) -> Option<&Candle> {
        self.candles.last()
    }
}
