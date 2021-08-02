use crate::dev::prelude::*;
use crate::schematic_service::handlers::output_message::OutputMessage;

#[derive(Message, Clone)]
#[rtype(result = "Result<(), SchematicError>")]
pub(crate) struct ShortCircuit {
  pub(crate) tx_id: String,
  pub(crate) reference: String,
  pub(crate) payload: MessageTransport,
}

impl ShortCircuit {
  pub(crate) fn new<T, U>(tx_id: T, reference: U, payload: MessageTransport) -> Self
  where
    T: AsRef<str>,
    U: AsRef<str>,
  {
    Self {
      tx_id: tx_id.as_ref().to_owned(),
      reference: reference.as_ref().to_owned(),
      payload,
    }
  }
}

impl Handler<ShortCircuit> for SchematicService {
  type Result = ResponseActFuture<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: ShortCircuit, ctx: &mut Context<Self>) -> Self::Result {
    trace!("SC:{}:{}:BYPASS", self.name, msg.reference);
    let reference = msg.reference;
    let tx_id = msg.tx_id;
    let payload = msg.payload;

    let outputs = self.get_outputs(&reference);

    let downstreams: Vec<ConnectionDefinition> = outputs
      .iter()
      .flat_map(|port| self.get_port_connections(port))
      .collect();

    trace!(
      "SC:{}:{}:BYPASS:Connections {}",
      self.name,
      reference,
      join(&downstreams, ", ")
    );

    let outputs: Vec<OutputMessage> = downstreams
      .into_iter()
      .flat_map(|conn| {
        vec![
          OutputMessage {
            tx_id: tx_id.clone(),
            port: conn.from.clone(),
            payload: payload.clone(),
          },
          OutputMessage {
            tx_id: tx_id.clone(),
            port: conn.from,
            payload: MessageTransport::done(),
          },
        ]
      })
      .collect();

    let schematic_host = ctx.address();

    let futures = outputs
      .into_iter()
      .map(move |message| schematic_host.send(message));

    Box::pin(
      async move {
        join_or_err(futures, 6002).await?;
        Ok(())
      }
      .into_actor(self),
    )
  }
}
