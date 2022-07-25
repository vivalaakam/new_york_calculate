use crate::indicators::result::{IndicatorsError, IndicatorsResult};

/**
The Average Directional Movement Index can help determine trend strength.

https://github.com/TulipCharts/tulipindicators/blob/master/indicators/adx.c
```
use new_york_calculate_core::indicators::adx;

let high_data = vec![94.1875,94.5,93.5,92.75,92.875,90.75,89.875,89.125,90.4375,90.0,88.5,87.75,87.0625,85.8125,86.5625,90.375,91.375,92.25,93.375,92.0625,92.875,93.9375,95.25,97.125,97.1875,94.875,94.3125,93.3125,94.125,96.9375,101.125,108.75,115.0,117.125,115.0,116.625,118.0,119.25,119.25,118.812,118.375,119.938,117.75,118.625,117.125,116.375,113.875,112.25,113.688,114.25];
let low_data = vec![92.125,91.9375,91.5,90.3125,90.5,84.375,86.4375,86.4375,88.25,87.0625,86.9375,85.875,85.0,84.5,84.375,88.4375,88.375,89.5,91.0,89.5,89.5625,90.875,92.875,95.7344,94.75,92.875,91.6875,91.4375,92.25,92.75,95.3125,98.5,108.938,113.625,111.188,110.625,115.125,116.75,116.125,117.062,116.812,117.125,116.25,112.0,112.25,109.375,108.375,107.312,111.375,108.688];

let result = adx(high_data, low_data, 14);
assert_eq!(result.is_ok(), true);

let data = result.unwrap();
assert_eq!(data.0.len(), 24);
assert_eq!(format!("{:?}", data.0), "[18.479823237258355, 17.73293035170528, 16.640161573107605, 16.46076006072464, 17.55698666763429, 19.97582464311788, 22.924470438671687, 25.853503532342618, 27.65358048690529, 29.51174780116884, 31.390735315999777, 33.27263151751787, 34.76253187746976, 36.14601078313937, 37.31513645118196, 38.624593306665794, 39.415112826863314, 38.36601367364624, 37.39185017423039, 35.45647795188519, 33.33208769575989, 31.016665549944133, 29.30563974612901, 27.556556457980257]");
```
 */
pub fn adx(high: Vec<f64>, low: Vec<f64>, period: usize) -> IndicatorsResult<(Vec<f64>,)> {
    if period < 2 {
        return Err(IndicatorsError::InvalidOption("period".to_string()));
    }

    if high.len() <= (period - 1) * 2 {
        return Ok((vec![],));
    }

    let mut output = vec![];

    let per = (period as f64 - 1.0) / period as f64;
    let invper = 1.0 / period as f64;

    let mut dmup = 0.0;
    let mut dmdown = 0.0;

    for i in 1..period {
        let mut up = high[i] - high[i - 1];
        let mut down = low[i - 1] - low[i];

        if up < 0.0 {
            up = 0.0;
        } else if up > down {
            down = 0.0;
        }
        if down < 0.0 {
            down = 0.0;
        } else if down > up {
            up = 0.0;
        }
        dmup += up;
        dmdown += down;
    }

    let mut adx = (dmup - dmdown).abs() / (dmup + dmdown) * 100.0;

    for i in period..high.len() {
        let mut up = high[i] - high[i - 1];
        let mut down = low[i - 1] - low[i];

        if up < 0.0 {
            up = 0.0;
        } else if up > down {
            down = 0.0;
        }
        if down < 0.0 {
            down = 0.0;
        } else if down > up {
            up = 0.0;
        }

        dmup = dmup * per + up;
        dmdown = dmdown * per + down;

        let dx = (dmup - dmdown).abs() / (dmup + dmdown) * 100.0;

        if i - period <= period - 2 {
            adx += dx;
        } else {
            adx = adx * per + dx;
        }

        if i - period >= period - 2 {
            output.push(adx * invper)
        }
    }

    Ok((output,))
}
