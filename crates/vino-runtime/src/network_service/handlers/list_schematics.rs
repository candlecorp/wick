use crate::dev::prelude::*;
use crate::schematic_service::handlers::get_signature::GetSignature;

type Result<T> = std::result::Result<T, NetworkError>;

#[derive(Message)]
#[rtype(result = "Result<Vec<SchematicSignature>>")]
pub(crate) struct ListSchematics {}

impl Handler<ListSchematics> for NetworkService {
  type Result = ActorResult<Self, Result<Vec<SchematicSignature>>>;

  fn handle(&mut self, _msg: ListSchematics, _ctx: &mut Context<Self>) -> Self::Result {
    actix_try!(self.ensure_is_started(), 5003);
    let schematics = self.schematics.clone();
    let requests = schematics
      .into_values()
      .map(|addr| addr.send(GetSignature {}));
    let task = async move {
      let mut results = Vec::new();
      for msg in requests {
        results.push(msg.await.map_err(|_| InternalError(5004))?);
      }

      let mut signatures = vec![];
      for result in results {
        if let Err(err) = result {
          warn!("Error requesting a schematic signature: {}", err);
          continue;
        }
        signatures.push(result.unwrap());
      }
      Ok(signatures)
    }
    .into_actor(self);

    ActorResult::reply_async(task)
  }
}
