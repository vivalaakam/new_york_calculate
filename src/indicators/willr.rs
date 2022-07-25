use crate::indicators::result::{IndicatorsError, IndicatorsResult};

/**
```
use new_york_calculate_core::indicators::willr;

let high_data = vec![82.15,81.89,83.03,83.30,83.85,83.90,83.33,84.30,84.84,85.00,85.90,86.58,86.98,88.00,87.87];
let low_data = vec![81.29,80.64,81.31,82.65,83.07,83.11,82.49,82.30,84.15,84.11,84.03,85.39,85.76,87.17,87.01];
let close_data = vec![81.59,81.06,82.87,83.00,83.61,83.15,82.84,83.99,84.55,84.36,85.53,86.54,86.89,87.77,87.29];

let result = willr(high_data, low_data, close_data, 5);
assert_eq!(result.is_ok(), true);

let data = result.unwrap();
assert_eq!(data.0.len(), 11);
assert_eq!(format!("{:?}", data.0), "[-9.374999999999844, -23.006134969325117, -40.926640926640964, -15.500000000000114, -11.417322834645887, -23.7037037037037, -10.27777777777788, -0.9345794392521503, -3.0508474576272313, -5.793450881612192, -17.884130982367605]");

```
 */
pub fn willr(
    high: Vec<f64>,
    low: Vec<f64>,
    close: Vec<f64>,
    period: usize,
) -> IndicatorsResult<(Vec<f64>,)> {
    if period < 1 {
        return Err(IndicatorsError::InvalidOption("period".to_string()));
    }

    if high.len() < period - 1 {
        return Ok((vec![],));
    }

    let mut trail = 0;
    let mut maxi = 0;
    let mut mini = 0;
    let mut max = high[0];
    let mut min = low[0];
    let mut output = vec![];

    for i in period - 1..high.len() {
        /* Maintain highest. */
        let mut bar = high[i];
        if maxi < trail {
            maxi = trail;
            max = high[maxi];
            let mut j = trail;
            while j < i {
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

        /* Maintain lowest. */
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

        let r = if (max - min) == 0.0 {
            0.0
        } else {
            -100.0 * ((max - close[i]) / (max - min))
        };
        output.push(r);
        trail += 1;
    }

    Ok((output,))
}
