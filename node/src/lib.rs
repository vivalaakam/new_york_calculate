#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use new_york_calculate_core;


#[napi]
fn get_applicant_id(interval: String, start: String, end: String, model_id: String) -> String {
    new_york_calculate_core::get_id::get_applicant_id(interval, start, end, model_id)
}
