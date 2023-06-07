use crate::indicators::result::{IndicatorsError, IndicatorsResult};

/**
The Simple Moving Average is one of the most common smoothing functions used on time series data.

https://github.com/TulipCharts/tulipindicators/blob/master/indicators/sma.c
```
use new_york_calculate_core::indicators::sma;

let close_data = vec![25.0,24.875,24.781,24.594,24.5,24.625,25.219,27.25];

let result = sma(close_data, 5);
assert_eq!(result.is_ok(), true);

let data = result.unwrap();
assert_eq!(data.0.len(), 4);
assert_eq!(format!("{:?}", data.0), "[24.75, 24.675, 24.7438, 25.2376]");
```
 */
pub fn sma(input: Vec<f64>, period: usize) -> IndicatorsResult<(Vec<f64>,)> {
    if period < 1 {
        return Err(IndicatorsError::InvalidOption("period".to_string()));
    }

    if input.is_empty() {
        return Ok((vec![],));
    }
    let scale = 1.0 / period as f64;
    let mut output = vec![];

    let mut sum = 0.0;

    for val in input.iter().take(period) {
        sum += val;
    }

    output.push(sum * scale);

    for i in period..input.len() {
        sum += input[i];
        sum -= input[i - period];
        output.push(sum * scale);
    }

    Ok((output,))
}
