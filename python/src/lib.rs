use pyo3::prelude::*;

use crate::py_calculate::PyCalculate;
use crate::py_calculate_v1::PyCalculateV1;
use crate::py_calculate_v2::PyCalculateV2;

mod py_calculate_v1;
mod py_calculate_v2;
mod py_candle;
mod py_calculate;

#[pymodule]
pub fn new_york_calculate(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyCalculate>()?;
    module.add_class::<PyCalculateV1>()?;
    module.add_class::<PyCalculateV2>()?;
    Ok(())
}
