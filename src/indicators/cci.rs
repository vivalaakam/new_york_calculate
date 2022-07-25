use crate::hlc3;
use crate::indicators::buffer::Buffer;
use crate::indicators::result::{IndicatorsError, IndicatorsResult};

/**
The Commodity Channel Index indicator is used to detect trends. It works by taking a Simple Moving Average of the Typical Price and comparing it to the amount of volatility in Typical Price.

https://github.com/TulipCharts/tulipindicators/blob/master/indicators/cci.c
```
use new_york_calculate_core::indicators::cci;

let high_data = vec![15.125,15.052,14.8173,14.69,14.7967,14.794,14.093,14.7,14.5255,14.6579,14.7842,14.8273];
let low_data = vec![14.936,14.6267,14.5557,14.46,14.5483,13.9347,13.8223,14.02,14.2652,14.3773,14.5527,14.3309];
let close_data = vec![14.936,14.752,14.5857,14.6,14.6983,13.946,13.9827,14.45,14.3452,14.4197,14.5727,14.4773];

let result = cci(high_data, low_data, close_data, 5);
assert_eq!(result.is_ok(), true);
let data = result.unwrap();
assert_eq!(data.0.len(), 4);
assert_eq!(format!("{:?}", data.0), "[18.089002860429975, 84.46052669366142, 109.1186277090461, 46.654034321440264]");
```
 */
pub fn cci(
    high: Vec<f64>,
    low: Vec<f64>,
    close: Vec<f64>,
    period: usize,
) -> IndicatorsResult<(Vec<f64>,)> {
    if period < 1 {
        return Err(IndicatorsError::InvalidOption("period".to_string()));
    }

    if high.len() < (period - 1) * 2 {
        return Ok((vec![],));
    }

    let scale = 1.0 / period as f64;
    let mut output = vec![];

    let mut sum = Buffer::new(period);
    let size = high.len();

    for i in 0..size {
        let today = hlc3!(high[i], low[i], close[i]);
        sum.push(today);
        let avg = sum.sum * scale;

        if i >= period * 2 - 2 {
            let mut acc = 0.0;
            for j in 0..period {
                acc += (avg - sum.get(j)).abs();
            }

            let cci = (today - avg) / (acc * scale * 0.015);
            output.push(cci)
        }
    }
    Ok((output,))
}
