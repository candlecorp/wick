use crate::dev::prelude::*;

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<(), SchematicError>")]
pub(crate) struct UpdateProvider {
  pub(crate) model: ProviderModel,
}

impl Handler<UpdateProvider> for SchematicService {
  type Result = Result<(), SchematicError>;

  fn handle(&mut self, msg: UpdateProvider, _ctx: &mut Context<Self>) -> Self::Result {
    self.update_network_provider(msg.model);
    self.validate_final()
  }
}
