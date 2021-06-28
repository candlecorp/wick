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
use vino_codec::messagepack::serialize;
use vino_transport::MessageTransport;

use super::schematic::Schematic;
use crate::actix::ActorResult;
use crate::component_model::ComponentModel;
use crate::network_definition::NetworkDefinition;
use crate::schematic::SchematicOutput;
use crate::util::hlreg::HostLocalSystemService;
use crate::{
  Error,
  InvocationResponse,
  Result,
};

#[derive(Clone, Debug)]

pub struct Network {
  // host_labels: HashMap<String, String>,
  // kp: Option<KeyPair>,
  started: bool,
  started_time: std::time::Instant,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
  host_id: String,
  schematics: HashMap<String, Addr<Schematic>>,
  definition: NetworkDefinition,
}

impl Default for Network {
  fn default() -> Self {
    Network {
      // host_labels: HashMap::new(),
      // kp: None,
      started: false,
      started_time: std::time::Instant::now(),
      allow_latest: false,
      allowed_insecure: vec![],
      host_id: "".to_string(),
      schematics: HashMap::new(),
      definition: NetworkDefinition::default(),
    }
  }
}
impl Network {
  pub fn for_id(id: &str) -> Addr<Self> {
    Network::from_hostlocal_registry(id)
  }
  pub(crate) fn get_schematic(&self, id: &str) -> Result<Addr<Schematic>> {
    self
      .schematics
      .get(id)
      .cloned()
      .ok_or_else(|| Error::NetworkError(format!("Schematic '{}' not found", id)))
  }
  pub fn ensure_is_started(&self) -> Result<()> {
    if self.started {
      Ok(())
    } else {
      Err(Error::NetworkError("Network not started".into()))
    }
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
) -> Result<SchematicOutput> {
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
  trace!("Result: {:?}", result);
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
  type Result = ActorResult<Self, Result<()>>;

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
        let arbiter = Arbiter::new();
        let addr = Schematic::start_in_arbiter(&arbiter.handle(), |_| Schematic::default());
        addr_map.insert(definition.get_name(), addr.clone());
        init_requests.push(addr.send(super::schematic::Initialize {
          network: network_addr.clone(),
          host_id: host_id.to_string(),
          seed: seed.clone(),
          schematic: definition.clone(),
          allow_latest,
          allowed_insecure: allowed_insecure.clone(),
        }))
      }

      match try_join_all(init_requests).await {
        Ok(results) => {
          let errors: Vec<Error> = results.into_iter().filter_map(|e| e.err()).collect();
          if errors.is_empty() {
            debug!("Schematics initialized");
            Ok(addr_map)
          } else {
            Err(itertools::join(errors, ", "))
          }
        }
        Err(e) => Err(e.to_string()),
      }
    };

    ActorResult::reply_async(
      actor_fut
        .into_actor(self)
        .then(move |addr_map, network, _ctx| match addr_map {
          Ok(addr_map) => {
            network.started = true;
            network.schematics = addr_map;
            ok(())
          }
          Err(e) => err(Error::NetworkError(format!(
            "Error initializing schematics: {}",
            e
          ))),
        }),
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
  type Result = ActorResult<Self, Result<SchematicOutput>>;

  fn handle(&mut self, msg: Request, _ctx: &mut Context<Self>) -> Self::Result {
    actix_try!(self.ensure_is_started());
    let schematic_name = msg.schematic;
    trace!("Requesting schematic '{}'", schematic_name);

    let schematic = actix_try!(self.get_schematic(&schematic_name));

    let request = super::schematic::Request {
      tx_id: uuid::Uuid::new_v4().to_string(),
      schematic: schematic_name.to_string(),
      payload: msg.data,
    };

    let task = async move {
      let payload: InvocationResponse = schematic.send(request).await??;
      trace!("Schematic {} finishing", schematic_name);
      match payload {
        InvocationResponse::Success { msg, .. } => match msg {
          MessageTransport::OutputMap(map) => Ok(map),
          MessageTransport::MultiBytes(map) => Ok(
            map
              .into_iter()
              .map(|(name, payload)| (name, From::from(&payload)))
              .collect(),
          ),
          _ => unreachable!(),
        },
        InvocationResponse::Stream { .. } => unreachable!(),
        InvocationResponse::Error { msg, .. } => Err(Error::ExecutionError(msg)),
      }
    }
    .into_actor(self);

    ActorResult::reply_async(task)
  }
}

pub type MetadataMap = HashMap<String, ComponentModel>;

#[cfg(test)]
mod test {
  use std::fs;
  use std::path::{
    Path,
    PathBuf,
  };
  use std::str::FromStr;

  use actix::Addr;
  use maplit::hashmap;
  use test_env_log::test;
  use vino_codec::messagepack::serialize;
  use vino_manifest::{
    Loadable,
    NetworkManifest,
  };
  use vino_transport::MessageTransport;
  use wascap::prelude::KeyPair;

  use super::*;
  use crate::network::Initialize;
  use crate::util::hlreg::HostLocalSystemService;

  async fn init_network(path: &str) -> Result<Addr<Network>> {
    let manifest = NetworkManifest::V0(vino_manifest::v0::NetworkManifest::from_yaml(
      &fs::read_to_string(path)?,
    )?);
    println!("{:#?}", manifest);
    let def = NetworkDefinition::new(&manifest);
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
    trace!("Manifest applied");
    Ok(network)
  }

  #[test_env_log::test(actix_rt::test)]
  async fn native_component() -> Result<()> {
    let network = init_network("./manifests/native-component.yaml").await?;

    let data = hashmap! {
        "left" => 42,
        "right" => 302309,
    };

    let mut result = request(&network, "native_component", data).await?;

    println!("Result: {:?}", result);
    let output: MessageTransport = result.remove("output").unwrap();
    println!("Output: {:?}", output);
    assert_eq!(
      output,
      MessageTransport::MessagePack(serialize(42 + 302309 + 302309)?)
    );
    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn wapc_component() -> Result<()> {
    let network = init_network("./manifests/wapc-component.yaml").await?;

    let data = hashmap! {
        "input" => "1234567890",
    };

    let mut result = request(&network, "wapc_component", data).await?;

    let output: MessageTransport = result.remove("output").unwrap();
    trace!("output: {:?}", output);
    assert_eq!(
      output,
      MessageTransport::MessagePack(serialize("1234567890")?)
    );

    let data = hashmap! {
        "input" => "1234",
    };
    let mut result = request(&network, "wapc_component", data).await?;

    let output: MessageTransport = result.remove("output").unwrap();
    assert_eq!(
      output,
      MessageTransport::Exception("Needs to be longer than 8 characters".to_string())
    );

    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn short_circuit() -> Result<()> {
    let network = init_network("./manifests/short-circuit.yaml").await?;

    trace!("requesting schematic execution");
    let data = hashmap! {
        "input_port1" => "short",
    };

    let mut result = request(&network, "short_circuit", data).await?;

    println!("result: {:?}", result);
    let output1: MessageTransport = result.remove("output1").unwrap();
    println!("Output: {:?}", output1);
    assert_eq!(
      output1,
      MessageTransport::Exception("Needs to be longer than 8 characters".to_string())
    );
    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn multiple_schematics() -> Result<()> {
    let network = init_network("./manifests/multiple-schematics.yaml").await?;

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

    println!("Result: {:?}", result);
    let output: MessageTransport = result.remove("output").unwrap();
    println!("Output: {:?}", output);
    assert_eq!(
      output,
      MessageTransport::MessagePack(serialize("some string")?)
    );
    Ok(())
  }
}
