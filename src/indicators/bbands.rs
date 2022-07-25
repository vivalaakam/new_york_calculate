use crate::indicators::result::{IndicatorsError, IndicatorsResult};

/**


```
use new_york_calculate_core::indicators::bbands;

let raw_data = vec![31.875,32.125,32.3125,32.125,31.875,32.3125,32.25,32.4375,32.8125,32.375,32.5,32.4375,32.75,33.1875,33.0625,33.0625,33.125,33.0625,32.8125,32.875,33.25,33.125];

let result = bbands(raw_data, 5, 2.0);
assert_eq!(result.is_ok(), true);
let data = result.unwrap();
assert_eq!(data.0.len(), 18);
assert_eq!(format!("{:?}", data.0), "[31.72708980337476, 31.827897531831532, 31.84522735104544, 31.81757353648083, 31.732314078153706, 32.04221529247895, 32.099167590541114, 32.202258158850874, 32.226790293071495, 32.05523113733054, 32.19273113733207, 32.354564394267676, 32.73542385066098, 33.00000000000346, 32.80705505282163, 32.74511600713076, 32.70289753183153, 32.70289753183153]");

assert_eq!(data.1.len(), 18);
assert_eq!(format!("{:?}", data.1), "[32.0625, 32.15, 32.175000000000004, 32.2, 32.3375, 32.4375, 32.475, 32.5125, 32.575, 32.65, 32.7875, 32.9, 33.0375, 33.1, 33.025, 32.987500000000004, 33.025, 33.025]");

assert_eq!(data.2.len(), 18);
assert_eq!(format!("{:?}", data.2), "[32.39791019662524, 32.472102468168465, 32.504772648954564, 32.58242646351918, 32.94268592184629, 32.83278470752105, 32.85083240945889, 32.82274184114913, 32.92320970692851, 33.24476886266946, 33.382268862667935, 33.44543560573232, 33.339576149339024, 33.19999999999654, 33.242944947178366, 33.22988399286925, 33.347102468168465, 33.347102468168465]");
```
 */
pub fn bbands(
    input: Vec<f64>,
    period: usize,
    stddev: f64,
) -> IndicatorsResult<(Vec<f64>, Vec<f64>, Vec<f64>)> {
    let mut lower = vec![];
    let mut middle = vec![];
    let mut upper = vec![];

    if period < 1 {
        return Err(IndicatorsError::InvalidOption("period".to_string()));
    }

    if input.len() < period - 1 {
        return Ok((lower, middle, upper));
    }

    let scale = 1.0 / period as f64;

    let mut sum = 0.0;
    let mut sum2 = 0.0;

    for i in 0..period {
        sum += input[i];
        sum2 += input[i] * input[i];
    }

    let mut sd = (sum2 * scale - (sum * scale) * (sum * scale)).sqrt();
    let mut m = sum * scale;
    middle.push(m);
    lower.push(m - stddev * sd);
    upper.push(m + stddev * sd);

    for i in period..input.len() {
        sum += input[i];
        sum2 += input[i] * input[i];

        sum -= input[i - period];
        sum2 -= input[i - period] * input[i - period];

        sd = (sum2 * scale - (sum * scale) * (sum * scale)).sqrt();
        m = sum * scale;
        middle.push(m);
        lower.push(m - stddev * sd);
        upper.push(m + stddev * sd);
    }

    Ok((lower, middle, upper))
}
