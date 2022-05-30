use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::calculate::Calculate;
use crate::py_candle::PyCandle;

#[pyclass]
pub struct PyCalculateV1 {
    instance: Calculate,
}

#[pymethods]
impl PyCalculateV1 {
    #[new]
    fn __new__(
        candles: &PyList,
        initial_balance: Option<f32>,
        stake: Option<f32>,
        gain: Option<f32>,
        profit: Option<f32>,
        interval: Option<u64>,
    ) -> Self {
        let mut candles_map = HashMap::new();

        let candles = candles.extract::<Vec<PyCandle>>().expect("Expected a candle");

        for candle in candles {
            candles_map.insert(candle.start_time, candle.into());
        }

        PyCalculateV1 {
            instance: Calculate::new(
                candles_map,
                initial_balance,
                stake,
                gain,
                profit,
                interval,
            ),
        }
    }

    pub fn calculate(&self, results: &PyDict) -> PyResult<PyObject> {
        let results = results
            .extract::<HashMap<u64, bool>>()
            .expect("Expected a candle");

        let result = self.instance.calculate(results);

        let gil = Python::acquire_gil();
        let py = gil.python();

        Ok(result.into_py(py))
    }
}
