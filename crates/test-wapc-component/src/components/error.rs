use wapc_guest::prelude::*;

use crate::generated::error::*;

pub(crate) fn job(_input: Inputs, _output: Outputs) -> HandlerResult<()> {
  panic!("This WaPC component will always panic")
}
