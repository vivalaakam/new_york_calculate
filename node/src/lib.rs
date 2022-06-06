mod get_id;

use crate::get_id::get_applicant_id;
use neon::prelude::*;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("get_applicant_id", get_applicant_id)?;
    Ok(())
}
