use std::collections::HashMap;

use crate::activate::Activate;
use crate::{CalculateAgent, CandleTrait};

pub struct Calculate<'a, T, C>
where
    T: Activate<C>,
    C: CandleTrait,
{
    candles: &'a HashMap<u64, Vec<C>>,
    pointer: usize,
    ts: Vec<u64>,
    agents: Vec<CalculateAgent<T, C>>,
}

impl<'a, T, C> Calculate<'a, T, C>
where
    T: Activate<C>,
    C: CandleTrait,
{
    pub fn new(candles: &'a HashMap<u64, Vec<C>>, agents: Vec<CalculateAgent<T, C>>) -> Self {
        let mut ts = candles.keys().copied().collect::<Vec<_>>();
        ts.sort();

        Calculate {
            candles,
            pointer: 0,
            agents,
            ts,
        }
    }

    pub fn get_agents(&self) -> &Vec<CalculateAgent<T, C>> {
        &self.agents
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

impl<'a, T, C> Iterator for Calculate<'a, T, C>
where
    T: Activate<C>,
    C: CandleTrait,
{
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        let ts = self.ts.get(self.pointer)?;
        let candles = self.candles.get(ts)?;

        for agent in self.agents.iter_mut() {
            for candle in candles.iter() {
                let order = agent.activate(candle, candles);
                let _ = agent.perform_order(order, candle);
            }
        }

        for agent in &mut self.agents {
            for candle in candles.iter() {
                agent.perform_candle(candle);
            }
            agent.on_end_round(*ts, candles);
        }

        self.pointer += 1;

        Some(())
    }
}
