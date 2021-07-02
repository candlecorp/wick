use std::collections::HashMap;

use actix::dev::Message;
use actix::fut::{
  err,
  ok,
};
use futures::future::try_join_all;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use vino_rpc::SchematicSignature;

use crate::dev::prelude::*;
use crate::dispatch::inv_error;
use crate::provider_model::{
  create_network_provider_channel,
  create_network_provider_model,
};
use crate::schematic::{
  GetSignature,
  ProviderInitResponse,
};

type Result<T> = std::result::Result<T, NetworkError>;
#[derive(Clone, Debug)]

pub(crate) struct NetworkService {
  // host_labels: HashMap<String, String>,
  // kp: Option<KeyPair>,
  started: bool,
  started_time: std::time::Instant,
  id: String,
  schematics: HashMap<String, Addr<Schematic>>,
  definition: NetworkDefinition,
}

impl Default for NetworkService {
  fn default() -> Self {
    NetworkService {
      // host_labels: HashMap::new(),
      // kp: None,
      started: false,
      started_time: std::time::Instant::now(),
      id: "".to_owned(),
      schematics: HashMap::new(),
      definition: NetworkDefinition::default(),
    }
  }
}

type ServiceMap = HashMap<String, Addr<NetworkService>>;
static HOST_REGISTRY: Lazy<Mutex<ServiceMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

impl NetworkService {
  pub(crate) fn for_id(network_id: &str) -> Addr<Self> {
    trace!("getting network for host {}", network_id);
    let sys = System::current();
    let mut registry = HOST_REGISTRY.lock();
    let addr = registry
      .entry(network_id.to_owned())
      .or_insert_with(|| NetworkService::start_service(sys.arbiter()));

    addr.clone()
  }
  pub(crate) fn get_schematic(&self, id: &str) -> Result<Addr<Schematic>> {
    self
      .schematics
      .get(id)
      .cloned()
      .ok_or_else(|| NetworkError::SchematicNotFound(id.to_owned()))
  }
  pub(crate) fn ensure_is_started(&self) -> Result<()> {
    if self.started {
      Ok(())
    } else {
      Err(NetworkError::NotStarted)
    }
  }
}

impl Supervised for NetworkService {}

impl SystemService for NetworkService {
  fn service_started(&mut self, ctx: &mut Context<Self>) {
    trace!("Network started");
    ctx.set_mailbox_capacity(1000);
  }
}

impl Actor for NetworkService {
  type Context = Context<Self>;
}

#[derive(Message, Debug)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) network_id: String,
  pub(crate) seed: String,
  pub(crate) network: NetworkDefinition,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) allow_latest: bool,
}

impl Handler<Initialize> for NetworkService {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: Initialize, _ctx: &mut Context<Self>) -> Self::Result {
    trace!("Network initializing on {}", msg.network_id);
    self.started = true;

    self.id = msg.network_id.clone();
    let network_id = msg.network_id;
    self.definition = msg.network;
    let allowed_insecure = msg.allowed_insecure;

    let seed = msg.seed;
    let allow_latest = msg.allow_latest;
    let schematics = self.definition.schematics.clone();

    let actor_fut = async move {
      let mut init_requests = vec![];
      let mut addr_map = HashMap::new();
      let channel = create_network_provider_channel("self", network_id).await?;

      for definition in schematics {
        let arbiter = Arbiter::new();
        let addr = Schematic::start_in_arbiter(&arbiter.handle(), |_| Schematic::default());
        addr_map.insert(definition.get_name(), addr.clone());
        init_requests.push(addr.send(super::schematic::Initialize {
          seed: seed.clone(),
          network_provider_channel: Some(channel.clone()),
          schematic: definition.clone(),
          allow_latest,
          allowed_insecure: allowed_insecure.clone(),
        }));
      }

      let results = try_join_all(init_requests)
        .await
        .map_err::<NetworkError, _>(|_| InternalError(5001).into())?;

      let errors: Vec<SchematicError> = results
        .into_iter()
        .filter_map(std::result::Result::err)
        .collect();
      if errors.is_empty() {
        debug!("Schematics initialized");
        Ok(addr_map)
      } else {
        Err(NetworkError::InitializationError(itertools::join(
          errors, ", ",
        )))
      }
    };

    ActorResult::reply_async(
      actor_fut
        .into_actor(self)
        .then(move |result, network, _ctx| {
          result.map_or_else(err, |addr_map| {
            network.schematics = addr_map;
            ok(())
          })
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

impl Handler<Request> for NetworkService {
  type Result = ActorResult<Self, Result<HashMap<String, MessageTransport>>>;

  fn handle(&mut self, msg: Request, _ctx: &mut Context<Self>) -> Self::Result {
    actix_try!(self.ensure_is_started());
    let schematic_name = msg.schematic;
    trace!("Requesting schematic '{}'", schematic_name);

    let schematic = actix_try!(self.get_schematic(&schematic_name));

    let request = super::schematic::Request {
      tx_id: uuid::Uuid::new_v4().to_string(),
      schematic: schematic_name.clone(),
      payload: msg.data,
    };

    let task = async move {
      let payload: InvocationResponse = schematic
        .send(request)
        .await
        .map_err(|_| InternalError(5003))??;
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
        InvocationResponse::Stream { mut rx, .. } => {
          debug!("Got stream");
          let mut map = HashMap::new();

          while let Some(next) = rx.next().await {
            debug!("Received packet on port {}: {:?}", next.port, next.payload);
            map.insert(next.port, next.payload.into());
          }
          Ok(map)
        }
        InvocationResponse::Error { msg, .. } => Err(NetworkError::ExecutionError(msg)),
      }
    }
    .into_actor(self);

    ActorResult::reply_async(task)
  }
}

#[derive(Message)]
#[rtype(result = "Result<Vec<SchematicSignature>>")]
pub(crate) struct ListSchematics {}

impl Handler<ListSchematics> for NetworkService {
  type Result = ActorResult<Self, Result<Vec<SchematicSignature>>>;

  fn handle(&mut self, _msg: ListSchematics, _ctx: &mut Context<Self>) -> Self::Result {
    actix_try!(self.ensure_is_started());
    let schematics = self.schematics.clone();
    let requests = schematics
      .into_values()
      .map(|addr| addr.send(GetSignature {}));
    type SchematicResult<T> = std::result::Result<T, SchematicError>;
    let task = async move {
      let results: Vec<SchematicResult<SchematicSignature>> = try_join_all(requests)
        .await
        .map_err(|_| InternalError(5004))?;

      Ok(results.into_iter().map(SchematicResult::unwrap).collect())
    }
    .into_actor(self);

    ActorResult::reply_async(task)
  }
}

impl Handler<Invocation> for NetworkService {
  type Result = ActorResult<Self, InvocationResponse>;

  fn handle(&mut self, msg: Invocation, _ctx: &mut Context<Self>) -> Self::Result {
    let tx_id = msg.tx_id;
    actix_ensure_ok!(self
      .ensure_is_started()
      .map_err(|e| inv_error(&tx_id, &e.to_string())));
    let schematic_name = actix_ensure_ok!(msg
      .target
      .into_schematic()
      .map_err(|_| inv_error(&tx_id, "Sent invalid entity")));

    trace!("Invoking schematic '{}'", schematic_name);

    let schematic = actix_ensure_ok!(self
      .get_schematic(&schematic_name)
      .map_err(|e| inv_error(&tx_id, &e.to_string())));
    let payload = actix_ensure_ok!(msg
      .msg
      .into_multibytes()
      .map_err(|e| inv_error(&tx_id, &e.to_string())));

    let request = super::schematic::Request {
      tx_id: tx_id.clone(),
      schematic: schematic_name,
      payload,
    };

    let task = async move {
      match schematic.send(request).await {
        Ok(Ok(response)) => response,
        Ok(Err(e)) => InvocationResponse::error(tx_id, format!("Error invoking schematic: {}", e)),
        Err(e) => {
          InvocationResponse::error(tx_id, format!("Internal error invoking schematic: {}", e))
        }
      }
    }
    .into_actor(self);

    ActorResult::reply_async(task)
  }
}

#[cfg(test)]
mod test {

  use vino_transport::MessageTransport;

  use super::*;
  use crate::test::prelude::*;

  #[test_env_log::test(actix_rt::test)]
  async fn simple_schematic() -> TestResult<()> {
    let (network, _) = init_network_from_yaml("./manifests/simple.yaml").await?;

    let data = hashmap! {
        "input" => "simple string",
    };

    let mut result = network.request("simple", &data).await?;

    println!("Result: {:?}", result);
    let output: MessageTransport = result.remove("output").unwrap();
    println!("Output: {:?}", output);
    equals!(
      output,
      MessageTransport::MessagePack(mp_serialize("simple string")?)
    );
    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn native_component() -> TestResult<()> {
    let (network, _) = init_network_from_yaml("./manifests/native-component.yaml").await?;

    let data = hashmap! {
        "left" => 42,
        "right" => 302309,
    };

    let mut result = network.request("native_component", &data).await?;

    println!("Result: {:?}", result);
    let output: MessageTransport = result.remove("output").unwrap();
    println!("Output: {:?}", output);
    equals!(
      output,
      MessageTransport::MessagePack(mp_serialize(42 + 302309 + 302309)?)
    );
    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn nested_schematics() -> TestResult<()> {
    let (network, _) = init_network_from_yaml("./manifests/nested-schematics.yaml").await?;

    let user_data = "user inputted data";

    let data = hashmap! {
        "parent_input" => user_data,
    };

    let mut result = network.request("parent", &data).await?;

    println!("Result: {:?}", result);
    let output: MessageTransport = result.remove("parent_output").unwrap();
    println!("Output: {:?}", output);
    equals!(
      output,
      MessageTransport::MessagePack(mp_serialize(42 + 302309 + 302309)?)
    );
    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn wapc_component() -> TestResult<()> {
    let (network, _) = init_network_from_yaml("./manifests/wapc-component.yaml").await?;

    let data = hashmap! {
        "input" => "1234567890",
    };

    let mut result = network.request("wapc_component", &data).await?;

    let output: MessageTransport = result.remove("output").unwrap();
    trace!("output: {:?}", output);
    equals!(
      output,
      MessageTransport::MessagePack(mp_serialize("1234567890")?)
    );

    let data = hashmap! {
        "input" => "1234",
    };
    let mut result = network.request("wapc_component", &data).await?;

    let output: MessageTransport = result.remove("output").unwrap();
    equals!(
      output,
      MessageTransport::Exception("Needs to be longer than 8 characters".to_owned())
    );

    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn short_circuit() -> TestResult<()> {
    let (network, _) = init_network_from_yaml("./manifests/short-circuit.yaml").await?;

    trace!("requesting schematic execution");
    let data = hashmap! {
        "input_port1" => "short",
    };

    let mut result = network.request("short_circuit", &data).await?;

    println!("result: {:?}", result);
    let output1: MessageTransport = result.remove("output1").unwrap();
    println!("Output: {:?}", output1);
    equals!(
      output1,
      MessageTransport::Exception("Needs to be longer than 8 characters".to_owned())
    );
    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn multiple_schematics() -> TestResult<()> {
    let (network, _) = init_network_from_yaml("./manifests/multiple-schematics.yaml").await?;

    let data = hashmap! {
        "left" => 42,
        "right" => 302309,
    };

    let mut result = network.request("first_schematic", &data).await?;

    trace!("result: {:?}", result);
    let output: MessageTransport = result.remove("output").unwrap();
    equals!(
      output,
      MessageTransport::MessagePack(mp_serialize(42 + 302309)?)
    );

    let data = hashmap! {
        "input" => "some string",
    };

    let mut result = network.request("second_schematic", &data).await?;

    println!("Result: {:?}", result);
    let output: MessageTransport = result.remove("output").unwrap();
    println!("Output: {:?}", output);
    equals!(
      output,
      MessageTransport::MessagePack(mp_serialize("some string")?)
    );
    Ok(())
  }
}
