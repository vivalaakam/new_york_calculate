use std::collections::HashMap;

use crate::activate::Activate;
use crate::{CalculateAgent, Candle};

pub struct Calculate<'a, T>
where
    T: Activate,
{
    candles: &'a HashMap<u64, Vec<Candle>>,
    pointer: usize,
    ts: Vec<u64>,
    check_period: usize,
    agents: Vec<CalculateAgent<T>>,
}

impl<'a, T> Calculate<'a, T>
where
    T: Activate,
{
    pub fn new(
        candles: &'a HashMap<u64, Vec<Candle>>,
        check_period: usize,
        agents: Vec<CalculateAgent<T>>,
    ) -> Self {
        let mut ts = candles.keys().copied().collect::<Vec<_>>();
        ts.sort();

        Calculate {
            candles,
            pointer: 0,
            check_period,
            agents,
            ts,
        }
    }

    pub fn get_pointer(&self) -> usize {
        self.pointer
    }

    pub fn on_end(&mut self) {
        for agent in self.agents.iter_mut() {
            agent.on_end();
        }
    }
}

impl<'a, T> Iterator for Calculate<'a, T>
where
    T: Activate,
{
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        let ts = self.ts.get(self.pointer)?;
        let candles = self.candles.get(ts)?;

        if self.pointer < self.candles.len() - self.check_period {
            for agent in self.agents.iter_mut() {
                for candle in candles.iter() {
                    let order = agent.activate(candle);
                    let _ = agent.perform_order(order, candle);
                }
            }
        }

        for agent in &mut self.agents {
            for candle in candles.iter() {
                agent.perform_candle(candle);
            }
            agent.on_end_round();
        }

        self.pointer += 1;

        Some(())
    }
}
