use crate::indicators::result::{IndicatorsError, IndicatorsResult};

/**
The Volume Weighted Moving Average is simalair to a Simple Moving Average, but it weights each bar by its volume.

https://github.com/TulipCharts/tulipindicators/blob/master/indicators/vwma.c
```
use new_york_calculate_core::indicators::vwma;

let close_data = vec![81.59,81.06,82.87,83.00,83.61,83.15,82.84,83.99,84.55,84.36,85.53,86.54,86.89,87.77,87.29];
let volume_data = vec![5653100.0,6447400.0,7690900.0,3831400.0,4455100.0,3798000.0,3936200.0,4732000.0,4841300.0,3915300.0,6830800.0,6694100.0,5293600.0,7985800.0,4807900.0];

let result = vwma(close_data, volume_data, 5);
assert_eq!(result.is_ok(), true);

let data = result.unwrap();
assert_eq!(data.0.len(), 11);
assert_eq!(format!("{:?}", data.0), "[82.33182207358813, 82.61024520646156, 83.06991101401846, 83.3537948797024, 83.68218935237518, 83.82238964698344, 84.40856688764656, 85.16530775353064, 85.69808504774235, 86.41762965663615, 86.80515430751419]");
```
 */
pub fn vwma(input: Vec<f64>, volume: Vec<f64>, period: usize) -> IndicatorsResult<(Vec<f64>,)> {
    if period < 1 {
        return Err(IndicatorsError::InvalidOption("period".to_string()));
    }

    if input.len() < period - 1 {
        return Ok((vec![],));
    }
    let mut output = vec![];

    let mut sum = 0.0;
    let mut vsum = 0.0;

    for i in 0..period {
        sum += input[i] * volume[i];
        vsum += volume[i];
    }

    output.push(sum / vsum);

    for i in period..input.len() {
        sum += input[i] * volume[i];
        sum -= input[i - period] * volume[i - period];
        vsum += volume[i];
        vsum -= volume[i - period];

        output.push(sum / vsum);
    }

    Ok((output,))
}
