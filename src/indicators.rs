pub use ad::ad;
pub use adx::adx;
pub use ao::ao;
pub use atr::atr;
pub use bbands::bbands;
pub use cci::cci;
pub use cpr::cpr;
pub use ema::ema;
pub use hma::hma;
pub use macd::macd;
pub use mom::mom;
pub use obv::obv;
pub use rsi::rsi;
pub use sma::sma;
pub use stoch::stoch;
pub use stochrsi::stochrsi;
pub use strend::strend;
pub use vpt::vpt;
pub use vwma::vwma;
pub use willr::willr;

use crate::utils::range;
use crate::{hl2, hlc3, hlcc4, ohlc4, Candle};

mod ad;
mod adx;
mod ao;
mod atr;
mod bbands;
mod buffer;
mod cci;
mod cpr;
mod ema;
mod hma;
mod macd;
mod macros;
mod mom;
mod obv;
mod result;
mod rsi;
mod sma;
mod stoch;
mod stochrsi;
mod strend;
mod vpt;
mod vwma;
mod willr;

#[derive(PartialEq, Debug)]
pub enum IndicatorsInput {
    Close,
    HLC3,
    HL2,
    OHLC4,
    HLCC4,
}

#[derive(PartialEq, Debug)]
pub enum Indicators {
    Open24,
    High24,
    Low24,
    Close24,
    Volume24,
    QuoteAsset24,
    Trades24,
    BuyBase24,
    BuyQuote24,
    Candle24Delta,
    Volume24Delta,
    QuoteAsset24Delta,
    Trades24Delta,
    BuyBase24Delta,
    BuyQuote24Delta,
    Ad,
    Adx(usize),
    Ao,
    Atr(usize),
    BBands(IndicatorsInput, usize, f64, usize),
    Cci(usize),
    Cpr(usize),
    Ema(IndicatorsInput, usize),
    Hma(IndicatorsInput, usize),
    Macd(IndicatorsInput, usize, usize, usize, usize),
    Mom(IndicatorsInput, usize),
    Obv,
    Rsi(IndicatorsInput, usize),
    Sma(IndicatorsInput, usize),
    Stoch(usize, usize, usize, usize),
    StochRsi(IndicatorsInput, usize),
    STrend(usize, usize),
    Vpt,
    Vwma(IndicatorsInput, usize),
    WillR(usize),
}

impl Indicators {
    pub fn candle_24() -> Vec<Indicators> {
        vec![
            Indicators::Open24,
            Indicators::High24,
            Indicators::Low24,
            Indicators::Close24,
            Indicators::Candle24Delta,
        ]
    }

    pub fn volume_24() -> Vec<Indicators> {
        vec![Indicators::Volume24, Indicators::Volume24Delta]
    }

    pub fn quote_asset_24() -> Vec<Indicators> {
        vec![Indicators::QuoteAsset24, Indicators::QuoteAsset24Delta]
    }

    pub fn trades_24() -> Vec<Indicators> {
        vec![Indicators::Trades24, Indicators::Trades24Delta]
    }

    pub fn buy_base_24() -> Vec<Indicators> {
        vec![Indicators::BuyBase24, Indicators::BuyBase24Delta]
    }

    pub fn buy_quote_24() -> Vec<Indicators> {
        vec![Indicators::BuyQuote24, Indicators::BuyQuote24Delta]
    }

    pub fn get_data(
        &self,
        candles: &Vec<&Candle>,
        look_back: usize,
        deltas: &Vec<(f64, f64)>,
    ) -> Vec<f64> {
        let data = match self {
            Indicators::Open24 => candles
                .into_iter()
                .map(|c| range(deltas[0].0, deltas[0].1, 0f64, 1f64, c.open))
                .collect::<Vec<_>>(),
            Indicators::High24 => candles
                .into_iter()
                .map(|c| range(deltas[0].0, deltas[0].1, 0f64, 1f64, c.high))
                .collect::<Vec<_>>(),
            Indicators::Low24 => candles
                .into_iter()
                .map(|c| range(deltas[0].0, deltas[0].1, 0f64, 1f64, c.low))
                .collect::<Vec<_>>(),
            Indicators::Close24 => candles
                .into_iter()
                .map(|c| range(deltas[0].0, deltas[0].1, 0f64, 1f64, c.close))
                .collect::<Vec<_>>(),
            Indicators::Volume24 => candles
                .into_iter()
                .map(|c| range(deltas[1].0, deltas[1].1, 0f64, 1f64, c.volume))
                .collect::<Vec<_>>(),
            Indicators::QuoteAsset24 => candles
                .into_iter()
                .map(|c| range(deltas[2].0, deltas[2].1, 0f64, 1f64, c.quote))
                .collect::<Vec<_>>(),
            Indicators::Trades24 => candles
                .into_iter()
                .map(|c| range(deltas[3].0, deltas[3].1, 0f64, 1f64, c.trades))
                .collect::<Vec<_>>(),
            Indicators::BuyBase24 => candles
                .into_iter()
                .map(|c| range(deltas[4].0, deltas[4].1, 0f64, 1f64, c.buy_base))
                .collect::<Vec<_>>(),
            Indicators::BuyQuote24 => candles
                .into_iter()
                .map(|c| range(deltas[5].0, deltas[5].1, 0f64, 1f64, c.buy_quote))
                .collect::<Vec<_>>(),
            Indicators::Candle24Delta => candles
                .into_iter()
                .map(|_| deltas[0].1 / deltas[0].0 - 1f64)
                .collect::<Vec<_>>(),
            Indicators::Volume24Delta => candles
                .into_iter()
                .map(|_| (deltas[1].1 / deltas[1].0 - 1f64).min(10000f64))
                .collect::<Vec<_>>(),
            Indicators::QuoteAsset24Delta => candles
                .into_iter()
                .map(|_| (deltas[2].1 / deltas[2].0 - 1f64).min(10000f64))
                .collect::<Vec<_>>(),
            Indicators::Trades24Delta => candles
                .into_iter()
                .map(|_| (deltas[3].1 / deltas[3].0 - 1f64).min(10000f64))
                .collect::<Vec<_>>(),
            Indicators::BuyBase24Delta => candles
                .into_iter()
                .map(|_| (deltas[4].1 / deltas[4].0 - 1f64).min(10000f64))
                .collect::<Vec<_>>(),
            Indicators::BuyQuote24Delta => candles
                .into_iter()
                .map(|_| (deltas[5].1 / deltas[5].0 - 1f64).min(10000f64))
                .collect::<Vec<_>>(),
            Indicators::Ad => {
                let mut high_data = vec![];
                let mut low_data = vec![];
                let mut close_data = vec![];
                let mut volume_data = vec![];

                for candle in candles {
                    high_data.push(candle.high);
                    low_data.push(candle.low);
                    close_data.push(candle.close);
                    volume_data.push(candle.volume);
                }

                let result = ad(high_data, low_data, close_data, volume_data);
                result.unwrap().0
            }
            Indicators::Adx(period) => {
                let mut high_data = vec![];
                let mut low_data = vec![];

                for candle in candles {
                    high_data.push(candle.high);
                    low_data.push(candle.low);
                }

                let result = adx(high_data, low_data, *period);

                result.unwrap().0
            }
            Indicators::Ao => {
                let mut high_data = vec![];
                let mut low_data = vec![];

                for candle in candles {
                    high_data.push(candle.high);
                    low_data.push(candle.low);
                }

                let result = ao(high_data, low_data);

                result.unwrap().0
            }
            Indicators::Atr(period) => {
                let mut high_data = vec![];
                let mut low_data = vec![];
                let mut close_data = vec![];

                for candle in candles {
                    high_data.push(candle.high);
                    low_data.push(candle.low);
                    close_data.push(candle.close);
                }

                let result = atr(high_data, low_data, close_data, *period);

                result.unwrap().0
            }
            Indicators::BBands(input, period, stddev, output) => {
                let raw_data = get_input(candles, input);
                let result = bbands(raw_data, *period, *stddev).unwrap();

                match output {
                    0 => result.0,
                    1 => result.1,
                    2 => result.2,
                    _ => result.0,
                }
            }
            Indicators::Cci(period) => {
                let mut high_data = vec![];
                let mut low_data = vec![];
                let mut close_data = vec![];

                for candle in candles {
                    high_data.push(candle.high);
                    low_data.push(candle.low);
                    close_data.push(candle.close);
                }

                let result = cci(high_data, low_data, close_data, *period);

                result.unwrap().0
            }
            Indicators::Cpr(output) => {
                let mut high_data = vec![];
                let mut low_data = vec![];
                let mut close_data = vec![];

                for candle in candles {
                    high_data.push(candle.high);
                    low_data.push(candle.low);
                    close_data.push(candle.close);
                }

                let result = cpr(high_data, low_data, close_data).unwrap();

                match output {
                    0 => result.0,
                    1 => result.1,
                    2 => result.2,
                    _ => result.0,
                }
            }
            Indicators::Ema(input, period) => {
                let close_data = get_input(candles, input);
                let result = ema(close_data, *period);
                result.unwrap().0
            }
            Indicators::Hma(input, period) => {
                let close_data = get_input(candles, input);
                let result = hma(close_data, *period);
                result.unwrap().0
            }
            Indicators::Macd(input, short_period, long_period, signal_period, output) => {
                let raw_data = get_input(candles, input);

                let result = macd(raw_data, *short_period, *long_period, *signal_period).unwrap();

                match output {
                    0 => result.0,
                    1 => result.1,
                    2 => result.2,
                    _ => result.0,
                }
            }
            Indicators::Mom(input, period) => {
                let raw_data = get_input(candles, input);
                let result = mom(raw_data, *period);
                result.unwrap().0
            }
            Indicators::Obv => {
                let mut close_data = vec![];
                let mut volume_data = vec![];

                for candle in candles {
                    close_data.push(candle.close);
                    volume_data.push(candle.volume);
                }

                let result = obv(close_data, volume_data);

                result.unwrap().0
            }
            Indicators::Rsi(input, period) => {
                let raw_data = get_input(candles, input);
                let result = rsi(raw_data, *period);
                result.unwrap().0
            }
            Indicators::Sma(input, period) => {
                let close_data = get_input(candles, input);
                let result = sma(close_data, *period);
                result.unwrap().0
            }
            Indicators::Stoch(kperiod, kslow, dperiod, output) => {
                let mut high_data = vec![];
                let mut low_data = vec![];
                let mut close_data = vec![];

                for candle in candles {
                    high_data.push(candle.high);
                    low_data.push(candle.low);
                    close_data.push(candle.close);
                }
                let result =
                    stoch(high_data, low_data, close_data, *kperiod, *kslow, *dperiod).unwrap();

                match output {
                    0 => result.0,
                    1 => result.1,
                    _ => result.0,
                }
            }
            Indicators::StochRsi(input, period) => {
                let raw_data = get_input(candles, input);
                let result = stochrsi(raw_data, *period);
                result.unwrap().0
            }
            Indicators::STrend(atr_period, factor) => {
                let mut high_data = vec![];
                let mut low_data = vec![];
                let mut close_data = vec![];

                for candle in candles {
                    high_data.push(candle.high);
                    low_data.push(candle.low);
                    close_data.push(candle.close);
                }

                let result = strend(high_data, low_data, close_data, *atr_period, *factor);

                result.unwrap().0
            }
            Indicators::Vpt => {
                let mut volume_data = vec![];
                let mut close_data = vec![];

                for candle in candles {
                    close_data.push(candle.close);
                    volume_data.push(candle.volume);
                }

                let result = vpt(close_data, volume_data);

                result.unwrap().0
            }
            Indicators::Vwma(input, period) => {
                let mut volume_data = vec![];
                let input_data = get_input(candles, input);

                for candle in candles {
                    volume_data.push(candle.volume);
                }

                let result = vwma(input_data, volume_data, *period);

                result.unwrap().0
            }
            Indicators::WillR(period) => {
                let mut high_data = vec![];
                let mut low_data = vec![];
                let mut close_data = vec![];

                for candle in candles {
                    high_data.push(candle.high);
                    low_data.push(candle.low);
                    close_data.push(candle.close);
                }

                let result = willr(high_data, low_data, close_data, *period);

                result.unwrap().0
            }
        };

        data[data.len() - look_back..data.len()].to_vec()
    }
}

fn get_input(candles: &Vec<&Candle>, input: &IndicatorsInput) -> Vec<f64> {
    let mut raw_data = vec![];

    match input {
        IndicatorsInput::Close => {
            for candle in candles {
                raw_data.push(candle.close);
            }
        }
        IndicatorsInput::HLC3 => {
            for candle in candles {
                raw_data.push(hlc3!(candle.high, candle.low, candle.close));
            }
        }
        IndicatorsInput::HL2 => {
            for candle in candles {
                raw_data.push(hl2!(candle.high, candle.low));
            }
        }
        IndicatorsInput::OHLC4 => {
            for candle in candles {
                raw_data.push(ohlc4!(candle.open, candle.high, candle.low, candle.close));
            }
        }
        IndicatorsInput::HLCC4 => {
            for candle in candles {
                raw_data.push(hlcc4!(candle.high, candle.low, candle.close));
            }
        }
    }

    raw_data
}
