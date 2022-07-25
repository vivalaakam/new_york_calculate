use crate::indicators::result::{IndicatorsError, IndicatorsResult};

/**

```
use new_york_calculate_core::indicators::mom;

let close_data = vec![81.59,81.06,82.87,83.00,83.61,83.15,82.84,83.99,84.55,84.36,85.53,86.54,86.89,87.77,87.29];

let result = mom(close_data, 5);
assert_eq!(result.is_ok(), true);

let data = result.unwrap();
assert_eq!(data.0.len(), 10);
assert_eq!(format!("{:?}", data.0), "[1.5600000000000023, 1.7800000000000011, 1.1199999999999903, 1.5499999999999972, 0.75, 2.3799999999999955, 3.700000000000003, 2.9000000000000057, 3.219999999999999, 2.930000000000007]");
```
*/
pub fn mom(input: Vec<f64>, period: usize) -> IndicatorsResult<(Vec<f64>,)> {
    if period < 1 {
        return Err(IndicatorsError::InvalidOption("period".to_string()));
    }

    if input.len() < period {
        return Ok((vec![],));
    }

    let mut output = vec![];

    for i in period..input.len() {
        output.push(input[i] - input[i - period]);
    }

    Ok((output,))
}
