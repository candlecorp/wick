use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use futures::future::try_join_all;
use futures::Future;
use nkeys::KeyPair;
use tokio::sync::mpsc::unbounded_channel;

use crate::dev::prelude::*;
use crate::dispatch::inv_error;
use crate::schematic_service::handlers::component_output::ComponentOutput;
use crate::schematic_service::handlers::payload_received::PayloadReceived;
use crate::schematic_service::handlers::transaction_update::TransactionUpdate;

type Result<T> = std::result::Result<T, SchematicError>;

impl Handler<Invocation> for SchematicService {
  type Result = ActorResult<Self, InvocationResponse>;

  fn handle(&mut self, msg: Invocation, ctx: &mut Context<Self>) -> Self::Result {
    let tx_id = msg.tx_id.clone();
    let target = msg.target.clone();
    let result = match target {
      Entity::Schematic(name) => handle_schematic(self, ctx.address(), &name, msg),
      Entity::Component(c) => handle_schematic(self, ctx.address(), &c.name, msg),
      Entity::Reference(reference) => self
        .get_component_definition(&reference)
        .map_or(Err(SchematicError::ReferenceNotFound(reference)), |def| {
          handle_schematic(self, ctx.address(), &def.id, msg)
        }),
      _ => Err(SchematicError::FailedPreRequestCondition(
        "Schematic invoked with entity it doesn't handle".into(),
      )),
    };

    match result {
      Ok(task) => {
        ActorResult::reply_async(task.into_actor(self).map(move |result, _, _| {
          result.map_or_else(|e| inv_error(&tx_id, &e.to_string()), |r| r)
        }))
      }
      Err(e) => ActorResult::reply(inv_error(&tx_id, &e.to_string())),
    }
  }
}

fn handle_schematic(
  service: &mut SchematicService,
  address: Addr<SchematicService>,
  name: &str,
  invocation: Invocation,
) -> Result<impl Future<Output = Result<InvocationResponse>>> {
  trace!("Requesting schematic '{}'", name);
  let tx_id = invocation.tx_id.clone();

  let (trans_tx, trans_rx) = unbounded_channel::<PayloadReceived>();
  service.tx_internal.insert(tx_id.clone(), trans_tx);

  let mut ready_rx = service.new_transaction(tx_id.clone(), trans_rx);

  let addr = address.clone();
  tokio::spawn(async move {
    while let Some(msg) = ready_rx.recv().await {
      if let TransactionUpdate::Done(tx_id) = &msg {
        info!("Schematic request finishing on transaction {}", tx_id);
        ready_rx.close();
      }
      meh!(addr.send(msg).await);
    }
    Ok!(())
  });

  let (tx, rx) = unbounded_channel::<ComponentOutput>();
  service.tx_external.insert(tx_id.clone(), tx);

  let state = service.state.as_mut().unwrap();
  let model = state.model.clone();

  let payload = invocation.msg.into_multibytes().map_err(|_| {
    SchematicError::FailedPreRequestCondition("Schematic sent invalid payload".into())
  })?;

  let payloads_rcvd = generate_packets(&model, &state.seed, &tx_id, &payload)?;

  Ok(async move {
    try_join_all(payloads_rcvd.into_iter().map(|inv| address.send(inv)))
      .await
      .map_err(|_| {
        SchematicError::FailedPreRequestCondition("Error pushing to schematic ports".into())
      })?;

    Ok(InvocationResponse::stream(tx_id, rx))
  })
}

fn generate_packets(
  model: &Arc<Mutex<SchematicModel>>,
  seed: &str,
  tx_id: &str,
  bytemap: &HashMap<String, Vec<u8>>,
) -> Result<Vec<PayloadReceived>> {
  let model = model.lock()?;
  let first_connections = model.get_downstream_connections(SCHEMATIC_INPUT);
  drop(model);
  trace!(
    "Generating schematic invocations for connections: {}",
    ConnectionDefinition::print_all(&first_connections)
  );

  let _kp = KeyPair::from_seed(seed)?;

  let invocations: Vec<PayloadReceived> = first_connections
    .into_iter()
    .map(|conn| {
      let bytes = bytemap
        .get(&conn.from.port)
        .unwrap_or_else(|| panic!("Port {} not found", conn.from));

      PayloadReceived {
        origin: conn.from.into(),
        target: conn.to.into(),
        tx_id: tx_id.to_owned(),
        payload: MessageTransport::MessagePack(bytes.clone()),
      }
    })
    .collect();
  Ok(invocations)
}
