use crate::indicators::result::IndicatorsResult;

/**


```
use new_york_calculate_core::indicators::vpt;

let close_data = vec![20.5625,20.375,20.0625,19.5,19.8125,19.8125,20.0625,20.0625,20.375,20.75];
let volume_data = vec![27802.0,16178.0,22766.0,46074.0,22904.0,20428.0,29260.0,30652.0,38332.0,40054.0];

let result = vpt(close_data, volume_data);
assert_eq!(result.is_ok(), true);

let data = result.unwrap();
assert_eq!(data.0.len(), 9);
assert_eq!(format!("{:?}", data.0), "[-147.51975683890578, -496.6915359800101, -1788.4859285033745, -1421.4346464520925, -1421.4346464520925, -1052.2232899852156, -1052.2232899852156, -455.1516388948729, 282.0385451542069]");
```
 */
pub fn vpt(close: Vec<f64>, volume: Vec<f64>) -> IndicatorsResult<(Vec<f64>,)> {
    let mut output = vec![];
    let mut vpt = 0.0;
    for i in 1..close.len() {
        vpt += volume[i] * (close[i] - close[i - 1]) / close[i - 1];
        output.push(vpt);
    }

    Ok((output,))
}
