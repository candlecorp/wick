use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use actix::fut::future::ActorFutureExt;
use futures::future::try_join_all;
use serde::{
  Deserialize,
  Serialize,
};
use tokio::sync::mpsc::{
  unbounded_channel,
  UnboundedReceiver,
  UnboundedSender,
};
use vino_rpc::SchematicSignature;
use wascap::prelude::KeyPair;

use crate::dev::prelude::*;
use crate::error::SchematicError;
use crate::provider_model::{
  initialize_native_provider,
  initialize_provider,
  ProviderChannel,
};
use crate::schematic_model::Connection;
use crate::transaction::TransactionMap;
use crate::validator::Validator;

type Result<T> = std::result::Result<T, SchematicError>;

#[derive(Debug)]
pub(crate) struct Schematic {
  recipients: HashMap<String, ProviderChannel>,
  invocation_map: HashMap<String, (String, String, Entity)>,
  state: Option<State>,
  tx_external: HashMap<String, UnboundedSender<ComponentOutput>>,
  tx_internal: HashMap<String, UnboundedSender<PayloadReceived>>,
}

#[derive(Debug)]
struct State {
  model: Arc<Mutex<SchematicModel>>,
  seed: String,
  name: String,
  transaction_map: TransactionMap,
}

impl Supervised for Schematic {}

impl Default for Schematic {
  fn default() -> Self {
    Schematic {
      recipients: HashMap::new(),
      invocation_map: HashMap::new(),
      state: None,
      tx_external: HashMap::new(),
      tx_internal: HashMap::new(),
    }
  }
}

impl Actor for Schematic {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    trace!("Schematic started");
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {}
}

impl Schematic {
  pub(crate) fn get_name(&self) -> String {
    self.get_state().name.clone()
  }

  fn validate_model(&mut self) -> Result<()> {
    let model = self.get_mut_state().model.lock()?;
    Validator::validate_late_errors(&model)?;
    Ok(())
  }

  fn get_state(&self) -> &State {
    if self.state.is_none() {
      panic!("Internal Error: schematic uninitialized");
    }
    let state = self.state.as_ref().unwrap();
    state
  }
  fn get_mut_state(&mut self) -> &mut State {
    if self.state.is_none() {
      panic!("Internal Error: schematic uninitialized");
    }
    let state = self.state.as_mut().unwrap();
    state
  }
  fn new_transaction(
    &mut self,
    tx_id: String,
    rx: UnboundedReceiver<PayloadReceived>,
  ) -> UnboundedReceiver<TransactionUpdate> {
    let state = self.get_mut_state();
    state.transaction_map.new_transaction(tx_id, rx)
  }

  fn get_component_model(&self, reference: &str) -> Option<ComponentModel> {
    let state = self.get_state();
    let lock = state.model.lock().unwrap();
    let def = lock.get_component_definition(reference)?;
    lock.get_component_model(&def.id)
  }
  fn get_component_definition(&self, reference: &str) -> Option<ComponentDefinition> {
    let state = self.get_state();
    let lock = state.model.lock().unwrap();
    lock.get_component_definition(reference)
  }
  fn get_recipient(&self, reference: &str) -> Option<Recipient<Invocation>> {
    trace!("Getting downstream recipient '{}'", reference);
    let component = self.get_component_definition(reference)?;
    let state = self.get_state();
    let lock = state.model.lock().unwrap();
    if !lock.has_component(&component.id) {
      return None;
    }
    drop(lock);
    trace!("Downstream recipient is: {:?}", component);
    let (ns, _name) = match parse_namespace(&component.id) {
      Ok(result) => result,
      Err(_) => return None,
    };
    let channel = self.recipients.get(&ns)?;
    Some(channel.recipient.clone())
  }
  fn get_outputs(&self, reference: &str) -> Vec<PortReference> {
    let state = self.get_state();
    let lock = state.model.lock().unwrap();
    lock.get_outputs(reference)
  }
  fn get_port_connections(&self, port: &PortReference) -> Vec<Connection> {
    let state = self.get_state();
    let lock = state.model.lock().unwrap();
    lock.get_port_connections(port)
  }
}

#[derive(Message, Debug)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) schematic: SchematicDefinition,
  pub(crate) network_provider_channel: Option<ProviderChannel>,
  pub(crate) seed: String,
  pub(crate) allow_latest: bool,
  pub(crate) allowed_insecure: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ProviderInitResponse {
  pub(crate) model: ProviderModel,
  pub(crate) channel: ProviderChannel,
}

impl Handler<Initialize> for Schematic {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
    trace!("Initializing schematic {}", msg.schematic.get_name());
    let seed = msg.seed;
    let allow_latest = msg.allow_latest;
    let name = msg.schematic.name.clone();
    let providers = msg.schematic.providers.clone();
    let model = SchematicModel::new(msg.schematic);
    actix_try!(Validator::validate_early_errors(&model));
    let model = Arc::new(Mutex::new(model));
    let allowed_insecure = msg.allowed_insecure;
    let network_provider_channel = msg.network_provider_channel;

    let task = initialize_providers(providers, seed.clone(), allow_latest, allowed_insecure)
      .into_actor(self)
      .map(|result, this, _ctx| {
        match result {
          Ok((mut channels, providers)) => {
            if let Some(network_provider_channel) = network_provider_channel {
              channels.push(network_provider_channel);
            }
            this.recipients = channels
              .into_iter()
              .map(|c| (c.namespace.clone(), c))
              .collect();
            let mut model = this.get_state().model.lock().unwrap();
            model.commit_providers(providers);
          }
          Err(e) => {
            error!("Error starting providers: {}", e);
          }
        }
        Ok!(())
      });
    let task = task.map(|_, this, _| this.validate_model());

    let state = State {
      name,
      seed,
      transaction_map: TransactionMap::new(model.clone()),
      model,
    };
    self.state = Some(state);

    ActorResult::reply_async(task)
  }
}

async fn initialize_providers(
  providers: Vec<ProviderDefinition>,
  seed: String,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
) -> Result<(Vec<ProviderChannel>, Vec<ProviderModel>)> {
  let (channel, provider_model) = initialize_native_provider("vino").await?;
  let mut channels = vec![channel];
  let mut models = vec![provider_model];

  for provider in providers {
    let (channel, provider_model) =
      initialize_provider(provider, &seed, allow_latest, &allowed_insecure).await?;
    channels.push(channel);
    models.push(provider_model);
  }
  Ok((channels, models))
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<()>")]
pub struct ComponentOutput {
  pub port: String,
  pub invocation_id: String,
  pub payload: Packet,
}

/// Maps output by invocation ID to its transaction and reference
impl Handler<ComponentOutput> for Schematic {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: ComponentOutput, ctx: &mut Context<Self>) -> Self::Result {
    let metadata = self.invocation_map.get(&msg.invocation_id).cloned();
    let (tx_id, schematic_name, entity) = metadata.unwrap();
    trace!(
      "Got output for tx '{}' on schematic '{}' and port {}",
      tx_id,
      schematic_name,
      entity
    );

    let receiver = ctx.address();
    let payload = msg.payload;
    let port = msg.port;

    ActorResult::reply_async(
      async move {
        let port = PortReference {
          name: port,
          reference: entity.into_component()?.reference,
        };
        trace!("Sending output ready to schematic");
        receiver
          .send(OutputPortReady {
            port,
            tx_id,
            payload: payload.into(),
          })
          .await
          .map_err(|_| InternalError(6013))??;
        trace!("Sent output ready to schematic");
        Ok(())
      }
      .into_actor(self),
    )
  }
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<InvocationResponse>")]
pub(crate) struct Request {
  pub(crate) tx_id: String,
  pub(crate) schematic: String,
  pub(crate) payload: HashMap<String, Vec<u8>>,
}

impl Handler<Request> for Schematic {
  type Result = ActorResult<Self, Result<InvocationResponse>>;

  fn handle(&mut self, msg: Request, ctx: &mut Context<Self>) -> Self::Result {
    trace!("Requesting schematic '{}'", msg.schematic);
    let tx_id = msg.tx_id.clone();

    let (trans_tx, trans_rx) = unbounded_channel::<PayloadReceived>();
    self.tx_internal.insert(tx_id.clone(), trans_tx);

    let mut ready_rx = self.new_transaction(tx_id.clone(), trans_rx);

    let addr = ctx.address();
    tokio::spawn(async move {
      while let Some(msg) = ready_rx.recv().await {
        if let TransactionUpdate::SchematicDone(tx_id) = &msg {
          info!("Schematic request finishing on transaction {}", tx_id);
          ready_rx.close();
        }
        meh!(addr.send(msg).await);
      }
      Ok!(())
    });

    let (tx, rx) = unbounded_channel::<ComponentOutput>();
    self.tx_external.insert(tx_id.clone(), tx);

    let state = self.state.as_mut().unwrap();
    let model = state.model.clone();

    highlight!("payload: {:?}", &msg.payload);

    let invocations = actix_try!(generate_packets(&model, &state.seed, &tx_id, &msg.payload));

    let host = ctx.address();
    ActorResult::reply_async(
      async move {
        let invocations = try_join_all(invocations.into_iter().map(|inv| host.send(inv)));

        invocations.await.map_err(|_| {
          SchematicError::FailedPreRequestCondition("Error pushing to schematic ports".into())
        })?;

        Ok(InvocationResponse::stream(tx_id, rx))
      }
      .into_actor(self),
    )
  }
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<()>")]
pub(crate) enum TransactionUpdate {
  ReferenceReady(ReferenceReady),
  SchematicOutput(SchematicOutputReceived),
  SchematicDone(String),
}
impl Handler<TransactionUpdate> for Schematic {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: TransactionUpdate, ctx: &mut Context<Self>) -> Self::Result {
    trace!("Transaction update: {:?}", msg);
    let addr = ctx.address();
    match msg {
      TransactionUpdate::ReferenceReady(msg) => ActorResult::reply_async(
        async move { addr.send(msg).await.map_err(|_| InternalError(6011))? }.into_actor(self),
      ),
      TransactionUpdate::SchematicOutput(msg) => ActorResult::reply_async(
        async move { addr.send(msg).await.map_err(|_| InternalError(6012))? }.into_actor(self),
      ),
      TransactionUpdate::SchematicDone(tx_id) => {
        let tx = actix_try!(self
          .tx_external
          .get(&tx_id)
          .ok_or_else(|| SchematicError::TransactionNotFound(tx_id.clone())));

        debug!("Sending output on transmitter");
        let output_msg = ComponentOutput {
          invocation_id: tx_id.clone(),
          payload: Packet::V0(packet::v0::Payload::Close),
          port: "<system>".to_owned(),
        };
        match tx.send(output_msg) {
          Ok(_) => debug!("Sent output to receiver for tx {}", tx_id),
          Err(e) => warn!("{}", SchematicError::SchematicClosedEarly(e.to_string())),
        }

        ActorResult::reply(Ok(()))
      }
    }
  }
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<()>")]
pub(crate) struct ReferenceReady {
  pub(crate) tx_id: String,
  pub(crate) reference: String,
  pub(crate) payload_map: HashMap<String, MessageTransport>,
}

impl Handler<ReferenceReady> for Schematic {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: ReferenceReady, ctx: &mut Context<Self>) -> Self::Result {
    trace!("Reference '{}' is ready to continue", msg.reference);
    let seed = self.get_state().seed.clone();
    let reference = msg.reference.clone();
    let tx_id = msg.tx_id;

    let kp = actix_try!(KeyPair::from_seed(&seed));
    let def = self.get_component_model(&msg.reference).unwrap();
    let mut invoke_payload = HashMap::new();
    for (name, payload) in msg.payload_map {
      match payload {
        MessageTransport::MessagePack(bytes) => {
          invoke_payload.insert(name, bytes);
        }
        payload => {
          let addr = ctx.address();
          return ActorResult::reply_async(
            async move {
              addr
                .send(ShortCircuit {
                  payload,
                  reference,
                  tx_id,
                })
                .await
                .map_err(|_| InternalError(6010))?
            }
            .into_actor(self),
          );
        }
      }
    }

    let invocation = Invocation::next(
      &tx_id,
      &kp,
      Entity::Schematic("<state>".to_owned()),
      Entity::Component(ComponentEntity {
        name: def.name,
        reference: msg.reference.clone(),
      }),
      MessageTransport::MultiBytes(invoke_payload),
    );
    let handler = actix_try!(self
      .get_recipient(&msg.reference)
      .ok_or_else(|| SchematicError::ReferenceError(reference.clone())));

    let addr = ctx.address();
    let name = self.get_name();

    self.invocation_map.insert(
      invocation.id.clone(),
      (tx_id, name.clone(), invocation.target.clone()),
    );

    let task = async move {
      let target = invocation.target.url();

      let response = handler
        .send(invocation)
        .await
        .map_err(|_| InternalError(6009))?;

      match response {
        InvocationResponse::Success { .. } => unreachable!(),
        InvocationResponse::Stream { tx_id, mut rx } => {
          trace!(
            "spawning task to handle output for {}:{}|{}",
            tx_id,
            name,
            target
          );
          tokio::spawn(async move {
            while let Some(next) = rx.next().await {
              match addr.send(next).await {
                Ok(_) => {
                  debug!(
                    "Sent ready output to network for {}:{}:{}",
                    tx_id, name, target
                  );
                }
                Err(e) => {
                  error!("Error sending ready output: {}", e);
                }
              };
            }
            trace!("Task finished");
          });
          Ok(())
        }
        InvocationResponse::Error { tx_id, msg } => {
          warn!(
            "Tx '{}': schematic '{}' short-circuiting from '{}': {}",
            tx_id, name, reference, msg
          );
          addr
            .send(ShortCircuit {
              tx_id: tx_id.clone(),
              reference: reference.clone(),
              payload: MessageTransport::Error(msg),
            })
            .await
            .map_err(|_| InternalError(6007))?
        }
      }
    };

    ActorResult::reply_async(task.into_actor(self))
  }
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<()>")]
pub(crate) struct SchematicOutputReceived {
  pub(crate) port: String,
  pub(crate) tx_id: String,
  pub(crate) payload: MessageTransport,
}

impl Handler<SchematicOutputReceived> for Schematic {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: SchematicOutputReceived, _ctx: &mut Context<Self>) -> Self::Result {
    trace!("Schematic port '{}' is ready", msg.port);

    let tx = actix_try!(self
      .tx_external
      .get(&msg.tx_id)
      .ok_or_else(|| SchematicError::TransactionNotFound(msg.tx_id.clone())));

    debug!("Sending output on transmitter");
    let err = Packet::V0(packet::v0::Payload::Error(
      "Invalid payload received as schematic output".to_owned(),
    ));
    let output_msg = ComponentOutput {
      invocation_id: msg.tx_id,
      payload: match msg.payload {
        MessageTransport::Invalid => Packet::V0(packet::v0::Payload::Invalid),
        MessageTransport::Exception(v) => Packet::V0(packet::v0::Payload::Exception(v)),
        MessageTransport::Error(v) => Packet::V0(packet::v0::Payload::Error(v)),
        MessageTransport::MessagePack(v) => Packet::V0(packet::v0::Payload::MessagePack(v)),
        MessageTransport::MultiBytes(_) => err,
        MessageTransport::OutputMap(_) => err,
        MessageTransport::Test(_) => err,
        MessageTransport::Signal(v) => match v {
          MessageSignal::Close => Packet::V0(packet::v0::Payload::Close),
          MessageSignal::OpenBracket => Packet::V0(packet::v0::Payload::OpenBracket),
          MessageSignal::CloseBracket => Packet::V0(packet::v0::Payload::CloseBracket),
        },
      },
      port: msg.port,
    };
    meh!(tx.send(output_msg));

    ActorResult::reply(Ok(()))
  }
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<SchematicSignature>")]
pub(crate) struct GetSignature {}

impl Handler<GetSignature> for Schematic {
  type Result = Result<SchematicSignature>;

  fn handle(&mut self, _msg: GetSignature, _ctx: &mut Context<Self>) -> Self::Result {
    let state = self.get_state();
    let model = state.model.lock()?;

    Ok(SchematicSignature {
      name: self.get_name(),
      inputs: model.get_schematic_input_signatures()?.clone(),
      outputs: model.get_schematic_output_signatures()?.clone(),
      providers: model.get_provider_signatures()?.clone(),
    })
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Message, PartialEq)]
#[rtype(result = "Result<()>")]
pub struct PayloadReceived {
  pub tx_id: String,
  pub origin: PortReference,
  pub target: PortReference,
  pub payload: MessageTransport,
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

impl Handler<PayloadReceived> for Schematic {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: PayloadReceived, _ctx: &mut Context<Self>) -> Self::Result {
    trace!("Payload received: {:?}", msg);
    let port = msg.target.clone();
    let tx_id = msg.tx_id.clone();
    trace!("Receiving on port {}", port);

    let transaction_handler = actix_try!(self.tx_internal.get(&tx_id).ok_or(InternalError(6003)));
    debug!("Sent output to transaction handler for {:?}", msg);
    actix_try!(transaction_handler.send(msg));

    ActorResult::reply(Ok(()))
  }
}

#[derive(Message, Clone)]
#[rtype(result = "Result<()>")]
pub(crate) struct ShortCircuit {
  pub(crate) tx_id: String,
  pub(crate) reference: String,
  pub(crate) payload: MessageTransport,
}

impl Handler<ShortCircuit> for Schematic {
  type Result = ResponseActFuture<Self, Result<()>>;

  fn handle(&mut self, msg: ShortCircuit, ctx: &mut Context<Self>) -> Self::Result {
    trace!("Short circuiting component {}", msg.reference);
    let reference = msg.reference;
    let tx_id = msg.tx_id;
    let payload = msg.payload;

    let outputs = self.get_outputs(&reference);
    trace!("Output ports for {} : {:?}", reference, outputs);
    let downstreams: Vec<Connection> = outputs
      .iter()
      .flat_map(|port| self.get_port_connections(port))
      .collect();
    trace!(
      "Connections to short {:?}",
      Connection::print_all(&downstreams)
    );
    let outputs: Vec<OutputPortReady> = downstreams
      .into_iter()
      .map(|conn| OutputPortReady {
        tx_id: tx_id.clone(),
        port: conn.from,
        payload: payload.clone(),
      })
      .collect();
    let schematic_host = ctx.address();

    let futures = outputs
      .into_iter()
      .map(move |message| schematic_host.send(message));

    Box::pin(
      async move {
        try_join_all(futures)
          .await
          .map_err(|_| InternalError(6002))?;
        Ok(())
      }
      .into_actor(self),
    )
  }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<()>")]
pub(crate) struct OutputPortReady {
  pub(crate) port: PortReference,
  pub(crate) tx_id: String,
  pub(crate) payload: MessageTransport,
}

impl Handler<OutputPortReady> for Schematic {
  type Result = ResponseActFuture<Self, Result<()>>;

  fn handle(&mut self, msg: OutputPortReady, ctx: &mut Context<Self>) -> Self::Result {
    trace!("Output ready on {}", msg.port);
    let defs = self.get_port_connections(&msg.port);
    let reference = msg.port.reference;
    let port = msg.port.name;
    let tx_id = msg.tx_id;
    let data = msg.payload;
    let addr = ctx.address();
    let task = async move {
      let origin = PortReference {
        name: port.clone(),
        reference: reference.clone(),
      };
      let to_message = |conn: Connection| PayloadReceived {
        tx_id: tx_id.clone(),
        origin: origin.clone(),
        target: PortReference {
          name: conn.to.name.clone(),
          reference: conn.to.reference,
        },
        payload: data.clone(),
      };
      let invocations = try_join_all(defs.into_iter().map(to_message).map(|ips| addr.send(ips)));
      invocations.await.map_err(|_| InternalError(6001))?;
      Ok(())
    }
    .into_actor(self);

    Box::pin(task)
  }
}

#[cfg(test)]
mod test {
  use vino_rpc::{
    bind_new_socket,
    make_rpc_server,
  };

  use super::*;
  use crate::test::prelude::*;

  #[test_env_log::test(actix_rt::test)]
  async fn test_basic_schematic() -> TestResult<()> {
    let kp = KeyPair::new_server();
    let tx_id = get_uuid();
    let schematic = Schematic::default();
    let addr = schematic.start();
    let schematic_name = "logger";

    let mut schematic_def = new_schematic(schematic_name);
    schematic_def
      .components
      .insert("logger".to_owned(), ComponentDefinition::new("vino", "log"));
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: SCHEMATIC_INPUT.to_owned(),
        port: "input".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: "logger".to_owned(),
        port: "input".to_owned(),
      },
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "logger".to_owned(),
        port: "output".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_owned(),
        port: "output".to_owned(),
      },
    });

    addr
      .send(Initialize {
        schematic: schematic_def,
        network_provider_channel: None,
        seed: kp.seed()?,
        allow_latest: true,
        allowed_insecure: vec![],
      })
      .await??;
    let mut input: HashMap<String, Vec<u8>> = HashMap::new();
    let user_data = "this is test input";
    input.insert("input".to_owned(), mp_serialize(user_data)?);
    let response: InvocationResponse = addr
      .send(super::Request {
        tx_id: tx_id.clone(),
        schematic: schematic_name.to_owned(),
        payload: input,
      })
      .await??;
    match response {
      InvocationResponse::Success { .. } => panic!("should have gotten a stream"),

      InvocationResponse::Stream { mut rx, .. } => {
        debug!("Got stream");
        let mut i = 0;

        while let Some(next) = rx.next().await {
          i += 1;
          let packet = next.payload;
          debug!("Packet {}: {:?}", i, packet);
          let payload: String = packet.try_into()?;
          debug!("Payload {}", payload);
          equals!(payload, user_data);
        }
        debug!("Number of packets received: {}", i);
        equals!(i, 1);
      }
      InvocationResponse::Error { msg, .. } => panic!("{}", msg),
    };

    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn test_native_provider() -> TestResult<()> {
    let schematic = Schematic::default();
    let addr = schematic.start();
    let schematic_name = "logger";

    let mut schematic_def = new_schematic(schematic_name);
    schematic_def
      .components
      .insert("logger".to_owned(), ComponentDefinition::new("vino", "log"));
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: SCHEMATIC_INPUT.to_owned(),
        port: "input".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: "logger".to_owned(),
        port: "input".to_owned(),
      },
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "logger".to_owned(),
        port: "output".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_owned(),
        port: "output".to_owned(),
      },
    });

    let hostkey = KeyPair::new_server();
    let tx_id = get_uuid();

    addr
      .send(Initialize {
        schematic: schematic_def,
        network_provider_channel: None,
        seed: hostkey.seed()?,
        allow_latest: true,
        allowed_insecure: vec![],
      })
      .await??;
    let mut input: HashMap<String, Vec<u8>> = HashMap::new();
    let user_data = "Hello world";
    input.insert("input".to_owned(), mp_serialize(user_data)?);
    let response: InvocationResponse = addr
      .send(super::Request {
        tx_id: tx_id.clone(),
        schematic: schematic_name.to_owned(),
        payload: input,
      })
      .await??;

    match response {
      InvocationResponse::Success { .. } => panic!("should have gotten a stream"),

      InvocationResponse::Stream { mut rx, .. } => {
        debug!("Got stream");
        let mut i = 0;

        while let Some(next) = rx.next().await {
          i += 1;
          let packet = next.payload;
          debug!("Packet {}: {:?}", i, packet);
          let payload: String = packet.try_into()?;
          debug!("Payload {}", payload);
          equals!(payload, user_data);
        }
        debug!("Number of packets received: {}", i);
        equals!(i, 1);
      }
      InvocationResponse::Error { msg, .. } => panic!("{}", msg),
    };
    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn test_grpc_url_provider() -> TestResult<()> {
    let socket = bind_new_socket()?;
    let port = socket.local_addr()?.port();
    make_rpc_server(socket, test_vino_provider::Provider::default());

    let schematic = Schematic::default();
    let addr = schematic.start();
    let schematic_name = "logger";

    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.providers.push(ProviderDefinition {
      namespace: "test-namespace".to_owned(),
      kind: ProviderKind::GrpcUrl,
      reference: format!("http://127.0.0.1:{}", port),
      data: HashMap::new(),
    });
    schematic_def
      .components
      .insert("logger".to_owned(), ComponentDefinition::new("vino", "log"));
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: SCHEMATIC_INPUT.to_owned(),
        port: "input".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: "logger".to_owned(),
        port: "input".to_owned(),
      },
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "logger".to_owned(),
        port: "output".to_owned(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_owned(),
        port: "output".to_owned(),
      },
    });

    let hostkey = KeyPair::new_server();
    let tx_id = get_uuid();

    addr
      .send(Initialize {
        schematic: schematic_def,
        seed: hostkey.seed()?,
        network_provider_channel: None,
        allow_latest: true,
        allowed_insecure: vec![],
      })
      .await??;
    let mut input: HashMap<String, Vec<u8>> = HashMap::new();
    let user_data = "Hello world";
    input.insert("input".to_owned(), mp_serialize(user_data)?);
    let response: InvocationResponse = addr
      .send(super::Request {
        tx_id: tx_id.clone(),
        schematic: schematic_name.to_owned(),
        payload: input,
      })
      .await??;

    match response {
      InvocationResponse::Success { .. } => panic!("should have gotten a stream"),
      InvocationResponse::Stream { mut rx, .. } => {
        debug!("Got stream");
        let mut i = 0;

        while let Some(next) = rx.next().await {
          i += 1;
          let packet = next.payload;
          debug!("Packet {}: {:?}", i, packet);
          let payload: String = packet.try_into()?;
          debug!("Payload {}", payload);
          equals!(payload, user_data);
        }
        debug!("Number of packets received: {}", i);
        equals!(i, 1);
      }
      InvocationResponse::Error { msg, .. } => panic!("{}", msg),
    };
    Ok(())
  }
}
