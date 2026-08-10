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
    /// Built once in `new()` — O(1) candle lookup per order without per-tick HashMap.
    symbol_index: HashMap<(TimeStamp, &'a str), usize>,
    price_map_buf: HashMap<&'a str, f32>,
}

fn build_symbol_index<C: CandleTrait>(
    candles: &HashMap<TimeStamp, Vec<C>>,
) -> HashMap<(TimeStamp, &str), usize> {
    let mut index = HashMap::with_capacity(candles.values().map(|row| row.len()).sum());
    for (&ts, row) in candles {
        for (i, c) in row.iter().enumerate() {
            index.insert((ts, c.get_symbol()), i);
        }
    }
    index
}

impl<'a, T, C> Calculate<'a, T, C>
where
    T: Activate<C>,
    C: CandleTrait + Debug,
{
    pub fn new(candles: &'a HashMap<TimeStamp, Vec<C>>, agents: Vec<CalculateAgent<T, C>>) -> Self {
        let mut ts = candles.keys().copied().collect::<Vec<_>>();
        ts.sort();
        let symbol_index = build_symbol_index(candles);

        Calculate {
            candles,
            pointer: 1,
            agents,
            ts,
            symbol_index,
            price_map_buf: HashMap::new(),
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
        let prev_ts = *self.ts.get(self.pointer - 1)?;
        let ts = *self.ts.get(self.pointer)?;
        let prev_candles = self.candles.get(&prev_ts)?;
        let current_candles = self.candles.get(&ts)?;

        let mut price_map = std::mem::take(&mut self.price_map_buf);
        price_map.clear();
        price_map.extend(
            current_candles
                .iter()
                .map(|c| (c.get_symbol(), c.get_open())),
        );

        for agent in self.agents.iter_mut() {
            let orders = agent.activate(prev_candles, &price_map);
            for order in orders {
                let candle = self
                    .symbol_index
                    .get(&(prev_ts, order.get_symbol()))
                    .and_then(|&i| prev_candles.get(i));

                if let Some(candle) = candle {
                    let result = agent.perform_order(order, candle);

                    if let Err(e) = result {
                        warn!(error = ?e, "Error performing order");
                    }
                }
            }

            for candle in current_candles.iter() {
                agent.perform_candle(candle);
            }

            agent.on_end_round(ts, current_candles);
        }

        self.price_map_buf = price_map;

        self.pointer += 1;
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{Candle, init_tracing};
    use crate::{Activate, CalculateCommand, CalculateResultRef, Symbol, buy_market};
    use std::collections::HashMap;
    use tracing::info;

    #[derive(Debug, Default)]
    struct TestActivate;

    impl Activate<Candle> for TestActivate {
        fn activate(
            &self,
            candles: &[Candle],
            _prices: &HashMap<&str, f32>,
            _stats: CalculateResultRef<'_>,
            _active: &HashMap<Symbol, Vec<crate::Order>>,
        ) -> Vec<CalculateCommand> {
            let Some(candle) = candles.last() else {
                return vec![];
            };
            if candle.start_time == 0 {
                vec![buy_market!(candle.get_symbol(), 5.0)]
            } else {
                vec![CalculateCommand::None]
            }
        }
    }

    #[test]
    fn calculate_iter_updates_min_balance() {
        init_tracing();

        let symbol = "BTC".to_string();
        let mut candles = HashMap::new();
        candles.insert(
            0,
            vec![Candle {
                symbol: symbol.clone(),
                start_time: 0,
                open: 100.0,
                high: 120.0,
                low: 90.0,
                close: 110.0,
            }],
        );
        candles.insert(
            1,
            vec![Candle {
                symbol: symbol.clone(),
                start_time: 1,
                open: 100.0,
                high: 120.0,
                low: 90.0,
                close: 110.0,
            }],
        );

        let agent = CalculateAgent::new(1000.0, 0.0001, Box::new(TestActivate));
        let mut calc = Calculate::new(&candles, vec![agent]);

        while calc.next().is_some() {}

        let result = calc.get_agents()[0].get_result();
        info!(?result, "after calculate iter");

        assert_eq!(result.balance, 499.95);
        assert_eq!(result.min_balance, 499.95);
    }
}
