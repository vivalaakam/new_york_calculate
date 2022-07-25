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
use crate::Candle;

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
                .map(|_| deltas[1].1 / deltas[1].0 - 1f64)
                .collect::<Vec<_>>(),
            Indicators::QuoteAsset24Delta => candles
                .into_iter()
                .map(|_| deltas[2].1 / deltas[2].0 - 1f64)
                .collect::<Vec<_>>(),
            Indicators::Trades24Delta => candles
                .into_iter()
                .map(|_| deltas[3].1 / deltas[3].0 - 1f64)
                .collect::<Vec<_>>(),
            Indicators::BuyBase24Delta => candles
                .into_iter()
                .map(|_| deltas[4].1 / deltas[4].0 - 1f64)
                .collect::<Vec<_>>(),
            Indicators::BuyQuote24Delta => candles
                .into_iter()
                .map(|_| deltas[5].1 / deltas[5].0 - 1f64)
                .collect::<Vec<_>>(),
        };

        data[data.len() - look_back..data.len()].to_vec()
    }
}
