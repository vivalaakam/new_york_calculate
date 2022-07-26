use crate::calculate_command::CalculateCommand;
use crate::utils::{ceil_to_nearest, floor_to_nearest};
use crate::{Candle, Order};

type CalculateActivate = Box<dyn Fn(&Candle, usize) -> CalculateCommand>;

pub struct CalculateIter<'a> {
    candles: &'a Vec<Candle>,
    stake: f64,
    gain: f64,
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
}

impl<'a> CalculateIter<'a> {
    pub fn new(
        candles: &'a Vec<Candle>,
        balance: f64,
        stake: f64,
        gain: f64,
        profit: f64,
        interval: u64,
        step_lot: f64,
        step_price: f64,
        activate: Box<dyn Fn(&Candle, usize) -> CalculateCommand>,
    ) -> Self {
        CalculateIter {
            candles,
            stake,
            gain,
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
        }
    }

    pub fn next(&mut self) -> Result<(), &str> {
        let candle = self.candles.get(self.pointer);

        if candle.is_none() {
            return Err("not found");
        }

        let candle = candle.unwrap();

        match (self.activate)(candle, self.pointer) {
            CalculateCommand::None => {}
            CalculateCommand::BuyProfit => {
                if self.balance > self.stake {
                    let curr_stake = floor_to_nearest(self.stake / candle.open, self.step_lot);
                    let order_sum = curr_stake * candle.open;
                    self.balance -= order_sum;
                    self.balance -= order_sum * 0.001;

                    self.opened_orders.push(Order {
                        start_time: candle.start_time,
                        end_time: 0,
                        buy_price: candle.open,
                        sell_price: ceil_to_nearest(candle.open * self.gain, self.step_price),
                        qty: curr_stake,
                        commission: order_sum * 0.001,
                    });

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

            self.balance -= profit_size;
            self.wallet += profit_size;

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