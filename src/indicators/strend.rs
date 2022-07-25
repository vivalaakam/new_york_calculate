use crate::indicators::result::IndicatorsResult;
use crate::{hl2, truerange};

/**
Average True Range is a measure of volatility. It represents roughly how much you can expect a security to change in price on any given day. It is often used in position sizing formulas.

```
use new_york_calculate_core::indicators::strend;

let high_data = vec![12.3125,12.25,12.3125,12.2812,12.375,12.375,12.3125,12.375,12.5,12.5938,12.3438,12.4062,12.375,12.4062,12.375,12.2812,12.25,12.0625,12.3125,12.3125];
let low_data = vec![12.1875,12.1562,12.1875,12.125,12.125,12.2812,12.1875,12.1875,12.2812,12.3125,12.2812,12.2812,12.2812,12.2812,11.9688,12.0938,12.,11.6562,12.1562,12.125];
let close_data = vec![12.25,12.1875,12.25,12.1875,12.375,12.3125,12.2188,12.3438,12.5,12.375,12.3438,12.3438,12.2812,12.375,12.0938,12.1562,12.0625,11.9688,12.1875,12.1562];

let result = strend(high_data, low_data, close_data, 4, 3);
assert_eq!(result.is_ok(), true);
let data = result.unwrap();
assert_eq!(data.0.len(), 16);
assert_eq!(format!("{:?}", data.0), "[12.71875, 12.71875, 12.660184375, 12.660184375, 12.660184375, 12.660184375, 12.660184375, 12.660184375, 12.660184375, 12.660184375, 12.660184375, 12.660184375, 12.660184375, 12.643403141874819, 12.643403141874819, 12.643403141874819]");

```
 */
pub fn strend(
    high: Vec<f64>,
    low: Vec<f64>,
    close: Vec<f64>,
    atr_period: usize,
    factor: usize,
) -> IndicatorsResult<(Vec<f64>,)> {
    let mut output = vec![];

    let per = 1.0 / atr_period as f64;

    let mut sum = 0.0;

    sum += high[0] - low[0];
    for i in 1..atr_period {
        sum += truerange!(high[i], low[i], close[i - 1]);
    }

    let mut atr = sum / atr_period as f64;

    let mut last_upper_band = 0.0;
    let mut last_lower_band = 0.0;
    let mut last_supertrend = 0.0;

    for i in atr_period..high.len() {
        let v = truerange!(high[i], low[i], close[i - 1]);
        atr = (v - atr) * per + atr;

        let hl2 = hl2!(high[i], low[i]);
        let delta = factor as f64 * atr;
        let upper_band_basic = hl2 + delta;
        let lower_band_basic = hl2 - delta;

        let upper_band = if upper_band_basic < last_upper_band || close[i - 1] > last_upper_band {
            upper_band_basic
        } else {
            last_upper_band
        };

        let lower_band = if lower_band_basic > last_lower_band || close[i - 1] < last_lower_band {
            lower_band_basic
        } else {
            last_lower_band
        };

        let supertrend = if (last_supertrend == last_upper_band && close[i] <= upper_band)
            || (last_supertrend == last_lower_band && close[i] <= lower_band)
        {
            upper_band
        } else if (last_supertrend == last_upper_band && close[i] >= upper_band)
            || (last_supertrend == last_lower_band && close[i] >= lower_band)
        {
            lower_band
        } else {
            0.0
        };

        last_upper_band = upper_band;
        last_lower_band = lower_band;
        last_supertrend = supertrend;

        output.push(supertrend);
    }

    Ok((output,))
}
