use crate::indicators::result::{IndicatorsError, IndicatorsResult};

/**
The exponential moving average, or exponential smoothing function, works by calculating each bar as a portion of the current input and a portion of the previous exponential moving average.

https://github.com/TulipCharts/tulipindicators/blob/master/indicators/ema.c
```
use new_york_calculate_core::indicators::ema;

let close_data = vec![25.0,24.875,24.781,24.594,24.5,24.625,25.219,27.25];

let result = ema(close_data, 5);
assert_eq!(result.is_ok(), true);

let data = result.unwrap();
assert_eq!(data.0.len(), 8);
assert_eq!(format!("{:?}", data.0), "[25.0, 24.958333333333332, 24.89922222222222, 24.79748148148148, 24.69832098765432, 24.673880658436214, 24.855587105624142, 25.65372473708276]");
```
 */
pub fn ema(input: Vec<f64>, period: usize) -> IndicatorsResult<(Vec<f64>, )> {
    if period < 1 {
        return Err(IndicatorsError::InvalidOption("period".to_string()));
    }

    if input.is_empty() {
        return Ok((vec![], ));
    }

    let mut output = vec![];

    let per = 2.0 / (period as f64 + 1.0);
    let mut val = input[0];
    output.push(input[0]);

    for item in input.iter().skip(1) {
        val = (item - val) * per + val;
        output.push(val);
    }

    Ok((output, ))
}
