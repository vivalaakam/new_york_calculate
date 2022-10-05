use crate::calculate_activate::CalculateActivate;
use crate::{CalculateAgent, CalculateCommand, Candle, Order};

pub struct CalculateAgentIter<'a, T: CalculateActivate> {
    candles: &'a Vec<Candle>,
    profit: f64,
    step_lot: f64,
    step_price: f64,
    sell_orders: Vec<Order>,
    pointer: usize,
    agents: Vec<CalculateAgent<T>>,
}

impl<'a, T> CalculateAgentIter<'a, T>
where
    T: CalculateActivate,
{
    pub fn new(
        candles: &'a Vec<Candle>,
        profit: f64,
        step_lot: f64,
        step_price: f64,
        agents: Vec<CalculateAgent<T>>,
    ) -> Self {
        CalculateAgentIter {
            candles,
            step_lot,
            step_price,
            profit,
            sell_orders: vec![],
            pointer: 0,
            agents,
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
            for agent in self.agents.iter_mut() {
                agent.on_end();
            }
            return Err("not found");
        }

        let candle = candle.unwrap();

        for agent in self.agents.iter_mut() {
            match agent.activate(candle, self.pointer) {
                CalculateCommand::Unknown | CalculateCommand::None => {}
                CalculateCommand::BuyProfit(gain, stake) => {
                    match agent.buy_profit_open(candle, stake, gain, self.step_lot, self.step_price)
                    {
                        Some(order) => self.sell_orders.push(order),
                        None => {}
                    }
                }
            };
        }

        self.sell_orders
            .sort_by(|a, b| b.sell_price.partial_cmp(&a.sell_price).unwrap());

        let mut cont = true;

        while cont {
            let order = self.sell_orders.last();

            if order.is_none() || order.unwrap().sell_price > candle.high {
                cont = false;
                continue;
            }

            let order = self.sell_orders.pop().unwrap();

            match self.agents.get_mut(order.agent) {
                None => {}
                Some(agent) => agent.buy_profit_close(order, candle, self.profit),
            }
        }

        for agent in self.agents.iter_mut() {
            agent.on_end_round(candle.close);
        }

        self.pointer += 1;

        Ok(())
    }
}
