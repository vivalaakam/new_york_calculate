use crate::indicators::buffer::Buffer;
use crate::indicators::result::{IndicatorsError, IndicatorsResult};

/**
Hull Moving Average modifies Weighted Moving Average to greatly reduce lag.

https://github.com/TulipCharts/tulipindicators/blob/master/indicators/hma.c
```
use new_york_calculate_core::indicators::hma;

let close_data = vec![81.59,81.06,82.87,83.00,83.61,83.15,82.84,83.99,84.55,84.36,85.53,86.54,86.89,87.77,87.29];

let result = hma(close_data, 5);
assert_eq!(result.is_ok(), true);

let data = result.unwrap();
assert_eq!(data.0.len(), 10);
assert_eq!(format!("{:?}", data.0), "[83.68999999999998, 83.03799999999997, 83.47199999999997, 84.54977777777775, 84.83466666666666, 85.35955555555557, 86.55244444444448, 87.34600000000005, 87.96511111111117, 87.91622222222229]");
```
 */
pub fn hma(input: Vec<f64>, period: usize) -> IndicatorsResult<(Vec<f64>,)> {
    if period < 1 {
        return Err(IndicatorsError::InvalidOption("period".to_string()));
    }

    if input.len() < period + (period as f64).sqrt() as usize - 2 {
        return Ok((vec![],));
    }

    let mut output = vec![];

    let period2 = period / 2;
    let periodsqrt = (period as f64).sqrt() as usize;

    let weights = period * (period + 1) / 2;
    let weights2 = period2 * (period2 + 1) / 2;
    let weightssqrt = periodsqrt * (periodsqrt + 1) / 2;

    let mut sum = 0.0;
    let mut weight_sum = 0.0;

    let mut sum2 = 0.0;
    let mut weight_sum2 = 0.0;

    let mut sumsqrt = 0.0;
    let mut weight_sumsqrt = 0.0;

    for i in 0..period - 1 {
        weight_sum += input[i] * (i + 1) as f64;
        sum += input[i];

        if i >= period - period2 {
            weight_sum2 += input[i] * (i + 1 - (period - period2)) as f64;
            sum2 += input[i];
        }
    }

    let mut buff = Buffer::new(periodsqrt);

    for i in period - 1..input.len() {
        weight_sum += input[i] * period as f64;
        sum += input[i];

        weight_sum2 += input[i] * period2 as f64;
        sum2 += input[i];

        let diff = 2.0 * (weight_sum2 / weights2 as f64) - (weight_sum / weights as f64);

        weight_sumsqrt += diff * periodsqrt as f64;
        sumsqrt += diff;

        buff.push(diff);

        if i >= (period - 1) + (periodsqrt - 1) {
            output.push(weight_sumsqrt / weightssqrt as f64);

            weight_sumsqrt -= sumsqrt;
            sumsqrt -= buff.get(1);
        } else {
            weight_sumsqrt -= sumsqrt;
        }

        weight_sum -= sum;
        sum -= input[i - (period - 1)];

        weight_sum2 -= sum2;
        sum2 -= input[i - period2 + 1];
    }

    Ok((output,))
}
