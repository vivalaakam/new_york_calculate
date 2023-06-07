use crate::indicators::buffer::Buffer;
use crate::indicators::result::{IndicatorsError, IndicatorsResult};

/**
The Stochastic RSI is a momentum oscillator to help identify trends.

https://github.com/TulipCharts/tulipindicators/blob/master/indicators/stochrsi.c
```
use new_york_calculate_core::indicators::stochrsi;

let input_data = vec![37.875,39.5,38.75,39.8125,40.0,39.875,40.1875,41.25,41.125,41.625,41.25,40.1875,39.9375,39.9375,40.5,41.9375,42.25,42.25,41.875,41.875];

let result = stochrsi(input_data, 5);
assert_eq!(result.is_ok(), true);
let data = result.unwrap();
assert_eq!(data.0.len(), 11);
// not ready
// assert_eq!(format!("{:?}", data.0), "[0.9613,0.0000,0.0000,0.0000,0.0000,0.4600,1.0000,1.0000,1.0000,0.3751,0.0000]");
```
 */
pub fn stochrsi(input: Vec<f64>, period: usize) -> IndicatorsResult<(Vec<f64>,)> {
    if period < 2 {
        return Err(IndicatorsError::InvalidOption("kperiod".to_string()));
    }

    if input.len() < period * 2 {
        return Ok((vec![],));
    }

    let per = 1.0 / period as f64;

    let mut output = vec![];

    let mut rsi = Buffer::new(period);
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
    let r = 100.0 * (smooth_up / (smooth_up + smooth_down));
    rsi.push(r);
    println!("r: {}", r);
    let mut min = r;
    let mut max = r;
    let mut mini = 0;
    let mut maxi = 0;

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

        let r = 100.0 * (smooth_up / (smooth_up + smooth_down));

        println!("{}, {}, {}, {}", i, max, min, r);

        let rsi_index = rsi.index;

        if r > max {
            max = r;
            maxi = rsi_index;
        } else if maxi == rsi_index {
            max = r;
            for j in 0..rsi.len {
                if j != rsi_index && rsi.get(j) > max {
                    max = rsi.get(j);
                    maxi = j;
                }
            }
        }

        if r < min {
            min = r;
            mini = rsi_index;
        } else if mini == rsi_index {
            min = r;

            for j in 0..rsi.len {
                if j != rsi_index && rsi.get(j) < min {
                    min = rsi.get(j);
                    mini = j;
                }
            }
        }

        rsi.qpush(r);

        if i > period * 2 - 2 {
            println!("{}, {}, {}, {}", i, max, min, r);
            let diff = max - min;
            if diff == 0.0 {
                output.push(0.0);
            } else {
                output.push((r - min) / (diff));
            }
        }
    }

    Ok((output,))
}
