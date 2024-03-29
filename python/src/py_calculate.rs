use new_york_calculate_core::{Calculate, Candle};
use pyo3::prelude::*;
use pyo3::types::PyList;
use crate::py_calculate_result::PyCalculateResult;

use crate::py_candle::PyCandle;

#[pyclass]
pub struct PyCalculate {
    instance: Calculate,
}

#[pymethods]
impl PyCalculate {
    #[new]
    fn __new__(
        candles: &PyList,
        initial_balance: Option<f64>,
        stake: Option<f64>,
        gain: Option<f64>,
        profit: Option<f64>,
    ) -> Self {
        let mut candles = candles
            .extract::<Vec<PyCandle>>()
            .expect("Expected a candle")
            .into_iter()
            .map(|c| c.into())
            .collect::<Vec<Candle>>();

        candles.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());

        PyCalculate {
            instance: Calculate::new(candles, initial_balance, stake, gain, profit),
        }
    }

    pub fn calculate(&self, results: &PyList) -> PyResult<PyCalculateResult> {
        let results = results.extract::<Vec<u8>>().expect("Expected a candle");

        let result = self.instance.calculate(results);

        Ok(result.into())
    }
}
