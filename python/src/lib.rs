use pyo3::prelude::*;
use pyo3_asyncio;

use new_york_calculate_core;

use crate::py_calculate::PyCalculate;
use crate::py_candle::PyCandle;

mod py_calculate;
mod py_candle;

#[pyfunction]
pub fn get_candles(py: Python<'_>, ticker: String, period: usize, start_time: u64, look_back: usize) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let candles = new_york_calculate_core::get_candles(ticker, period, start_time, look_back).await;
        Ok(candles.into_iter().map(|c| c.into()).collect::<Vec<PyCandle>>())
    })
}

#[pyfunction]
pub fn get_applicant_id(interval: String, start: String, end: String, model_id: String) -> String {
    new_york_calculate_core::get_id::get_applicant_id(interval, start, end, model_id)
}

#[pymodule]
pub fn new_york_calculate(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyCalculate>()?;
    module.add_class::<PyCandle>()?;
    module.add_function(wrap_pyfunction!(get_applicant_id, module)?)?;
    module.add_function(wrap_pyfunction!(get_candles, module)?)?;
    Ok(())
}
