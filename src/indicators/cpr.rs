use crate::indicators::result::IndicatorsResult;
use crate::{hl2, hlc3};

/**
The central pivotal range (CPR) is the most prominent technical indicators for traders on price. CPR is mostly to identify the stock price movements by indicating crucial price points.
```
use new_york_calculate_core::indicators::cpr;

let high_data = vec![12.3125,12.25,12.3125,12.2812,12.375,12.375,12.3125,12.375,12.5,12.5938,12.3438,12.4062,12.375,12.4062,12.375,12.2812,12.25,12.0625,12.3125,12.3125];
let low_data = vec![12.1875,12.1562,12.1875,12.125,12.125,12.2812,12.1875,12.1875,12.2812,12.3125,12.2812,12.2812,12.2812,12.2812,11.9688,12.0938,12.,11.6562,12.1562,12.125];
let close_data = vec![12.25,12.1875,12.25,12.1875,12.375,12.3125,12.2188,12.3438,12.5,12.375,12.3438,12.3438,12.2812,12.375,12.0938,12.1562,12.0625,11.9688,12.1875,12.1562];

let result = cpr(high_data, low_data, close_data);
assert_eq!(result.is_ok(), true);
let data = result.unwrap();
assert_eq!(data.0.len(), 20);
assert_eq!(format!("{:?}", data.0), "[12.25, 12.197899999999999, 12.25, 12.197899999999999, 12.291666666666666, 12.322899999999999, 12.239600000000001, 12.302100000000001, 12.427066666666667, 12.427100000000001, 12.322933333333333, 12.343733333333333, 12.312466666666666, 12.354133333333332, 12.145866666666668, 12.177066666666667, 12.104166666666666, 11.895833333333334, 12.218733333333333, 12.197899999999999]");
assert_eq!(data.1.len(), 20);
assert_eq!(format!("{:?}", data.1), "[12.25, 12.2031, 12.25, 12.2031, 12.25, 12.3281, 12.25, 12.28125, 12.3906, 12.45315, 12.3125, 12.3437, 12.3281, 12.3437, 12.1719, 12.1875, 12.125, 11.85935, 12.23435, 12.21875]");
assert_eq!(data.2.len(), 20);
assert_eq!(format!("{:?}", data.2), "[12.25, 12.192699999999999, 12.25, 12.192699999999999, 12.333333333333332, 12.317699999999999, 12.229200000000002, 12.322950000000002, 12.463533333333334, 12.401050000000001, 12.333366666666667, 12.343766666666665, 12.296833333333332, 12.364566666666663, 12.119833333333336, 12.166633333333333, 12.083333333333332, 11.932316666666669, 12.203116666666666, 12.177049999999998]");
```
 */
pub fn cpr(
    high: Vec<f64>,
    low: Vec<f64>,
    close: Vec<f64>,
) -> IndicatorsResult<(Vec<f64>, Vec<f64>, Vec<f64>)> {
    let mut pivot = vec![];
    let mut bc = vec![];
    let mut tc = vec![];

    for i in 0..high.len() {
        let pivot_point = hlc3!(high[i], low[i], close[i]);
        let bc_point = hl2!(high[i], low[i]);
        let tc_point = pivot_point - bc_point + pivot_point;
        pivot.push(pivot_point);
        bc.push(bc_point);
        tc.push(tc_point);
    }

    Ok((pivot, bc, tc))
}
