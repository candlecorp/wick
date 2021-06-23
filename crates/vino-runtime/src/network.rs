use std::collections::HashMap;
use std::fmt::Display;

use actix::dev::Message;
use actix::fut::{
  err,
  ok,
};
use actix::prelude::*;
use futures::future::try_join_all;
use serde::Serialize;
use vino_codec::messagepack::{
  deserialize,
  serialize,
};
use vino_component::v0::Payload;
use vino_component::Output;
use vino_transport::MessageTransport;

use super::schematic::Schematic;
use crate::components::vino_component::BoxedComponent;
use crate::components::{
  Inputs,
  Outputs,
};
use crate::dispatch::{
  PortEntity,
  VinoEntity,
};
use crate::network_definition::NetworkDefinition;
use crate::schematic::OutputReady;
use crate::util::hlreg::HostLocalSystemService;
use crate::{
  Error,
  Invocation,
  Result,
};

#[derive(Clone, Debug)]

pub struct Network {
  // host_labels: HashMap<String, String>,
  // kp: Option<KeyPair>,
  // started: std::time::Instant,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
  registry: ComponentRegistry,
  host_id: String,
  schematics: HashMap<String, Addr<Schematic>>,
  definition: NetworkDefinition,
  invocation_map: HashMap<String, (String, String, VinoEntity)>,
}

impl Default for Network {
  fn default() -> Self {
    Network {
      // host_labels: HashMap::new(),
      // kp: None,
      // started: std::time::Instant::now(),
      allow_latest: false,
      allowed_insecure: vec![],
      registry: ComponentRegistry::default(),
      host_id: "".to_string(),
      schematics: HashMap::new(),
      definition: NetworkDefinition::default(),
      invocation_map: HashMap::new(),
    }
  }
}

impl Network {
  pub fn for_id(id: &str) -> Addr<Self> {
    Network::from_hostlocal_registry(id)
  }
}

impl Supervised for Network {}

impl SystemService for Network {
  fn service_started(&mut self, ctx: &mut Context<Self>) {
    trace!("Network started");
    ctx.set_mailbox_capacity(1000);
  }
}

impl HostLocalSystemService for Network {}

impl Actor for Network {
  type Context = Context<Self>;
}

pub async fn request<T: AsRef<str> + Display>(
  network: &Addr<Network>,
  schematic: &str,
  data: HashMap<T, impl Serialize>,
) -> Result<HashMap<String, MessageTransport>> {
  let serialized_data: HashMap<String, Vec<u8>> = data
    .iter()
    .map(|(k, v)| Ok((k.to_string(), serialize(&v)?)))
    .filter_map(Result::ok)
    .collect();

  let time = std::time::Instant::now();
  let result = network
    .send(Request {
      schematic: schematic.to_string(),
      data: serialized_data,
    })
    .await??;
  trace!(
    "result for {} took {} μs",
    schematic,
    time.elapsed().as_micros()
  );
  trace!("{:?}", result);
  Ok(result)
}

#[derive(Message, Debug)]
#[rtype(result = "Result<()>")]
pub struct Initialize {
  pub host_id: String,
  pub seed: String,
  pub network: NetworkDefinition,
}

impl Handler<Initialize> for Network {
  type Result = ResponseActFuture<Self, Result<()>>;

  fn handle(&mut self, msg: Initialize, ctx: &mut Context<Self>) -> Self::Result {
    trace!("Network initializing on {}", msg.host_id);
    self.host_id = msg.host_id;
    self.definition = msg.network;

    let host_id = self.host_id.to_string();
    let seed = msg.seed;
    let allow_latest = self.allow_latest;
    let allowed_insecure = self.allowed_insecure.clone();
    let schematics = self.definition.schematics.clone();
    let network_addr = ctx.address();

    let actor_fut = async move {
      let mut init_requests = vec![];
      let mut addr_map = HashMap::new();

      for definition in schematics {
        let schematic = Schematic::default().start();
        addr_map.insert(definition.get_name(), schematic.clone());
        init_requests.push(schematic.send(super::schematic::Initialize {
          network: network_addr.clone(),
          host_id: host_id.to_string(),
          seed: seed.clone(),
          schematic: definition.clone(),
          allow_latest,
          allowed_insecure: allowed_insecure.clone(),
        }))
      }
      try_join_all(init_requests).await?;
      Ok!(addr_map)
    };

    Box::pin(
      actor_fut
        .into_actor(self)
        .then(move |addr_map, network, _ctx| match addr_map {
          Ok(addr_map) => {
            network.schematics = addr_map;
            ok(())
          }
          Err(e) => err(Error::NetworkError(format!(
            "Error initializing schematics {}",
            e.to_string()
          ))),
        }),
    )
  }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "Option<(String, String, VinoEntity)>")]
pub(crate) struct GetReference {
  pub(crate) inv_id: String,
}

impl Handler<GetReference> for Network {
  type Result = Option<(String, String, VinoEntity)>;

  fn handle(&mut self, msg: GetReference, _ctx: &mut Context<Self>) -> Self::Result {
    self.invocation_map.get(&msg.inv_id).cloned()
  }
}

#[derive(Message)]
#[rtype(result = "()")]
pub(crate) struct RecordInvocationState {
  pub(crate) inv_id: String,
  pub(crate) tx_id: String,
  pub(crate) schematic: String,
  pub(crate) entity: VinoEntity,
}

impl Handler<RecordInvocationState> for Network {
  type Result = ();

  fn handle(&mut self, msg: RecordInvocationState, _ctx: &mut Context<Self>) -> Self::Result {
    self
      .invocation_map
      .insert(msg.inv_id, (msg.tx_id, msg.schematic, msg.entity));
  }
}

impl Handler<OutputReady> for Network {
  type Result = ResponseActFuture<Self, Result<()>>;

  fn handle(&mut self, msg: OutputReady, _ctx: &mut Context<Self>) -> Self::Result {
    let schematic_name = msg.port.schematic.to_string();
    let receiver = self.schematics.get(&schematic_name).cloned();
    Box::pin(
      async move {
        match receiver {
          Some(schematic) => Ok(schematic.send(msg).await??),
          None => Err(Error::NetworkError("Failed to propagate output".into())),
        }
      }
      .into_actor(self),
    )
  }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<()>")]
pub(crate) struct WapcOutputReady {
  pub(crate) port: String,
  pub(crate) invocation_id: String,
  pub(crate) payload: Vec<u8>,
}

impl Handler<WapcOutputReady> for Network {
  type Result = ResponseActFuture<Self, Result<()>>;

  fn handle(&mut self, msg: WapcOutputReady, _ctx: &mut Context<Self>) -> Self::Result {
    let metadata = self.invocation_map.get(&msg.invocation_id).cloned();

    let (tx_id, schematic_name, entity) = metadata.unwrap();

    let receiver = self.schematics.get(&schematic_name).cloned();
    let payload = msg.payload;
    let port = msg.port;
    // we won't get Serializable values from WaPC, this is to satisfy serde and the typechecker.
    type Unnecessary = String;
    debug!("Payload: {:?}", payload);
    let message_payload: MessageTransport = match deserialize::<Output<Unnecessary>>(&payload) {
      Ok(payload) => match payload {
        Output::V0(v0) => match v0 {
          Payload::Invalid => MessageTransport::Invalid,
          Payload::MessagePack(bytes) => MessageTransport::MessagePack(bytes),
          Payload::Exception(msg) => MessageTransport::Exception(msg),
          Payload::Error(msg) => MessageTransport::Error(msg),
          Payload::Serializable(_) => MessageTransport::Invalid,
        },
      },
      Err(err) => MessageTransport::Error(format!(
        "Deserialization error for {}: {}",
        port,
        err.to_string()
      )),
    };

    Box::pin(
      async move {
        let port = PortEntity {
          name: port,
          reference: entity.into_component()?.reference,
          schematic: schematic_name,
        };
        match receiver {
          Some(schematic) => Ok(
            schematic
              .send(OutputReady {
                port,
                tx_id,
                payload: message_payload,
              })
              .await??,
          ),
          None => Err(Error::NetworkError(
            "Failed to propagate WASM component output".into(),
          )),
        }
      }
      .into_actor(self),
    )
  }
}

#[derive(Message)]
#[rtype(result = "Result<HashMap<String, MessageTransport>>")]
pub(crate) struct Request {
  pub(crate) schematic: String,
  pub(crate) data: HashMap<String, Vec<u8>>,
}

impl Handler<Request> for Network {
  type Result = ResponseActFuture<Self, Result<HashMap<String, MessageTransport>>>;

  fn handle(&mut self, msg: Request, _ctx: &mut Context<Self>) -> Self::Result {
    trace!("Requesting schematic '{}'", msg.schematic);
    let schematic_name = msg.schematic;
    let payload = msg.data;

    let tx_id = uuid::Uuid::new_v4();
    trace!("Invoking schematic '{}'", schematic_name);
    let schematic = self
      .schematics
      .get(&schematic_name)
      .cloned()
      .ok_or_else(|| Error::NetworkError(format!("Schematic '{}' not found", schematic_name)));

    let request = super::schematic::Request {
      tx_id: tx_id.to_string(),
      schematic: schematic_name,
      payload,
    };

    Box::pin(
      async move {
        let payload = schematic?.send(request).await??;
        Ok(deserialize(&payload.payload)?)
      }
      .into_actor(self),
    )
  }
}

#[derive(Default, Clone)]
pub struct ComponentRegistry {
  pub(crate) components: HashMap<String, BoxedComponent>,
  pub(crate) receivers: HashMap<String, Recipient<Invocation>>,
}

impl std::fmt::Debug for ComponentRegistry {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ComponentRegistry")
      .field("components", &self.components.keys())
      .field("receivers", &self.receivers.keys())
      .finish()
  }
}

#[derive(Debug, Clone)]
pub struct ComponentMetadata {
  pub name: String,
  pub inputs: Inputs,
  pub outputs: Outputs,
  pub addr: Recipient<Invocation>,
}

pub type MetadataMap = HashMap<String, ComponentMetadata>;

impl ComponentRegistry {}

#[cfg(test)]
mod test {
  use actix::Addr;
  use maplit::hashmap;
  use test_env_log::test;
  use vino_codec::messagepack::serialize;
  use vino_manifest::NetworkManifest;
  use vino_transport::MessageTransport;
  use wascap::prelude::KeyPair;

  use super::*;
  use crate::network::Initialize;
  use crate::util::hlreg::HostLocalSystemService;

  async fn init_network(yaml: &str) -> Result<Addr<Network>> {
    let def = NetworkDefinition::new(&NetworkManifest::V0(serde_yaml::from_str(yaml)?));
    debug!("Manifest loaded");
    let kp = KeyPair::new_server();

    let network = super::Network::from_hostlocal_registry(&kp.public_key());
    network
      .send(Initialize {
        host_id: kp.public_key(),
        seed: kp.seed()?,
        network: def,
      })
      .await??;
    trace!("manifest applied");
    Ok(network)
  }

  #[test_env_log::test(actix_rt::test)]
  async fn native_component() -> Result<()> {
    let network = init_network(include_str!("./test/native-component.yaml")).await?;

    let data = hashmap! {
        "left" => 42,
        "right" => 302309,
    };

    let mut result = request(&network, "test", data).await?;

    trace!("result: {:?}", result);
    let output: MessageTransport = result.remove("output").unwrap();
    assert_eq!(
      output,
      MessageTransport::MessagePack(serialize(42 + 302309 + 302309)?)
    );
    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn wapc_component() -> Result<()> {
    let network = init_network(include_str!("./test/wapc-component.yaml")).await?;

    let data = hashmap! {
        "input" => "1234567890",
    };

    let mut result = request(&network, "test", data).await?;

    let output: MessageTransport = result.remove("output").unwrap();
    trace!("output: {:?}", output);
    assert_eq!(
      output,
      MessageTransport::MessagePack(serialize("1234567890")?)
    );

    let data = hashmap! {
        "input" => "1234",
    };
    let mut result = request(&network, "test", data).await?;

    let output: MessageTransport = result.remove("output").unwrap();
    assert_eq!(
      output,
      MessageTransport::Exception("Needs to be longer than 8 characters".to_string())
    );

    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn short_circuit() -> Result<()> {
    let network = init_network(include_str!("./test/short-circuit.yaml")).await?;

    trace!("requesting schematic execution");
    let data = hashmap! {
        "input_port1" => "short",
    };

    let mut result = request(&network, "test", data).await?;

    trace!("result: {:?}", result);
    let output1: MessageTransport = result.remove("output1").unwrap();
    assert_eq!(
      output1,
      MessageTransport::Exception("Needs to be longer than 8 characters".to_string())
    );
    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn multiple_schematics() -> Result<()> {
    let network = init_network(include_str!("./test/multiple-schematics.yaml")).await?;

    let data = hashmap! {
        "left" => 42,
        "right" => 302309,
    };

    let mut result = request(&network, "first_schematic", data).await?;

    trace!("result: {:?}", result);
    let output: MessageTransport = result.remove("output").unwrap();
    assert_eq!(
      output,
      MessageTransport::MessagePack(serialize(42 + 302309)?)
    );

    let data = hashmap! {
        "input" => "some string",
    };

    let mut result = request(&network, "second_schematic", data).await?;

    trace!("result: {:?}", result);
    let output: MessageTransport = result.remove("output").unwrap();
    assert_eq!(
      output,
      MessageTransport::MessagePack(serialize("some string")?)
    );
    Ok(())
  }
}
