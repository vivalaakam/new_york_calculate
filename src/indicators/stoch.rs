use crate::indicators::buffer::Buffer;
use crate::indicators::result::{IndicatorsError, IndicatorsResult};

/**
The Stochastic Oscillator indicator calculates two values, %k and %d.

https://github.com/TulipCharts/tulipindicators/blob/master/indicators/stoch.c
```
use new_york_calculate_core::indicators::stoch;

let high_data = vec![34.3750,34.7500,34.2188,33.8281,33.4375,33.4688,34.3750,34.7188,34.6250,34.9219,34.9531,35.0625,34.7812,34.3438,34.5938,34.3125,34.2500,34.1875,33.7812,33.8125,33.9688,33.8750,34.0156,33.5312];
let low_data = vec![33.5312,33.9062,33.6875,33.2500,33.0000,32.9375,33.2500,34.0469,33.9375,34.0625,34.4375,34.5938,33.7656,33.2188,33.9062,32.6562,32.7500,33.1562,32.8594,33.0000,33.2969,33.2812,33.0312,33.0156];
let close_data = vec![34.3125,34.1250,33.7500,33.6406,33.0156,33.0469,34.2969,34.1406,34.5469,34.3281,34.8281,34.8750,33.7812,34.2031,34.4844,32.6719,34.0938,33.2969,33.0625,33.7969,33.3281,33.8750,33.1094,33.1875];

let result = stoch(high_data, low_data, close_data, 5, 3, 3);
assert_eq!(result.is_ok(), true);
let data = result.unwrap();
assert_eq!(data.0.len(), 16);
assert_eq!(format!("{:?}", data.0), "[84.15242045176251, 75.98896743124592, 84.36226158361929, 82.02345783853924, 59.06554814462305, 45.974470737465936, 41.07821743040827, 40.89474596258881, 45.649597364341766, 33.790299703420736, 40.562561810851605, 40.96876481707973, 42.79320467394729, 61.29350634831644, 45.54423399049362, 38.85156496069189]");
assert_eq!(data.1.len(), 16);
assert_eq!(format!("{:?}", data.1), "[58.010543041455605, 72.06306027830054, 81.50121648887588, 80.79156228446813, 75.1504225222605, 62.35449224020938, 48.70607877083239, 42.64914471015431, 42.54085358577959, 40.111547676783744, 40.000819626204674, 38.440542110450664, 41.441510433959515, 48.351825279781124, 49.87698167091909, 48.56310176650062]");

```
 */
pub fn stoch(
    high: Vec<f64>,
    low: Vec<f64>,
    close: Vec<f64>,
    kperiod: usize,
    kslow: usize,
    dperiod: usize,
) -> IndicatorsResult<(Vec<f64>, Vec<f64>)> {
    if kperiod < 1 {
        return Err(IndicatorsError::InvalidOption("kperiod".to_string()));
    }
    if kslow < 1 {
        return Err(IndicatorsError::InvalidOption("kslow".to_string()));
    }
    if dperiod < 1 {
        return Err(IndicatorsError::InvalidOption("dperiod".to_string()));
    }

    if high.len() <= (kperiod + kslow + dperiod - 3) {
        return Ok((vec![], vec![]));
    }

    let kper = 1.0 / kslow as f64;
    let dper = 1.0 / dperiod as f64;

    let mut stoch = vec![];
    let mut stoch_ma = vec![];

    let mut trail = 0;
    let mut maxi = 0;
    let mut mini = 0;
    let mut max = high[0];
    let mut min = low[0];

    let mut k_sum = Buffer::new(kslow);
    let mut d_sum = Buffer::new(dperiod);

    for i in 1..high.len() {
        if i >= kperiod {
            trail += 1;
        }

        let mut bar = high[i];
        if maxi < trail {
            maxi = trail;
            max = high[maxi];
            let mut j = trail;
            while j <= i {
                bar = high[j];
                if bar >= max {
                    max = bar;
                    maxi = j;
                }
                j += 1;
            }
        } else if bar >= max {
            maxi = i;
            max = bar;
        }

        let mut bar = low[i];
        if mini < trail {
            mini = trail;
            min = low[mini];
            let mut j = trail;
            while j <= i {
                bar = low[j];
                if bar <= min {
                    min = bar;
                    mini = j;
                }
                j += 1;
            }
        } else if bar <= min {
            mini = i;
            min = bar;
        }

        let kdiff = max - min;
        let kfast = if kdiff == 0.0 {
            0.0
        } else {
            100.0 * ((close[i] - min) / kdiff)
        };
        k_sum.push(kfast);

        if i >= kperiod - 1 + kslow - 1 {
            let k = k_sum.sum * kper;
            d_sum.push(k);

            if i >= kperiod - 1 + kslow - 1 + dperiod - 1 {
                stoch.push(k);
                stoch_ma.push(d_sum.sum * dper)
            }
        }
    }

    Ok((stoch, stoch_ma))
}
