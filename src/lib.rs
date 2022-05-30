use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::py_calculate_v1::PyCalculateV1;

mod calculate;
mod candle;
mod py_candle;
mod order;
pub mod utils;
mod py_calculate_v1;

#[pymodule]
pub fn new_york_calculate(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyCalculateV1>()?;
    Ok(())
}
