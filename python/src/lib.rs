use pyo3::prelude::*;

use new_york_calculate_core;

use crate::py_calculate::PyCalculate;
use crate::py_calculate_v1::PyCalculateV1;
use crate::py_calculate_v2::PyCalculateV2;

mod py_calculate_v1;
mod py_calculate_v2;
mod py_candle;
mod py_calculate;

#[pyfunction]
pub fn get_applicant_id(interval: String, start: String, end: String, model_id: String) -> String {
    new_york_calculate_core::get_id::get_applicant_id(interval, start, end, model_id)
}

#[pymodule]
pub fn new_york_calculate(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyCalculate>()?;
    module.add_class::<PyCalculateV1>()?;
    module.add_class::<PyCalculateV2>()?;
    module.add_function(wrap_pyfunction!(get_applicant_id, module)?)?;
    Ok(())
}
