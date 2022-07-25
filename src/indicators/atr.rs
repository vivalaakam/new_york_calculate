use crate::indicators::result::{IndicatorsError, IndicatorsResult};
use crate::truerange;

/**
Average True Range is a measure of volatility. It represents roughly how much you can expect a security to change in price on any given day. It is often used in position sizing formulas.

```
use new_york_calculate_core::indicators::atr;

let high_data = vec![12.3125,12.25,12.3125,12.2812,12.375,12.375,12.3125,12.375,12.5,12.5938,12.3438,12.4062,12.375,12.4062,12.375,12.2812,12.25,12.0625,12.3125,12.3125];
let low_data = vec![12.1875,12.1562,12.1875,12.125,12.125,12.2812,12.1875,12.1875,12.2812,12.3125,12.2812,12.2812,12.2812,12.2812,11.9688,12.0938,12.,11.6562,12.1562,12.125];
let close_data = vec![12.25,12.1875,12.25,12.1875,12.375,12.3125,12.2188,12.3438,12.5,12.375,12.3438,12.3438,12.2812,12.375,12.0938,12.1562,12.0625,11.9688,12.1875,12.1562];

let result = atr(high_data, low_data, close_data, 4);
assert_eq!(result.is_ok(), true);
let data = result.unwrap();
assert_eq!(data.0.len(), 17);
assert_eq!(format!("{:?}", data.0), "[0.125, 0.15625, 0.14063749999999997, 0.13672812499999998, 0.14942109374999998, 0.16676582031249995, 0.19539936523437493, 0.16999952392578116, 0.15874964294433586, 0.14251223220825188, 0.1381341741561889, 0.2051506306171417, 0.20071297296285634, 0.21303472972214227, 0.2613510472916067, 0.28193828546870503, 0.2583287141015288]");
```
 */
pub fn atr(
    high: Vec<f64>,
    low: Vec<f64>,
    close: Vec<f64>,
    period: usize,
) -> IndicatorsResult<(Vec<f64>,)> {
    if period < 1 {
        return Err(IndicatorsError::InvalidOption("period".to_string()));
    }

    if high.len() < period - 1 {
        return Ok((vec![],));
    }

    let mut output = vec![];

    let per = 1.0 / period as f64;

    let mut sum = 0.0;

    sum += high[0] - low[0];
    for i in 1..period {
        sum += truerange!(high[i], low[i], close[i - 1]);
    }

    let mut val = sum / period as f64;
    output.push(val);

    for i in period..high.len() {
        let v = truerange!(high[i], low[i], close[i - 1]);
        val = (v - val) * per + val;
        output.push(val)
    }

    Ok((output,))
}
