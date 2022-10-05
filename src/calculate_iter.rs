use crate::calculate_activate::CalculateActivate;
use crate::calculate_agent::CalculateAgent;
use crate::calculate_command::CalculateCommand;
use crate::calculate_stats::CalculateStats;
use crate::{CalculateResult, Candle, Order};

type CalculateActivateLocal = Box<dyn Fn(&Candle, usize, &CalculateStats) -> CalculateCommand>;

pub struct CalculateIter<'a> {
    candles: &'a Vec<Candle>,
    profit: f64,
    step_lot: f64,
    step_price: f64,
    pub(crate) sell_orders: Vec<Order>,
    pointer: usize,
    pub(crate) agent: CalculateAgent<CalculateIterActivate>,
}

pub struct CalculateIterActivate {
    activate: CalculateActivateLocal,
    result: Option<CalculateResult>,
}

impl CalculateActivate for CalculateIterActivate {
    fn activate(
        &self,
        candle: &Candle,
        position: usize,
        stats: &CalculateStats,
    ) -> CalculateCommand {
        (self.activate)(candle, position, stats)
    }

    fn on_end(&mut self, result: CalculateResult) {
        self.result = Some(result);
    }
}

impl<'a> CalculateIter<'a> {
    pub fn new(
        candles: &'a Vec<Candle>,
        balance: f64,
        profit: f64,
        step_lot: f64,
        step_price: f64,
        activate: Box<dyn Fn(&Candle, usize, &CalculateStats) -> CalculateCommand>,
    ) -> Self {
        let agent = CalculateAgent::new(
            0,
            balance,
            false,
            Box::new(CalculateIterActivate {
                activate,
                result: None,
            }),
        );

        CalculateIter {
            candles,
            step_lot,
            step_price,
            profit,
            sell_orders: vec![],
            pointer: 0,
            agent,
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
            self.agent.on_end();
            return Err("not found");
        }

        let candle = candle.unwrap();

        match self.agent.activate(candle, self.pointer) {
            CalculateCommand::Unknown | CalculateCommand::None => {}
            CalculateCommand::BuyProfit(gain, stake) => {
                match self.agent.buy_profit_open(
                    candle,
                    stake,
                    gain,
                    self.step_lot,
                    self.step_price,
                ) {
                    None => {}
                    Some(order) => {
                        self.sell_orders.push(order);

                        self.sell_orders
                            .sort_by(|a, b| b.sell_price.partial_cmp(&a.sell_price).unwrap());
                    }
                }
            }
        }

        let mut cont = true;

        while cont {
            let order = self.sell_orders.last();

            if order.is_none() || order.unwrap().sell_price > candle.high {
                cont = false;
                continue;
            }

            let order = self.sell_orders.pop().unwrap();
            self.agent.buy_profit_close(order, candle, self.profit);
        }

        self.agent.on_end_round(candle.close);

        self.pointer += 1;

        Ok(())
    }
}
