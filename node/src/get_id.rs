use neon::prelude::*;

use new_york_calculate_core;

pub fn get_applicant_id(mut cx: FunctionContext) -> JsResult<JsString> {
    let interval = cx.argument::<JsString>(0).unwrap().value(&mut cx);
    let start = cx.argument::<JsString>(1).unwrap().value(&mut cx);
    let end = cx.argument::<JsString>(2).unwrap().value(&mut cx);
    let model_id = cx.argument::<JsString>(3).unwrap().value(&mut cx);

    Ok(cx.string(new_york_calculate_core::get_id::get_applicant_id(interval, start, end, model_id).as_str()))
}
