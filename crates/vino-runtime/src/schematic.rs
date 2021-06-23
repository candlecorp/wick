use std::collections::{
  HashMap,
  VecDeque,
};
use std::time::Duration;

use actix::fut::{
  err,
  ok,
};
use actix::prelude::*;
use futures::future::try_join_all;
use serde::{
  Deserialize,
  Serialize,
};
use vino_transport::MessageTransport;
use wascap::prelude::KeyPair;

use super::dispatch::{
  Invocation,
  InvocationResponse,
};
use super::schematic_response::initialize_schematic_output;
use crate::components::vino_component::load_components;
use crate::dispatch::{
  ComponentEntity,
  PortEntity,
  VinoEntity,
};
use crate::error::VinoError;
use crate::network::{
  ComponentMetadata,
  Network,
  RecordInvocationState,
};
use crate::schematic_definition::{
  ConnectionDefinition,
  ConnectionTargetDefinition,
  SchematicDefinition,
};
use crate::schematic_response::{
  get_schematic_output,
  push_to_schematic_output,
};
use crate::{
  Error,
  Result,
};
type TransactionMap = HashMap<String, InputRefMap>;
type InputRefMap = HashMap<String, BufferMap>;
type BufferMap = HashMap<String, PortBuffer>;
type PortBuffer = VecDeque<MessageTransport>;

#[derive(Debug)]
enum ComponentStatus {
  Ready(Invocation),
  Waiting,
  ShortCircuit(MessageTransport),
}

#[derive(Debug)]
pub(crate) struct Schematic {
  pub(crate) network: Option<Addr<Network>>,
  pub(crate) state: Option<SchematicState>,
  pub(crate) host_id: String,
  pub(crate) recipients: HashMap<String, Recipient<Invocation>>,
  pub(crate) seed: String,
  pub(crate) transaction_map: TransactionMap,
  pub(crate) definition: SchematicDefinition,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct SchematicState {
  pub(crate) components: HashMap<String, ComponentMetadata>,
  pub(crate) references: HashMap<String, String>,
}

impl Supervised for Schematic {}

impl Default for Schematic {
  fn default() -> Self {
    Schematic {
      network: None,
      state: None,
      host_id: "".to_string(),
      recipients: HashMap::new(),
      seed: "".to_string(),
      transaction_map: TransactionMap::new(),
      definition: SchematicDefinition::default(),
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
  fn get_downstream_recipient(&self, reference: String) -> Option<Recipient<Invocation>> {
    let state = self.state.as_ref().unwrap();
    trace!("getting downstream recipient {}", reference);
    match self.definition.get_component(&reference) {
      Some(comp) => state
        .components
        .get(&comp.id)
        .map(|component| component.addr.clone()),
      None => None,
    }
  }
  fn get_outputs(&self, reference: String) -> Vec<String> {
    let state = self.state.as_ref().unwrap();
    match state.references.get(&reference) {
      Some(actor) => match state.components.get(actor) {
        Some(metadata) => metadata.outputs.clone(),
        None => vec![],
      },
      None => vec![],
    }
  }
  fn get_connections(&self, reference: String, port: String) -> Vec<ConnectionDefinition> {
    let state = self.state.as_ref().unwrap();
    let references = &state.references;
    let connections: Vec<ConnectionDefinition> = self
      .definition
      .connections
      .iter()
      .filter(|connection| connection.from.instance == reference && connection.from.port == port)
      .filter_map(|connection| {
        let from_actor = match references.get(&connection.from.instance) {
          Some(a) => Some(a),
          None => {
            if connection.from.instance == crate::SCHEMATIC_INPUT {
              Some(&connection.from.instance)
            } else {
              return None;
            }
          }
        };
        let to_actor = match references.get(&connection.to.instance) {
          Some(a) => Some(a),
          None => {
            if connection.to.instance == crate::SCHEMATIC_OUTPUT {
              Some(&connection.to.instance)
            } else {
              return None;
            }
          }
        };
        if from_actor.is_none() || to_actor.is_none() {
          return None;
        }
        Some(ConnectionDefinition {
          from: ConnectionTargetDefinition {
            port: connection.from.port.to_string(),
            instance: connection.from.instance.to_string(),
          },
          to: ConnectionTargetDefinition {
            port: connection.to.port.to_string(),
            instance: connection.to.instance.to_string(),
          },
        })
      })
      .collect();
    connections
  }

  fn push_to_port(
    &mut self,
    tx_id: String,
    port: &PortEntity,
    data: MessageTransport,
  ) -> Result<ComponentStatus> {
    let state = self.state.as_ref().unwrap();
    let reference = port.reference.to_string();

    let kp = KeyPair::from_seed(&self.seed)?;

    let refmap = self
      .transaction_map
      .entry(tx_id.to_string())
      .or_insert_with(new_refmap);

    let component_id = state
      .references
      .get(&reference)
      .ok_or_else(|| Error::SchematicError(format!("Could not find reference {}", reference)))?;
    trace!("pushing to {}", port);
    let key = reference.to_string();
    let metadata = state.components.get(component_id).ok_or_else(|| {
      Error::SchematicError(format!(
        "Could not find metadata for {}. Component may have failed to load.",
        component_id
      ))
    })?;

    debug!("reference: {}", reference);
    debug!("refmap key: {}", key);
    refmap
      .entry(key)
      .or_insert_with(|| new_inputbuffer_map(metadata));
    debug!("port_name: {:?}", port.name);
    debug!("refmap: {:?}", refmap);

    push_to_portbuffer(refmap, reference.to_string(), port.name.clone(), data)?;

    debug!("refmap: {:?}", refmap);

    if !component_has_data(refmap, &reference) {
      return Ok(ComponentStatus::Waiting);
    }

    trace!("{} is ready to execute", reference);

    let payloads = get_component_data(refmap, &reference)?;

    let mut job_data: HashMap<String, Vec<u8>> = HashMap::new();
    for (port, payload) in payloads {
      if let MessageTransport::MessagePack(bytes) = payload {
        job_data.insert(port, bytes);
      } else {
        return Ok(ComponentStatus::ShortCircuit(payload));
      }
    }

    Ok(ComponentStatus::Ready(Invocation::next(
      &tx_id,
      &kp,
      VinoEntity::Schematic(port.schematic.to_string()),
      VinoEntity::Component(ComponentEntity {
        id: component_id.to_string(),
        name: metadata.name.to_string(),
        reference,
      }),
      MessageTransport::MultiBytes(job_data),
    )))
  }
}

fn buffer_has_data(buffer: &PortBuffer) -> bool {
  !buffer.is_empty()
}

fn component_has_data(componentref_map: &InputRefMap, reference: &str) -> bool {
  match componentref_map.get(reference) {
    Some(portbuffer_map) => portbuffer_map
      .values()
      .map(|port| buffer_has_data(port))
      .all(|has_data| has_data),
    None => false,
  }
}

fn get_component_data(
  componentref_map: &mut InputRefMap,
  reference: &str,
) -> std::result::Result<HashMap<String, MessageTransport>, &'static str> {
  match componentref_map.get_mut(reference) {
    Some(portbuffer_map) => {
      let mut next: HashMap<String, MessageTransport> = HashMap::new();
      for (key, buffer) in portbuffer_map.iter_mut() {
        if let Some(value) = buffer.pop_front() {
          next.insert(key.to_string(), value);
        } else {
          return Err("Buffer not actually ready");
        }
      }
      Ok(next)
    }
    None => Err("Could not get buffer map"),
  }
}

fn new_refmap() -> InputRefMap {
  InputRefMap::new()
}

fn new_inputbuffer_map(metadata: &ComponentMetadata) -> BufferMap {
  trace!("creating new inputbuffer map for {:?}", metadata);
  metadata
    .inputs
    .iter()
    .map(|p| (p.to_string(), VecDeque::new()))
    .collect()
}

fn push_to_portbuffer(
  component_ref_map: &mut InputRefMap,
  ref_id: String,
  port: String,
  data: MessageTransport,
) -> Result<()> {
  match component_ref_map.get_mut(&ref_id) {
    Some(portbuffer_map) => {
      trace!("Getting portbuffer for port {:?}", port);
      match portbuffer_map.get_mut(&port) {
        Some(buffer) => {
          buffer.push_back(data);
          Ok(())
        }
        None => Err(Error::SchematicError(format!(
          "Invalid actor state: no portbuffer for port {:?}",
          port
        ))),
      }
    }
    None => Err(Error::SchematicError(format!(
      "Could not get portbuffer map for reference {}",
      ref_id
    ))),
  }
}

#[derive(Message, Debug, Clone, new)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) schematic: SchematicDefinition,
  pub(crate) host_id: String,
  pub(crate) seed: String,
  pub(crate) network: Addr<Network>,
  pub(crate) allow_latest: bool,
  pub(crate) allowed_insecure: Vec<String>,
  // pub(crate) components: MetadataMap,
}

impl Handler<Initialize> for Schematic {
  type Result = ResponseActFuture<Self, Result<()>>;

  fn handle(&mut self, msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
    trace!("Initializing schematic {}", msg.schematic.get_name());
    let seed = msg.seed;
    let allow_latest = msg.allow_latest;
    let allowed_insecure = msg.allowed_insecure;
    let definition = msg.schematic;
    self.seed = seed.clone();
    self.definition = definition;

    self.host_id = msg.host_id;
    self.network = Some(msg.network);

    let mut state = SchematicState::default();

    self
      .definition
      .components
      .iter()
      .for_each(|(instance, actor)| {
        state
          .references
          .insert(instance.to_string(), actor.id.to_string());
      });

    self.state = Some(state);

    Box::pin(
      load_components(
        self.definition.clone(),
        seed,
        allow_latest,
        allowed_insecure,
      )
      .into_actor(self)
      .then(|components, schematic, _ctx| {
        let state = schematic.state.as_mut().unwrap();
        match components {
          Ok(components) => {
            state.components = components;
            ok(())
          }
          Err(e) => {
            error!("{:?}", e);
            err(e)
          }
        }
      }),
    )
  }
}

#[derive(Message, Debug, Clone, new)]
#[rtype(result = "Result<()>")]
pub(crate) struct InitializeProviders {
  pub(crate) schematic: SchematicDefinition,
  pub(crate) host_id: String,
  pub(crate) seed: String,
}

impl Handler<InitializeProviders> for Schematic {
  type Result = ResponseActFuture<Self, Result<()>>;

  fn handle(&mut self, msg: InitializeProviders, _ctx: &mut Self::Context) -> Self::Result {
    trace!(
      "Starting providers for schematic {}",
      msg.schematic.get_name()
    );

    // TODO:
    // - Add provider handler that starts an arbiter
    // - Add control layer to GrpcProviders so we can query information
    //     like what components are handled and what ports those components have

    // let schematic = msg.schematic;

    // struct ProviderHandler {}

    // let mut providers = vec![];

    // for provider in &schematic.providers {
    //   let namespace = provider.namespace;
    //   match provider.kind {
    //     ProviderKind::Native => {
    //       trace!(
    //         "registering component under the {} provider namespace",
    //         namespace
    //       );

    //       let component = NativeComponent::from_id(namespace, name)?;
    //       trace!("Starting native component '{}'", component.name(),);
    //       let arbiter = Arbiter::new();
    //       let actor = NativeComponentActor::start_in_arbiter(&arbiter.handle(), |_| {
    //         NativeComponentActor::default()
    //       });
    //       actor
    //         .send(native_component_actor::Initialize {
    //           name: component.name(),
    //           signing_seed: seed,
    //         })
    //         .await??;
    //       let recipient = actor.recipient::<Invocation>();

    //       return Ok((Box::new(component), recipient));
    //     }
    //     ProviderKind::GrpcUrl => {}
    //   }
    // }
    // let seed = msg.seed;

    Box::pin(ok(()).into_actor(self))
  }
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<SchematicResponse>")]
pub(crate) struct Request {
  pub(crate) tx_id: String,
  pub(crate) schematic: String,
  pub(crate) payload: HashMap<String, Vec<u8>>,
}

#[derive(Debug)]
pub(crate) struct SchematicResponse {
  pub(crate) payload: Vec<u8>,
}

impl Handler<Request> for Schematic {
  type Result = ResponseActFuture<Self, Result<SchematicResponse>>;

  fn handle(&mut self, msg: Request, ctx: &mut Context<Self>) -> Self::Result {
    trace!("Requesting schematic '{}'", msg.schematic);
    let tx_id = msg.tx_id.to_string();
    let schematic = msg.schematic.to_string();

    let invocations = gen_packets(self, tx_id.to_string(), schematic.to_string(), msg.payload);

    let host = ctx.address();

    Box::pin(
      async move {
        let response = handle_schematic_invocation(invocations?, host, tx_id, schematic).await?;
        Ok(SchematicResponse {
          payload: response.msg,
        })
      }
      .into_actor(self),
    )
  }
}

impl Handler<ResponseFuture> for Schematic {
  type Result = ResponseActFuture<Self, Result<InvocationResponse>>;

  fn handle(&mut self, msg: ResponseFuture, _ctx: &mut Context<Self>) -> Self::Result {
    trace!(
      "Requesting future for schematic '{}' on tx {}",
      msg.schematic,
      msg.tx_id
    );

    let tx_id = msg.tx_id;

    let schematic_name = msg.schematic;
    let timeout = Duration::from_millis(1000);
    let schematic = get_schematic_output(&tx_id, &schematic_name);

    let task = async move {
      match schematic {
        Ok(future) => match actix_rt::time::timeout(timeout, future).await {
          Ok(r) => Ok(r),
          Err(e) => Ok(InvocationResponse::error(
            tx_id,
            format!("Error waiting for schematic output {}", e.to_string()),
          )),
        },
        Err(e) => Ok(InvocationResponse::error(tx_id, e.to_string())),
      }
    }
    .into_actor(self);

    Box::pin(task)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Message, PartialEq)]
#[rtype(result = "Result<()>")]
struct PayloadReceived {
  tx_id: String,
  origin: VinoEntity,
  target: PortEntity,
  payload: MessageTransport,
}

fn gen_packets(
  sm: &mut Schematic,
  tx_id: String,
  name: String,
  bytemap: HashMap<String, Vec<u8>>,
) -> Result<Vec<PayloadReceived>> {
  let schematic = &sm.definition;
  let _kp = KeyPair::from_seed(&sm.seed)?;

  let schematic_outputs = schematic.get_output_names();

  initialize_schematic_output(&tx_id, &name, schematic_outputs);

  let invocations: Vec<PayloadReceived> = schematic
    .connections
    .iter()
    .filter(|conn| conn.from.instance == crate::SCHEMATIC_INPUT)
    .map(|conn| {
      let bytes = bytemap
        .get(&conn.from.port)
        .unwrap_or_else(|| panic!("Output on port '{}' not found", conn.to.instance));

      PayloadReceived {
        target: PortEntity {
          schematic: name.to_string(),
          name: conn.to.port.to_string(),
          reference: conn.to.instance.to_string(),
        },
        origin: VinoEntity::Schematic(name.to_string()),
        tx_id: tx_id.to_string(),
        payload: MessageTransport::MessagePack(bytes.clone()),
      }
    })
    .collect();
  Ok(invocations)
}

async fn handle_schematic_invocation(
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
    .send(ResponseFuture {
      tx_id: tx_id.to_string(),
      schematic: target,
    })
    .await
    .map_err(|e| Error::SchematicError(format!("Error pushing to schematic ports: {}", e)))??;

  Ok(response)
}

impl Handler<PayloadReceived> for Schematic {
  type Result = ResponseActFuture<Self, Result<()>>;

  fn handle(&mut self, msg: PayloadReceived, ctx: &mut Context<Self>) -> Self::Result {
    let name = self.definition.get_name();
    let port = msg.target;
    let payload = msg.payload;
    let tx_id = msg.tx_id;
    trace!("Receiving on port {}", port);

    let reference = port.reference.to_string();
    //TODO normalize output to the same buffers as regular ports
    if reference == crate::SCHEMATIC_OUTPUT {
      return Box::pin(
        async move {
          push_to_schematic_output(&tx_id, &name, &port.name, payload)?;
          Ok(())
        }
        .into_actor(self),
      );
    }
    let status = self.push_to_port(tx_id.to_string(), &port, payload);
    let schematic_host = ctx.address();

    let recipient = self.get_downstream_recipient(reference.to_string());
    let network = self.network.clone().unwrap();

    Box::pin(
      async move {
        match status {
          Err(err) => {
            log_err!(Error::SchematicError(format!("Error handling IP: {}", err)))
          }
          Ok(ComponentStatus::ShortCircuit(payload)) => match schematic_host
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

          Ok(ComponentStatus::Waiting) => {
            trace!("Component {} is still waiting on data", reference);
            Ok(())
          }
          Ok(ComponentStatus::Ready(invocation)) => {
            network.do_send(RecordInvocationState {
              inv_id: invocation.id.to_string(),
              tx_id: tx_id.clone(),
              schematic: name.to_string(),
              entity: invocation.target.clone(),
            });

            let response: Result<()> = match recipient {
              Some(recipient) => match recipient.send(invocation).await {
                Ok(_) => Ok(()),
                Err(err) => Err(err.into()),
              },
              None => Err("No receiver found".into()),
            };

            match response {
              Ok(_signal) => Ok(()),
              Err(err) => {
                warn!(
                  "Tx '{}': schematic '{}' short-circuiting from '{}': {}",
                  tx_id,
                  name,
                  reference,
                  err.to_string()
                );
                schematic_host
                  .send(ShortCircuit {
                    tx_id: tx_id.to_string(),
                    schematic: name.to_string(),
                    reference: reference.to_string(),
                    payload: MessageTransport::Error(err.to_string()),
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

    let outputs = self.get_outputs(reference.to_string());
    let downstreams: Vec<ConnectionDefinition> = outputs
      .iter()
      .flat_map(|port| self.get_connections(reference.to_string(), port.to_string()))
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
    let seed = self.seed.to_string();

    let reference = msg.port.reference;
    let port = msg.port.name;
    let tx_id = msg.tx_id;

    let data = msg.payload;
    let schematic = msg.port.schematic;

    let defs = self.get_connections(reference.to_string(), port.to_string());
    let addr = ctx.address();
    let task = async move {
      let _kp = KeyPair::from_seed(&seed).unwrap();
      let origin = VinoEntity::Port(PortEntity {
        schematic: schematic.to_string(),
        name: port.to_string(),
        reference: reference.to_string(),
      });
      let make_packet = |conn: ConnectionDefinition| {
        let entity = PortEntity {
          schematic: schematic.to_string(),
          name: conn.to.port.to_string(),
          reference: conn.to.instance,
        };
        PayloadReceived {
          tx_id: tx_id.clone(),
          origin: origin.clone(),
          target: entity,
          payload: data.clone(),
        }
      };
      let invocations = try_join_all(defs.into_iter().map(make_packet).map(|ips| addr.send(ips)));
      invocations.await?;
      Ok::<(), VinoError>(())
    }
    .into_actor(self);

    Box::pin(task)
  }
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<InvocationResponse>")]
pub(crate) struct ResponseFuture {
  pub(crate) schematic: String,
  pub(crate) tx_id: String,
}

#[cfg(test)]
mod test {

  use super::*;
  use crate::schematic_definition::{
    ComponentDefinition,
    ConnectionDefinition,
    ConnectionTargetDefinition,
  };
  use crate::util::hlreg::HostLocalSystemService;

  #[test_env_log::test(actix_rt::test)]
  async fn test_init() -> Result<()> {
    trace!("test_init");
    let schematic = Schematic::default();
    let addr = schematic.start();
    let mut schematic_def = SchematicDefinition::default();
    schematic_def.components.insert(
      "logger".to_string(),
      ComponentDefinition {
        metadata: None,
        id: "vino::log".to_string(),
      },
    );
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "vino::schematic".to_string(),
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
        instance: "vino::schematic".to_string(),
        port: "output".to_string(),
      },
    });

    let hostkey = KeyPair::new_server();

    addr
      .send(Initialize {
        network: Network::from_hostlocal_registry(&KeyPair::new_server().public_key()),
        host_id: "test".to_string(),
        schematic: schematic_def,
        seed: hostkey.seed()?,
        allow_latest: true,
        allowed_insecure: vec![],
      })
      .await??;
    let mut input: HashMap<String, Vec<u8>> = HashMap::new();
    input.insert("input".to_string(), vec![20]);
    let response = addr
      .send(super::Request {
        tx_id: "some_id".to_string(),
        schematic: "logger".to_string(),
        payload: input,
      })
      .await?;
    println!("{:?}", response);

    Ok(())
  }
}
