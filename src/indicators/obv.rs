use crate::indicators::result::IndicatorsResult;

/**
The On Balance Volume indicator calculates a running total of volume. Volume is added on up-days and subtracted on down days

https://github.com/TulipCharts/tulipindicators/blob/master/indicators/obv.c
```
use new_york_calculate_core::indicators::obv;

let close_data = vec![20.5625,20.375,20.0625,19.5,19.8125,19.8125,20.0625,20.0625,20.375,20.75];
let volume_data = vec![27802.0,16178.0,22766.0,46074.0,22904.0,20428.0,29260.0,30652.0,38332.0,40054.0];

let result = obv(close_data, volume_data);
assert_eq!(result.is_ok(), true);

let data = result.unwrap();
assert_eq!(data.0.len(), 10);
assert_eq!(format!("{:?}", data.0), "[0.0, -16178.0, -38944.0, -85018.0, -62114.0, -62114.0, -32854.0, -32854.0, 5478.0, 45532.0]");
```
 */
pub fn obv(close: Vec<f64>, volume: Vec<f64>) -> IndicatorsResult<(Vec<f64>,)> {
    let mut sum = 0.0;
    let mut output = vec![];
    output.push(sum);

    for i in 1..close.len() {
        if close[i] > close[i - 1] {
            sum += volume[i];
        } else if close[i] < close[i - 1] {
            sum -= volume[i];
        }

        output.push(sum)
    }

    Ok((output,))
}
