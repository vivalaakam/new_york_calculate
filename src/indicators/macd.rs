use crate::indicators::result::{IndicatorsError, IndicatorsResult};

/**
Moving Average Convergence/Divergence helps follow trends and has several uses.

It takes three parameter, a short period n, a long period m, and a signal period p.

https://github.com/TulipCharts/tulipindicators/blob/master/indicators/macd.c
```
use new_york_calculate_core::indicators::macd;

let raw_data = vec![63.75,63.625,63.0,62.75,63.25,65.375,66.0,65.0,64.875,64.75,64.375,64.375,64.625,64.375,64.5,65.25,67.875,68.0,66.875,66.25,65.875,66.0,65.875,64.75,63.0,63.375,63.375,63.375,63.875,65.5,63.25,60.75,57.25,59.125,59.25,58.5,59.125,59.75,60.625,60.5,59.0,59.5,58.875,59.625,59.875,59.75,59.625,59.25,58.875,59.125,60.875,60.75,61.125,62.5,63.25];

let result = macd(raw_data,12,26,9);
assert_eq!(result.is_ok(), true);
let data = result.unwrap();
assert_eq!(data.0.len(), 30);
assert_eq!(format!("{:?}", data.0), "[0.06924617252803955, -0.05674936092243854, -0.1551749187875089, -0.19331629582255516, -0.09925514518836565, -0.19293294511885506, -0.4519166196816258, -0.9129584718351751, -1.1245568452827612, -1.2688998018964952, -1.4243643287627066, -1.4836992143126864, -1.46678465191534, -1.3713592498963791, -1.29027823599764, -1.32451265866478, -1.2990287060768395, -1.3112528754112063, -1.2498625337019789, -1.1687834240289519, -1.1012611605282032, -1.0451575927948014, -1.0174131397454715, -1.0122781657132975, -0.9781026630162302, -0.808978519061732, -0.6762786525380591, -0.5362102476427708, -0.31692409900787055, -0.08469496852983838]");
assert_eq!(data.1.len(), 30);
assert_eq!(format!("{:?}", data.1), "[0.06924617252803955, 0.04404706583794393, 0.004202668912853369, -0.03530112403422834, -0.0480919282650558, -0.07706013163581565, -0.1520314292449777, -0.3042168377630172, -0.46828483926696607, -0.6284078317928719, -0.7875991311868388, -0.9268191478120082, -1.0348122486326745, -1.1021216488854155, -1.1397529663078605, -1.1767049047792444, -1.2011696650387633, -1.2231863071132518, -1.2285215524309971, -1.2165739267505882, -1.1935113735061111, -1.1638406173638491, -1.1345551218401737, -1.1100997306147984, -1.0837003170950847, -1.0287559574884142, -0.9582604964983432, -0.8738504467272288, -0.7624651771833572, -0.6269111354526534]");
assert_eq!(data.2.len(), 30);
assert_eq!(format!("{:?}", data.2), "[0.0, -0.10079642676038247, -0.15937758770036226, -0.15801517178832683, -0.05116321692330984, -0.1158728134830394, -0.2998851904366481, -0.6087416340721579, -0.6562720060157952, -0.6404919701036234, -0.6367651975758678, -0.5568800665006781, -0.4319724032826655, -0.2692376010109636, -0.15052526968977942, -0.14780775388553558, -0.09785904103807619, -0.08806656829795445, -0.021340981270981718, 0.04779050272163632, 0.09225021297790792, 0.11868302456904778, 0.11714198209470217, 0.09782156490150085, 0.10559765407885457, 0.21977743842668218, 0.28198184396028414, 0.33764019908445797, 0.4455410781754866, 0.542216166922815]");
```

 */
pub fn macd(
    input: Vec<f64>,
    short_period: usize,
    long_period: usize,
    signal_period: usize,
) -> IndicatorsResult<(Vec<f64>, Vec<f64>, Vec<f64>)> {
    if short_period < 1 {
        return Err(IndicatorsError::InvalidOption("short_period".to_string()));
    }

    if long_period < 1 {
        return Err(IndicatorsError::InvalidOption("long_period".to_string()));
    }

    if long_period < short_period {
        return Err(IndicatorsError::InvalidOption("long_period".to_string()));
    }

    if signal_period < 1 {
        return Err(IndicatorsError::InvalidOption("signal_period".to_string()));
    }

    if input.len() < long_period {
        return Ok((vec![], vec![], vec![]));
    }

    let mut macd = vec![];
    let mut signal = vec![];
    let mut hist = vec![];

    let mut short_per = 2.0 / (short_period as f64 + 1.0);
    let mut long_per = 2.0 / (long_period as f64 + 1.0);
    let signal_per = 2.0 / (signal_period as f64 + 1.0);

    if short_period == 12 && long_period == 26 {
        short_per = 0.15;
        long_per = 0.075;
    }

    let mut short_ema = input[0];
    let mut long_ema = input[0];
    let mut signal_ema = 0.0;
    for (i, val) in input.iter().enumerate().skip(1) {
        short_ema = (val - short_ema) * short_per + short_ema;
        long_ema = (val - long_ema) * long_per + long_ema;
        let out = short_ema - long_ema;

        if i == long_period - 1 {
            signal_ema = out;
        }
        if i >= long_period - 1 {
            signal_ema = (out - signal_ema) * signal_per + signal_ema;

            macd.push(out);
            signal.push(signal_ema);
            hist.push(out - signal_ema)
        }
    }

    Ok((macd, signal, hist))
}
