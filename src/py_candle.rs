use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::candle::Candle;

#[pyclass]
pub struct PyCandle {
    #[pyo3(get)]
    pub start_time: u64,
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
}

impl FromPyObject<'_> for PyCandle {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let keypair = ob.downcast::<PyList>().expect("Expected a tuple");
        let candle = PyCandle {
            start_time: *&keypair.get_item(0).extract::<u64>().expect("Expected u64"),
            open: *&keypair.get_item(1).extract::<f64>().expect("Expected u64"),
            high: *&keypair.get_item(2).extract::<f64>().expect("Expected u64"),
            low: *&keypair.get_item(3).extract::<f64>().expect("Expected u64"),
            close: *&keypair.get_item(4).extract::<f64>().expect("Expected u64"),
            volume: *&keypair.get_item(5).extract::<f64>().expect("Expected u64"),
        };

        Ok(candle)
    }
}

impl Into<Candle> for PyCandle {
    fn into(self) -> Candle {
        Candle {
            start_time: self.start_time,
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
        }
    }
}
