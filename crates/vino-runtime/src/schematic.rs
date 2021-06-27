use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};
use std::time::Duration;

use actix::fut::future::ActorFutureExt;
use actix::prelude::*;
use futures::future::try_join_all;
use serde::{
  Deserialize,
  Serialize,
};
use vino_component::Packet;
use vino_transport::MessageTransport;
use wascap::prelude::KeyPair;

use super::dispatch::{
  Invocation,
  InvocationResponse,
};
use super::schematic_response::initialize_schematic_output;
use crate::actix::ActorResult;
use crate::dispatch::{
  ComponentEntity,
  PortEntity,
  VinoEntity,
};
use crate::network::Network;
use crate::provider_model::initialize_provider;
use crate::schematic_definition::{
  ComponentDefinition,
  ConnectionDefinition,
  ProviderDefinition,
  ProviderKind,
  SchematicDefinition,
};
use crate::schematic_model::{
  SchematicModel,
  Validator,
};
use crate::schematic_response::{
  get_transaction_output,
  push_to_schematic_output,
};
use crate::transaction::TransactionMap;
use crate::{
  Error,
  Result,
};

pub type SchematicOutput = HashMap<String, MessageTransport>;

#[derive(Debug)]
enum ComponentStatus {
  Ready(Invocation),
  Waiting,
  ShortCircuit(MessageTransport),
}

#[derive(Debug)]
pub(crate) struct Schematic {
  network: Option<Addr<Network>>,
  recipients: HashMap<String, Recipient<Invocation>>,
  transaction_map: TransactionMap,
  invocation_map: HashMap<String, (String, String, VinoEntity)>,
  state: Option<State>,
}

#[derive(Debug)]
struct State {
  host_id: String,
  model: Arc<Mutex<SchematicModel>>,
  seed: String,
  name: String,
}

impl Supervised for Schematic {}

impl Default for Schematic {
  fn default() -> Self {
    Schematic {
      network: None,
      recipients: HashMap::new(),
      transaction_map: TransactionMap::new(),
      invocation_map: HashMap::new(),
      state: None,
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
    self
      .state
      .as_ref()
      .map(|state| state.name.clone())
      .unwrap_or_else(|| "<uninitialized>".to_string())
  }
  fn get_state(&self) -> &State {
    if self.state.is_none() {
      panic!("Internal Error: schematic uninitialized");
    }
    let state = self.state.as_ref().unwrap();
    state
  }
  fn get_component(&self, reference: &str) -> Option<ComponentDefinition> {
    let state = self.get_state();
    let lock = state.model.lock().unwrap();
    lock.get_component_definition(reference)
  }
  fn get_downstream_recipient(&self, reference: &str) -> Option<Recipient<Invocation>> {
    let state = self.get_state();
    let lock = state.model.lock().unwrap();
    lock.get_downstream_recipient(reference)
  }
  fn get_outputs(&self, reference: &str) -> Vec<String> {
    let state = self.get_state();
    let lock = state.model.lock().unwrap();
    lock.get_outputs(reference)
  }
  fn get_connections(&self, reference: &str, port: &str) -> Vec<ConnectionDefinition> {
    let state = self.get_state();
    let lock = state.model.lock().unwrap();
    lock.get_connections(reference, port)
  }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) schematic: SchematicDefinition,
  pub(crate) host_id: String,
  pub(crate) seed: String,
  pub(crate) network: Addr<Network>,
  pub(crate) allow_latest: bool,
  pub(crate) allowed_insecure: Vec<String>,
}

impl Handler<Initialize> for Schematic {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: Initialize, ctx: &mut Self::Context) -> Self::Result {
    trace!("Initializing schematic {}", msg.schematic.get_name());
    let seed = msg.seed;
    let allow_latest = msg.allow_latest;
    let allowed_insecure = msg.allowed_insecure;
    let model = SchematicModel::new(msg.schematic);
    actix_try!(Validator::validate_early_errors(&model));
    self.network = Some(msg.network);

    let provider_initialization = InitializeProviders {
      schematic: model.definition.clone(),
      host_id: msg.host_id.clone(),
      seed: seed.clone(),
    };

    let component_initialization = InitializeComponents {
      schematic: model.definition.clone(),
      host_id: msg.host_id.clone(),
      seed: seed.clone(),
      allow_latest,
      allowed_insecure,
    };
    let addr = ctx.address();
    let addr2 = ctx.address();
    let state = State {
      name: model.get_name(),
      seed,
      model: Arc::new(Mutex::new(model)),
      host_id: msg.host_id,
    };
    self.state = Some(state);
    let task = async move { addr.send(provider_initialization).await }
      .into_actor(self)
      .then(move |_, this, _| {
        async move { addr2.send(component_initialization).await? }.into_actor(this)
      });

    ActorResult::reply_async(task)
  }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<()>")]
struct InitializeProviders {
  schematic: SchematicDefinition,
  host_id: String,
  seed: String,
}

/// Starts an actix arbiter for each provider
impl Handler<InitializeProviders> for Schematic {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: InitializeProviders, _ctx: &mut Self::Context) -> Self::Result {
    trace!(
      "Starting providers for schematic {}",
      msg.schematic.get_name()
    );
    let seed = msg.seed.clone();

    let schematic = msg.schematic;

    let task = async move {
      let mut providers = vec![
        initialize_provider(
          &ProviderDefinition {
            namespace: "vino".to_string(),
            kind: ProviderKind::Native,
            reference: "".to_string(),
            data: HashMap::new(),
          },
          seed.clone(),
        )
        .await?,
      ];
      for provider in &schematic.providers {
        let handler = initialize_provider(provider, seed.clone()).await?;
        providers.push(handler);
      }
      Ok!(providers)
    };

    ActorResult::reply_async(task.into_actor(self).map(|providers, this, _ctx| {
      let state = this.get_state();
      let mut model = state.model.lock().unwrap();
      match providers {
        Ok(providers) => providers
          .into_iter()
          .for_each(|provider| meh!(model.add_provider(provider))),
        Err(e) => {
          error!("Error starting providers: {}", e);
        }
      };
      Ok(())
    }))
  }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<()>")]
pub(crate) struct InitializeComponents {
  pub(crate) schematic: SchematicDefinition,
  pub(crate) host_id: String,
  pub(crate) seed: String,
  pub(crate) allow_latest: bool,
  pub(crate) allowed_insecure: Vec<String>,
}

/// Starts an actix arbiter for each component
impl Handler<InitializeComponents> for Schematic {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: InitializeComponents, _ctx: &mut Self::Context) -> Self::Result {
    trace!(
      "Starting components for schematic {}",
      msg.schematic.get_name()
    );
    let seed = msg.seed.clone();

    let schematic = msg.schematic;
    let allow_latest = msg.allow_latest;
    let allowed_insecure = msg.allowed_insecure;
    let state = self.get_state();
    let model = state.model.clone();

    let task = async move {
      for (reference, def) in &schematic.components {
        let lock = model.lock().unwrap();
        if lock.has_component(&def.id) {
          warn!("Component with id '{}' already loaded, skipping", def.id);
          continue;
        }
        let external_definition = lock.lookup_external(&def.id);
        drop(lock);
        if external_definition.is_none() {
          warn!(
            "Component '{}' not started and has no external definition, skipping.",
            def.id
          );
          continue;
        }
        let external_definition = external_definition.unwrap();

        debug!(
          "Loading component {}({}) from {}",
          reference, def.id, external_definition.reference
        );

        let result = crate::components::vino_component::load_component(
          external_definition.reference,
          seed.clone(),
          allow_latest,
          &allowed_insecure,
        )
        .await;
        let mut model = model.lock().unwrap();
        match result {
          Ok(component) => {
            meh!(model.add_component(component));
          }
          Err(e) => {
            error!("Failed to load component {}: {}", reference, e);
          }
        }
      }
      Ok!(())
    };
    ActorResult::reply_async(task.into_actor(self))
  }
}

#[derive(Message, Debug)]
#[rtype(result = "Result<()>")]
pub struct PushOutput {
  pub port: String,
  pub invocation_id: String,
  pub payload: Packet,
}

impl Handler<PushOutput> for Schematic {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: PushOutput, ctx: &mut Context<Self>) -> Self::Result {
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
        let port = PortEntity {
          name: port,
          reference: entity.into_component()?.reference,
          schematic: schematic_name.to_string(),
        };
        trace!("Sending output ready to schematic");
        receiver
          .send(OutputReady {
            port,
            tx_id,
            payload: payload.into(),
          })
          .await??;
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
    let tx_id = msg.tx_id.to_string();
    let schematic = msg.schematic.to_string();

    self.transaction_map.new_transaction(tx_id.to_string());
    let state = self.state.as_ref().unwrap();
    let model = state.model.clone();

    let invocations = actix_try!(generate_packets(
      model,
      &state.seed,
      tx_id.to_string(),
      schematic.to_string(),
      msg.payload
    ));

    let host = ctx.address();
    ActorResult::reply_async(
      async move { handle_schematic_invocation(invocations, host, tx_id, schematic).await }
        .into_actor(self),
    )
  }
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<InvocationResponse>")]
pub(crate) struct GetTransactionOutput {
  pub(crate) schematic: String,
  pub(crate) tx_id: String,
}

impl Handler<GetTransactionOutput> for Schematic {
  type Result = ActorResult<Self, Result<InvocationResponse>>;

  fn handle(&mut self, msg: GetTransactionOutput, _ctx: &mut Context<Self>) -> Self::Result {
    trace!(
      "Requesting future for schematic '{}' on tx {}",
      msg.schematic,
      msg.tx_id
    );

    let tx_id = msg.tx_id;

    let schematic_name = msg.schematic;
    let timeout = Duration::from_millis(1000);
    let schematic = get_transaction_output(&tx_id, &schematic_name);

    let task = async move {
      match schematic {
        Ok(future) => match actix_rt::time::timeout(timeout, future).await {
          Ok(r) => Ok(r),
          Err(e) => Ok(InvocationResponse::error(
            tx_id,
            format!("Error waiting for schematic output: {}", e.to_string()),
          )),
        },
        Err(e) => Ok(InvocationResponse::error(tx_id, e.to_string())),
      }
    };

    ActorResult::reply_async(task.into_actor(self))
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Message, PartialEq)]
#[rtype(result = "Result<()>")]
struct PayloadReceived {
  tx_id: String,
  origin: PortEntity,
  target: PortEntity,
  payload: MessageTransport,
}

fn generate_packets(
  model: Arc<Mutex<SchematicModel>>,
  seed: &str,
  tx_id: String,
  name: String,
  bytemap: HashMap<String, Vec<u8>>,
) -> Result<Vec<PayloadReceived>> {
  let model = model.lock()?;
  let schematic_outputs = model.get_schematic_outputs();
  let first_connections = model.get_downstream_connections(crate::SCHEMATIC_INPUT);
  drop(model);

  let _kp = KeyPair::from_seed(seed)?;

  initialize_schematic_output(&tx_id, &name, schematic_outputs);

  let invocations: Vec<PayloadReceived> = first_connections
    .iter()
    .map(|conn| {
      let bytes = bytemap
        .get(&conn.from.port)
        .unwrap_or_else(|| panic!("Output on port '{}' not found", conn.to.instance));

      PayloadReceived {
        origin: PortEntity {
          schematic: name.to_string(),
          name: conn.from.port.to_string(),
          reference: conn.from.instance.to_string(),
        },
        target: PortEntity {
          schematic: name.to_string(),
          name: conn.to.port.to_string(),
          reference: conn.to.instance.to_string(),
        },
        tx_id: tx_id.to_string(),
        payload: MessageTransport::MessagePack(bytes.clone()),
      }
    })
    .collect();
  Ok(invocations)
}

async fn handle_schematic_invocation<'a>(
  invocations: Vec<PayloadReceived>,
  schematic: Addr<Schematic>,
  tx_id: String,
  target: String,
) -> Result<InvocationResponse> {
  let invocations = try_join_all(invocations.into_iter().map(|inv| schematic.send(inv)));

  invocations
    .await
    .map_err(|_| Error::SchematicError("Error pushing to schematic ports".into()))?;

  let response = schematic
    .send(GetTransactionOutput {
      tx_id: tx_id.to_string(),
      schematic: target,
    })
    .await
    .map_err(|e| {
      Error::SchematicError(format!("Error waiting for schematic to complete: {}", e))
    })??;

  Ok(response)
}

#[derive(Message)]
#[rtype(result = "()")]
pub(crate) struct RecordInvocationState {
  pub(crate) inv_id: String,
  pub(crate) tx_id: String,
  pub(crate) schematic: String,
  pub(crate) entity: VinoEntity,
}

impl Handler<RecordInvocationState> for Schematic {
  type Result = ();

  fn handle(&mut self, msg: RecordInvocationState, _ctx: &mut Context<Self>) -> Self::Result {
    trace!("Recording invocation {}", msg.inv_id);
    self
      .invocation_map
      .insert(msg.inv_id, (msg.tx_id, msg.schematic, msg.entity));
  }
}

impl Handler<PayloadReceived> for Schematic {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: PayloadReceived, ctx: &mut Context<Self>) -> Self::Result {
    let name = self.get_name();
    let port = msg.target;
    let payload = msg.payload;
    let tx_id = msg.tx_id;
    let state = self.state.as_ref().unwrap();
    let model = state.model.clone();
    trace!("Receiving on port {}", port);

    let reference = port.reference.to_string();
    //TODO normalize output to the same buffers as regular ports
    if reference == crate::SCHEMATIC_OUTPUT {
      return ActorResult::reply(push_to_schematic_output(&tx_id, &name, &port.name, payload));
    }
    let status = if !payload.is_ok() {
      ComponentStatus::ShortCircuit(payload)
    } else {
      actix_try!(self
        .transaction_map
        .push_to_port(&tx_id, port.clone(), payload));
      let metadata = actix_try!({
        let lock = model.lock().unwrap();
        lock.get_component_metadata(&reference)
      });
      let input_ports: Vec<PortEntity> = metadata
        .inputs
        .iter()
        .map(|port_name| PortEntity {
          schematic: self.get_name(),
          reference: reference.to_string(),
          name: port_name.to_string(),
        })
        .collect();
      let ready = self.transaction_map.are_ports_ready(&tx_id, &input_ports);
      if !ready {
        ComponentStatus::Waiting
      } else {
        let kp = actix_try!(KeyPair::from_seed(&state.seed));
        let def = self.get_component(&reference).unwrap();
        let job_data = actix_try!(self.transaction_map.take_from_ports(&tx_id, input_ports));
        ComponentStatus::Ready(Invocation::next(
          &tx_id,
          &kp,
          VinoEntity::Schematic(port.schematic),
          VinoEntity::Component(ComponentEntity {
            id: def.id,
            name: metadata.name,
            reference: reference.to_string(),
          }),
          MessageTransport::MultiBytes(job_data),
        ))
      }
    };

    let schematic_host = ctx.address();

    let recipient = self.get_downstream_recipient(&reference);

    ActorResult::reply_async(
      async move {
        match status {
          ComponentStatus::ShortCircuit(payload) => match schematic_host
            .send(ShortCircuit {
              tx_id: tx_id.to_string(),
              schematic: name,
              reference,
              payload,
            })
            .await
          {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::SchematicError(format!(
              "Error deserializing job signal: {}",
              e
            ))),
          },
          ComponentStatus::Waiting => {
            trace!("Component {} is still waiting on data", reference);
            Ok(())
          }
          ComponentStatus::Ready(invocation) => {
            schematic_host.do_send(RecordInvocationState {
              inv_id: invocation.id.to_string(),
              tx_id: tx_id.clone(),
              schematic: name.to_string(),
              entity: invocation.target.clone(),
            });
            let target = invocation.target.url();

            let response: InvocationResponse = match recipient {
              Some(recipient) => recipient.send(invocation).await?,
              None => InvocationResponse::error(tx_id, "No recipient found".to_string()),
            };

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
                  while let Some(next) = rx.recv().await {
                    match schematic_host.send(next).await {
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
                  trace!("Task finished")
                });
                Ok(())
              }
              InvocationResponse::Error { tx_id, msg } => {
                warn!(
                  "Tx '{}': schematic '{}' short-circuiting from '{}': {}",
                  tx_id, name, reference, msg
                );
                schematic_host
                  .send(ShortCircuit {
                    tx_id: tx_id.to_string(),
                    schematic: name.to_string(),
                    reference: reference.to_string(),
                    payload: MessageTransport::Error(msg),
                  })
                  .await?
              }
            }
          }
        }
      }
      .into_actor(self),
    )
  }
}

#[derive(Message, Clone)]
#[rtype(result = "Result<()>")]
pub(crate) struct ShortCircuit {
  pub(crate) tx_id: String,
  pub(crate) schematic: String,
  pub(crate) reference: String,
  pub(crate) payload: MessageTransport,
}

impl Handler<ShortCircuit> for Schematic {
  type Result = ResponseActFuture<Self, Result<()>>;

  fn handle(&mut self, msg: ShortCircuit, ctx: &mut Context<Self>) -> Self::Result {
    trace!("Short circuiting component {}", msg.reference);
    let reference = msg.reference;
    let tx_id = msg.tx_id;
    let schematic = msg.schematic;
    let payload = msg.payload;

    let outputs = self.get_outputs(&reference);
    trace!("Output ports for {} : {:?}", reference, outputs);
    let downstreams: Vec<ConnectionDefinition> = outputs
      .iter()
      .flat_map(|port| self.get_connections(&reference, port))
      .collect();
    trace!(
      "Connections to short {:?}",
      ConnectionDefinition::print_all(&downstreams)
    );
    let outputs: Vec<OutputReady> = downstreams
      .iter()
      .map(|conn| OutputReady {
        tx_id: tx_id.to_string(),
        port: PortEntity {
          reference: conn.from.instance.to_string(),
          name: conn.from.port.to_string(),
          schematic: schematic.to_string(),
        },
        payload: payload.clone(),
      })
      .collect();
    let schematic_host = ctx.address();

    let futures = outputs
      .into_iter()
      .map(move |message| schematic_host.send(message));

    Box::pin(
      async move {
        match try_join_all(futures).await {
          Ok(_) => Ok(()),
          Err(e) => Err(e.into()),
        }
      }
      .into_actor(self),
    )
  }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<()>")]
pub(crate) struct OutputReady {
  pub(crate) port: PortEntity,
  pub(crate) tx_id: String,
  pub(crate) payload: MessageTransport,
}

impl Handler<OutputReady> for Schematic {
  type Result = ResponseActFuture<Self, Result<()>>;

  fn handle(&mut self, msg: OutputReady, ctx: &mut Context<Self>) -> Self::Result {
    trace!("Output ready on {}", msg.port);
    let reference = msg.port.reference;
    let port = msg.port.name;
    let tx_id = msg.tx_id;
    let data = msg.payload;
    let schematic = msg.port.schematic;
    let defs = self.get_connections(&reference, &port);
    let addr = ctx.address();
    let task = async move {
      let origin = PortEntity {
        schematic: schematic.to_string(),
        name: port.to_string(),
        reference: reference.to_string(),
      };
      let to_message = |conn: ConnectionDefinition| PayloadReceived {
        tx_id: tx_id.clone(),
        origin: origin.clone(),
        target: PortEntity {
          schematic: schematic.to_string(),
          name: conn.to.port.to_string(),
          reference: conn.to.instance,
        },
        payload: data.clone(),
      };
      let invocations = try_join_all(defs.into_iter().map(to_message).map(|ips| addr.send(ips)));
      invocations.await?;
      Ok!(())
    }
    .into_actor(self);

    Box::pin(task)
  }
}

#[cfg(test)]
mod test {

  use vino_codec::messagepack::{
    deserialize,
    serialize,
  };

  use super::*;
  use crate::schematic_definition::{
    ComponentDefinition,
    ConnectionDefinition,
    ConnectionTargetDefinition,
    ProviderDefinition,
    ProviderKind,
  };
  use crate::util::hlreg::HostLocalSystemService;
  use crate::{
    SCHEMATIC_INPUT,
    SCHEMATIC_OUTPUT,
  };

  #[test_env_log::test(actix_rt::test)]
  async fn test_basic_schematic() -> Result<()> {
    let kp = KeyPair::new_server();
    let host_id = kp.public_key();
    let tx_id = Invocation::uuid();
    let schematic = Schematic::default();
    let addr = schematic.start();
    let schematic_name = "logger";

    let mut schematic_def = SchematicDefinition::new(schematic_name.to_string());
    schematic_def.components.insert(
      "logger".to_string(),
      ComponentDefinition {
        metadata: None,
        id: "vino::log".to_string(),
      },
    );
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: SCHEMATIC_INPUT.to_string(),
        port: "input".to_string(),
      },
      to: ConnectionTargetDefinition {
        instance: "logger".to_string(),
        port: "input".to_string(),
      },
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "logger".to_string(),
        port: "output".to_string(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_string(),
        port: "output".to_string(),
      },
    });

    addr
      .send(Initialize {
        network: Network::from_hostlocal_registry(&host_id),
        host_id: host_id.to_string(),
        schematic: schematic_def,
        seed: kp.seed()?,
        allow_latest: true,
        allowed_insecure: vec![],
      })
      .await??;
    let mut input: HashMap<String, Vec<u8>> = HashMap::new();
    let user_data = "this is test input";
    input.insert("input".to_string(), serialize(user_data)?);
    let response: InvocationResponse = addr
      .send(super::Request {
        tx_id: tx_id.to_string(),
        schematic: schematic_name.to_string(),
        payload: input,
      })
      .await??;
    match response {
      InvocationResponse::Success { msg, .. } => {
        let mut map = msg.into_output_map()?;
        let output = map.remove("output").unwrap();
        let bytes = output.into_bytes()?;
        println!("response: {:?}", map);
        let payload: String = deserialize(&bytes)?;
        println!("payload {:?}", payload);
        assert_eq!(payload, user_data);
      }
      InvocationResponse::Stream { .. } => panic!("should not have gotten a stream"),
      InvocationResponse::Error { msg, .. } => panic!("{}", msg),
    };

    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn test_native_provider() -> Result<()> {
    let schematic = Schematic::default();
    let addr = schematic.start();
    let schematic_name = "logger";

    let mut schematic_def = SchematicDefinition::new(schematic_name.to_string());
    schematic_def.providers.push(ProviderDefinition {
      namespace: "test-namespace".to_string(),
      kind: ProviderKind::Native,
      reference: "internal".to_string(),
      data: HashMap::new(),
    });
    schematic_def.components.insert(
      "logger".to_string(),
      ComponentDefinition {
        metadata: None,
        id: "test-namespace::log".to_string(),
      },
    );
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: SCHEMATIC_INPUT.to_string(),
        port: "input".to_string(),
      },
      to: ConnectionTargetDefinition {
        instance: "logger".to_string(),
        port: "input".to_string(),
      },
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "logger".to_string(),
        port: "output".to_string(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_string(),
        port: "output".to_string(),
      },
    });

    let hostkey = KeyPair::new_server();
    let host_id = KeyPair::new_server().public_key();
    let tx_id = Invocation::uuid();

    addr
      .send(Initialize {
        network: Network::from_hostlocal_registry(&host_id),
        host_id: host_id.to_string(),
        schematic: schematic_def,
        seed: hostkey.seed()?,
        allow_latest: true,
        allowed_insecure: vec![],
      })
      .await??;
    let mut input: HashMap<String, Vec<u8>> = HashMap::new();
    let user_data = "Hello world";
    input.insert("input".to_string(), serialize(user_data)?);
    let response: InvocationResponse = addr
      .send(super::Request {
        tx_id: tx_id.to_string(),
        schematic: schematic_name.to_string(),
        payload: input,
      })
      .await??;

    match response {
      InvocationResponse::Success { msg, .. } => {
        let mut map = msg.into_output_map()?;
        debug!("map: {:?}", map);
        let output = map.remove("output").unwrap();
        let bytes = output.into_bytes()?;
        println!("response: {:?}", map);
        let payload: String = deserialize(&bytes)?;
        println!("payload {:?}", payload);
        assert_eq!(payload, user_data);
      }
      InvocationResponse::Stream { .. } => panic!("should not have gotten a stream"),
      InvocationResponse::Error { msg, .. } => panic!("{}", msg),
    };
    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn test_grpc_url_provider() -> Result<()> {
    let schematic = Schematic::default();
    let addr = schematic.start();
    let schematic_name = "logger";

    let mut schematic_def = SchematicDefinition::new(schematic_name.to_string());
    schematic_def.providers.push(ProviderDefinition {
      namespace: "test-namespace".to_string(),
      kind: ProviderKind::GrpcUrl,
      reference: "http://127.0.0.1:54321".to_string(),
      data: HashMap::new(),
    });
    schematic_def.components.insert(
      "logger".to_string(),
      ComponentDefinition {
        metadata: None,
        id: "test-namespace::test-component".to_string(),
      },
    );
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: SCHEMATIC_INPUT.to_string(),
        port: "input".to_string(),
      },
      to: ConnectionTargetDefinition {
        instance: "logger".to_string(),
        port: "input".to_string(),
      },
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "logger".to_string(),
        port: "output".to_string(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_string(),
        port: "output".to_string(),
      },
    });

    let hostkey = KeyPair::new_server();
    let host_id = KeyPair::new_server().public_key();
    let tx_id = Invocation::uuid();

    addr
      .send(Initialize {
        network: Network::from_hostlocal_registry(&host_id),
        host_id: host_id.to_string(),
        schematic: schematic_def,
        seed: hostkey.seed()?,
        allow_latest: true,
        allowed_insecure: vec![],
      })
      .await??;
    let mut input: HashMap<String, Vec<u8>> = HashMap::new();
    let user_data = "Hello world";
    input.insert("input".to_string(), serialize(user_data)?);
    let response: InvocationResponse = addr
      .send(super::Request {
        tx_id: tx_id.to_string(),
        schematic: schematic_name.to_string(),
        payload: input,
      })
      .await??;

    match response {
      InvocationResponse::Success { msg, .. } => {
        let mut map = msg.into_output_map()?;
        debug!("map: {:?}", map);
        let output = map.remove("output").unwrap();
        let bytes = output.into_bytes()?;
        println!("response: {:?}", map);
        let payload: String = deserialize(&bytes)?;
        println!("payload {:?}", payload);
        assert_eq!(payload, user_data);
      }
      InvocationResponse::Stream { .. } => panic!("should not have gotten a stream"),
      InvocationResponse::Error { msg, .. } => panic!("{}", msg),
    };
    Ok(())
  }
}
