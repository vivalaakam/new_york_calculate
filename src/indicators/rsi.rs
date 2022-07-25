use crate::indicators::result::{IndicatorsError, IndicatorsResult};

/**
The Relative Strength Index is a momentum oscillator to help identify trends.

https://github.com/TulipCharts/tulipindicators/blob/master/indicators/rsi.c
```
use new_york_calculate_core::indicators::rsi;

let raw_data = vec![37.875,39.5,38.75,39.8125,40.0,39.875,40.1875,41.25,41.125,41.625,41.25,40.1875,39.9375,39.9375,40.5,41.9375,42.25,42.25,41.875,41.875];

let result = rsi(raw_data, 5);
assert_eq!(result.is_ok(), true);
let data = result.unwrap();
assert_eq!(data.0.len(), 15);
assert_eq!(format!("{:?}", data.0), "[76.66666666666666, 78.86792452830188, 84.91582491582491, 81.48626817447496, 84.59677419354838, 73.08512954495971, 49.31726219262107, 45.01190063805298, 45.011900638052985, 57.92515573823219, 75.95963053503601, 78.46761736795519, 78.46761736795519, 65.62991684424605, 65.62991684424605]");
```
 */
pub fn rsi(input: Vec<f64>, period: usize) -> IndicatorsResult<(Vec<f64>,)> {
    if period < 1 {
        return Err(IndicatorsError::InvalidOption("period".to_string()));
    }

    if input.len() < period {
        return Ok((vec![],));
    }

    let mut output = vec![];
    let per = 1.0 / (period as f64);

    let mut smooth_up = 0.0;
    let mut smooth_down = 0.0;

    for i in 1..period + 1 {
        smooth_up += if input[i] > input[i - 1] {
            input[i] - input[i - 1]
        } else {
            0.0
        };
        smooth_down += if input[i] < input[i - 1] {
            input[i - 1] - input[i]
        } else {
            0.0
        };
    }

    smooth_up /= period as f64;
    smooth_down /= period as f64;

    output.push(100.0 * (smooth_up / (smooth_up + smooth_down)));

    for i in period + 1..input.len() {
        let upward = if input[i] > input[i - 1] {
            input[i] - input[i - 1]
        } else {
            0.0
        };
        let downward = if input[i] < input[i - 1] {
            input[i - 1] - input[i]
        } else {
            0.0
        };

        smooth_up = (upward - smooth_up) * per + smooth_up;
        smooth_down = (downward - smooth_down) * per + smooth_down;

        output.push(100.0 * (smooth_up / (smooth_up + smooth_down)));
    }

    Ok((output,))
}
