use crate::activate::Activate;
use crate::types::TimeStamp;
use crate::{CalculateAgent, CandleTrait};
use std::collections::HashMap;
use std::fmt::Debug;
use tracing::warn;

pub struct Calculate<'a, T, C>
where
    T: Activate<C>,
    C: CandleTrait,
{
    candles: &'a HashMap<TimeStamp, Vec<C>>,
    pointer: usize,
    ts: Vec<TimeStamp>,
    agents: Vec<CalculateAgent<T, C>>,
}

impl<'a, T, C> Calculate<'a, T, C>
where
    T: Activate<C>,
    C: CandleTrait + Debug,
{
    pub fn new(candles: &'a HashMap<TimeStamp, Vec<C>>, agents: Vec<CalculateAgent<T, C>>) -> Self {
        let mut ts = candles.keys().copied().collect::<Vec<_>>();
        ts.sort();

        Calculate {
            candles,
            pointer: 1,
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
        let prev_ts = self.ts.get(self.pointer - 1)?;
        let prev_candles = self.candles.get(prev_ts)?;

        let ts = self.ts.get(self.pointer)?;
        let current_candles = self.candles.get(ts)?;

        // Create a symbol-to-candle mapping for O(1) lookups
        let candle_map: HashMap<_, _> = prev_candles.iter().map(|c| (c.get_symbol(), c)).collect();
        let price_map: HashMap<_, _> = current_candles
            .iter()
            .map(|c| (c.get_symbol(), c.get_open()))
            .collect();

        for agent in self.agents.iter_mut() {
            let orders = agent.activate(prev_candles, &price_map);
            for order in orders {
                let candle = candle_map.get(&order.get_symbol());

                if let Some(&candle) = candle {
                    let result = agent.perform_order(order, candle);

                    if let Err(e) = result {
                        warn!(error = ?e, "Error performing order");
                    }
                }
            }

            for candle in current_candles.iter() {
                agent.perform_candle(candle);
            }
        }

        self.pointer += 1;
        Some(())
    }
}
