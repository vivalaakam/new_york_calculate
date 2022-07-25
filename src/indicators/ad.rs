use crate::indicators::result::IndicatorsResult;

/**
Average True Range is a measure of volatility. It represents roughly how much you can expect a security to change in price on any given day. It is often used in position sizing formulas.

```
use new_york_calculate_core::indicators::ad;

let high_data = vec![8.625,8.875,9.375,10.125,9.375,10.125,10.,9.75,9.5,9.625,10.,9.75,9.125,9.25,9.375,9.375,9.375,8.625,8.625,8.625];
let low_data = vec![8.25,8.375,8.375,8.75,8.75,9.25,9.125,9.375,9.0,8.875,9.375,8.75,8.75,9.125,9.25,9.125,8.625,8.25,8.25,7.875];
let close_data = vec![8.625,8.375,9.375,8.75,9.375,9.875,9.375,9.625,9.125,9.25,9.75,8.875,8.875,9.125,9.375,9.25,8.625,8.5,8.5,7.875];
let volume_data = vec![19194.0,10768.0,20032.0,55218.0,13172.0,22245.0,15987.0,9646.0,10848.0,14470.0,14973.0,15799.0,16860.0,6568.0,8312.0,5573.0,11480.0,6366.0,8394.0,12616.0];

let result = ad(high_data, low_data, close_data, volume_data);
assert_eq!(result.is_ok(), true);
let data = result.unwrap();
assert_eq!(data.0.len(), 20);
assert_eq!(format!("{:?}", data.0), "[19194.0, 8426.0, 28458.0, -26760.0, -13588.0, -4054.4285714285725, -10906.0, -7690.666666666667, -13114.666666666668, -13114.666666666668, -10120.066666666668, -21969.316666666666, -27589.316666666666, -34157.316666666666, -25845.316666666666, -25845.316666666666, -37325.316666666666, -35203.316666666666, -32405.316666666666, -45021.316666666666]");
```
 */
pub fn ad(
    high: Vec<f64>,
    low: Vec<f64>,
    close: Vec<f64>,
    volume: Vec<f64>,
) -> IndicatorsResult<(Vec<f64>,)> {
    let mut output = vec![];
    let mut sum = 0.0;

    for i in 0..high.len() {
        let hl = high[i] - low[i];
        if hl != 0.0 {
            sum += (close[i] - low[i] - high[i] + close[i]) / hl * volume[i];
        }
        output.push(sum)
    }

    return Ok((output,));
}
