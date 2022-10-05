use crate::calculate_command::CalculateCommand;
use crate::calculate_iter::CalculateIter;
use crate::calculate_result::CalculateResult;
use crate::candle::Candle;

pub struct Calculate {
    candles: Vec<Candle>,
    initial_balance: f64,
    stake: f64,
    gain: f64,
    profit: f64,
    step_lot: f64,
    step_price: f64,
}

impl Calculate {
    pub fn new(
        candles: Vec<Candle>,
        initial_balance: Option<f64>,
        stake: Option<f64>,
        gain: Option<f64>,
        profit: Option<f64>,
    ) -> Self {
        Calculate {
            candles,
            initial_balance: initial_balance.unwrap_or(3000f64),
            stake: stake.unwrap_or(10f64),
            gain: gain.unwrap_or(1f64) / 100f64 + 1f64,
            step_lot: 1f64,
            step_price: 0.0001f64,
            profit: profit.unwrap_or(0.5f64),
        }
    }

    pub fn calculate(&self, results: Vec<u8>) -> CalculateResult {
        let gain = self.gain;
        let stake = self.stake;
        let mut calculate_iter = CalculateIter::new(
            &self.candles,
            self.initial_balance,
            self.profit,
            self.step_lot,
            self.step_price,
            Box::new(move |_candle, ind, _stats| match results.get(ind) {
                None => CalculateCommand::None,
                Some(val) => {
                    if *val == 0 {
                        CalculateCommand::None
                    } else {
                        CalculateCommand::BuyProfit(gain, stake)
                    }
                }
            }),
        );

        let mut cont = Ok(());

        while cont.is_ok() {
            cont = calculate_iter.next();
        }

        calculate_iter.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candle_response::CandleResponse;
    use crate::get_candles::perform_candles;
    use serde_json::from_reader;
    use std::fs::File;

    #[test]
    fn should_work() {
        let file = File::open("tests/candles.json").expect("file should open read only");
        let json = from_reader::<_, Vec<CandleResponse>>(file);

        assert_eq!(json.is_ok(), true);

        let candles = json
            .unwrap()
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<Candle>>();

        let candles = perform_candles(candles, 1655769600, 12, Some(vec![]));

        let results = candles
            .clone()
            .into_iter()
            .map(|candle| if candle.max_profit_12 > 1.0 { 1 } else { 0 })
            .collect::<Vec<u8>>();

        let calculate = Calculate::new(candles, None, None, None, None);

        let resp = calculate.calculate(results);

        /* wallet */
        assert_eq!(resp.wallet, 8.26311035000005);
        /* balance */
        assert_eq!(resp.balance, 2998.380237350004);
        /* base_real */
        assert_eq!(resp.base_real, 9.882000000000001);
        /* base_expected */
        assert_eq!(resp.base_expected, 9.97200000000015);
        /* min_balance */
        assert_eq!(resp.min_balance, 2560.199113750004);
        /* drawdown */
        assert_eq!(resp.drawdown, 0.9999700832904862);
        /* opened_orders */
        assert_eq!(resp.opened_orders, 1);
        /* executed_orders */
        assert_eq!(resp.executed_orders, 207);
        /* avg_wait  */
        assert_eq!(resp.avg_wait, 7309.144927536232);
        /* score */
        assert_eq!(resp.score, 8.262863144928028);
        /* successful_ratio */
        assert_eq!(resp.successful_ratio, 1.0);
    }
}
