use crate::dev::prelude::*;
use crate::schematic_service::handlers::output_message::OutputMessage;

#[derive(Message, Clone)]
#[rtype(result = "Result<(), SchematicError>")]
pub(crate) struct ShortCircuit {
  pub(crate) tx_id: String,
  pub(crate) instance: String,
  pub(crate) payload: MessageTransport,
}

impl ShortCircuit {
  pub(crate) fn new<T, U>(tx_id: T, instance: U, payload: MessageTransport) -> Self
  where
    T: AsRef<str>,
    U: AsRef<str>,
  {
    Self {
      tx_id: tx_id.as_ref().to_owned(),
      instance: instance.as_ref().to_owned(),
      payload,
    }
  }
}

impl Handler<ShortCircuit> for SchematicService {
  type Result = ResponseActFuture<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: ShortCircuit, ctx: &mut Context<Self>) -> Self::Result {
    trace!("SC[{}]:{}:BYPASS", self.name, msg.instance);
    let instance = msg.instance;
    let tx_id = msg.tx_id;
    let payload = msg.payload;

    let outputs = get_outputs(self.get_model(), &instance);

    let downstreams: Vec<ConnectionDefinition> = outputs
      .iter()
      .flat_map(|port| get_port_connections(self.get_model(), port))
      .collect();

    trace!(
      "SC[{}]:{}:BYPASS:Connections {}",
      self.name,
      instance,
      join(&downstreams, ", ")
    );

    let schematic_host = ctx.address();

    let futures = downstreams
      .into_iter()
      .flat_map(move |conn| {
        vec![
          OutputMessage::new(&tx_id, conn.from.clone(), payload.clone()),
          OutputMessage::new(&tx_id, conn.from, MessageTransport::done()),
        ]
      })
      .map(move |message| schematic_host.send(message));

    Box::pin(
      async move {
        for msg in futures {
          msg.await.map_err(|_| InternalError::E6002)??;
        }

        Ok(())
      }
      .into_actor(self),
    )
  }
}
