use crate::dev::prelude::*;
use crate::models::validator::Validator;

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<SchematicSignature, SchematicError>")]
pub(crate) struct GetSignature {}

impl Handler<GetSignature> for SchematicService {
  type Result = Result<SchematicSignature, SchematicError>;

  fn handle(&mut self, _msg: GetSignature, _ctx: &mut Context<Self>) -> Self::Result {
    let state = self.get_state_mut();
    let mut model = state.model.lock();
    Validator::validate_final_errors(&model)?;
    model.final_initialization()?;

    Ok(SchematicSignature {
      name: model.get_name(),
      inputs: model.get_schematic_input_signatures()?.clone(),
      outputs: model.get_schematic_output_signatures()?.clone(),
      providers: model.get_provider_signatures()?.clone(),
    })
  }
}
