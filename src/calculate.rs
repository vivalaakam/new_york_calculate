use crate::activate::Activate;
use crate::{CalculateAgent, CandleTrait};
use std::collections::HashMap;
use std::fmt::Debug;

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
    C: CandleTrait + Debug,
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

impl<T, C> Iterator for Calculate<'_, T, C>
where
    T: Activate<C>,
    C: CandleTrait + Debug,
{
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        let ts = self.ts.get(self.pointer)?;
        let candles = self.candles.get(ts)?;

        // Create a symbol-to-candle mapping for O(1) lookups
        let candle_map: HashMap<_, _> = candles.iter().map(|c| (c.get_symbol(), c)).collect();

        for agent in self.agents.iter_mut() {
            let orders = agent.activate(candles);
            for order in orders {
                if let Some(&candle) = candle_map.get(&order.get_symbol()) {
                    let _ = agent.perform_order(order, candle);
                }
            }

            for candle in candles.iter() {
                agent.perform_candle(candle);
            }
            agent.on_end_round(*ts, candles);
        }

        self.pointer += 1;
        Some(())
    }
}
