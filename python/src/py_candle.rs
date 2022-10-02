use pyo3::prelude::*;

use new_york_calculate_core::Candle;

#[pyclass]
#[derive(Default, FromPyObject)]
pub struct PyCandle {
    #[pyo3(get)]
    pub start_time: u64,
    #[pyo3(get)]
    pub end_time: u64,
    #[pyo3(get)]
    pub open: f64,
    #[pyo3(get)]
    pub high: f64,
    #[pyo3(get)]
    pub low: f64,
    #[pyo3(get)]
    pub close: f64,
    #[pyo3(get)]
    pub volume: f64,
    #[pyo3(get)]
    pub hist: Vec<f64>,
    #[pyo3(get)]
    pub shape: Vec<usize>,
    #[pyo3(get)]
    pub max_profit_12: f64,
    #[pyo3(get)]
    pub max_profit_24: f64,
}

impl Into<Candle> for PyCandle {
    fn into(self) -> Candle {
        Candle {
            start_time: self.start_time,
            end_time: self.end_time,
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
            ..Candle::default()
        }
    }
}

impl Into<PyCandle> for Candle {
    fn into(self) -> PyCandle {
        PyCandle {
            start_time: self.start_time,
            end_time: self.end_time,
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
            hist: self.history,
            shape: self.shape,
            max_profit_12: self.max_profit_12,
            max_profit_24: self.max_profit_24,
        }
    }
}

#[pymethods]
impl PyCandle {
    #[new]
    pub fn new(params: (u64, f64, f64, f64, f64, f64, u64)) -> Self {
        PyCandle {
            start_time: params.0,
            end_time: params.6,
            open: params.1,
            high: params.2,
            low: params.3,
            close: params.4,
            volume: params.5,
            ..PyCandle::default()
        }
    }
}
