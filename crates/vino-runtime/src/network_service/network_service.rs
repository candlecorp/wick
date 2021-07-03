use std::collections::HashMap;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use vino_rpc::SchematicSignature;

use super::messages::*;
use crate::dev::prelude::*;
use crate::dispatch::inv_error;
use crate::models::provider_model::{
  create_network_provider_model,
  start_network_provider,
};
use crate::schematic_service::messages::{
  GetSignature,
  UpdateProvider,
};

type Result<T> = std::result::Result<T, NetworkError>;
#[derive(Clone, Debug)]

pub(crate) struct NetworkService {
  started: bool,
  started_time: std::time::Instant,
  id: String,
  schematics: HashMap<String, Addr<SchematicService>>,
  definition: NetworkDefinition,
}

impl Default for NetworkService {
  fn default() -> Self {
    NetworkService {
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
  pub(crate) fn get_schematic(&self, id: &str) -> Result<Addr<SchematicService>> {
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

impl Handler<Initialize> for NetworkService {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: Initialize, _ctx: &mut Context<Self>) -> Self::Result {
    trace!("Network {} initializing", msg.network_id);
    self.started = true;

    self.id = msg.network_id.clone();
    let network_id = msg.network_id;
    self.definition = msg.network;
    let allowed_insecure = msg.allowed_insecure;

    let seed = msg.seed;
    let allow_latest = msg.allow_latest;
    let schematics = self.definition.schematics.clone();
    self.schematics = start_schematic_services(&schematics);
    let address_map = self.schematics.clone();

    let task = async move {
      let provider_addr = start_network_provider(SELF_NAMESPACE, network_id).await?;
      let channel = ProviderChannel {
        namespace: SELF_NAMESPACE.to_owned(),
        recipient: provider_addr.clone().recipient(),
      };
      let init_msgs = schematics.into_iter().map(|def| {
        let addr = address_map.get(&def.name).unwrap();
        addr.send(crate::schematic_service::messages::Initialize {
          seed: seed.clone(),
          network_provider_channel: Some(channel.clone()),
          schematic: def,
          allow_latest,
          allowed_insecure: allowed_insecure.clone(),
        })
      });

      let results = join_or_err(init_msgs, 5001).await?;

      let errors: Vec<SchematicError> = filter_map(results, |e| e.err());
      if errors.is_empty() {
        debug!("Schematics initialized");
        Ok(provider_addr.clone())
      } else {
        Err(NetworkError::InitializationError(join(errors, ", ")))
      }
    }
    .into_actor(self)
    .then(|result, network, _ctx| {
      let schematics = network.schematics.clone();
      async move {
        let addr = result?;
        for _ in 1..5 {
          let result = create_network_provider_model("self", addr.clone()).await;
          if result.is_err() {
            continue;
          }
          let model = result.unwrap();
          let result = join_or_err(
            schematics.values().map(|addr| {
              addr.send(UpdateProvider {
                model: model.clone(),
              })
            }),
            5020,
          )
          .await;
          if result.is_ok() {
            return Ok(());
          }
        }
        Err(NetworkError::MaxTriesReached)
      }
      .into_actor(network)
    });

    ActorResult::reply_async(task)
  }
}

fn start_schematic_services(
  schematics: &[SchematicDefinition],
) -> HashMap<String, Addr<SchematicService>> {
  trace!("Starting schematic arbiters");
  map(schematics, |def| {
    let arbiter = Arbiter::new();
    let addr =
      SchematicService::start_in_arbiter(&arbiter.handle(), |_| SchematicService::default());
    (def.name.clone(), addr)
  })
}

impl Handler<Request> for NetworkService {
  type Result = ActorResult<Self, Result<HashMap<String, MessageTransport>>>;

  fn handle(&mut self, msg: Request, _ctx: &mut Context<Self>) -> Self::Result {
    actix_try!(self.ensure_is_started());
    let schematic_name = msg.schematic;
    trace!("Requesting schematic '{}'", schematic_name);

    let schematic = actix_try!(self.get_schematic(&schematic_name));

    let request = crate::schematic_service::messages::Request {
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
      let results: Vec<SchematicResult<SchematicSignature>> = join_or_err(requests, 5004).await?;
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

impl Handler<Invocation> for NetworkService {
  type Result = ActorResult<Self, InvocationResponse>;

  fn handle(&mut self, msg: Invocation, _ctx: &mut Context<Self>) -> Self::Result {
    let tx_id = msg.tx_id;
    actix_ensure_ok!(self
      .ensure_is_started()
      .map_err(|e| inv_error(&tx_id, &e.to_string())));
    let schematic_name = match msg.target {
      Entity::Schematic(name) => name,
      Entity::Component(c) => c.name,
      _ => return ActorResult::reply(inv_error(&tx_id, "Sent invalid entity")),
    };

    trace!("Invoking schematic '{}'", schematic_name);

    let schematic = actix_ensure_ok!(self
      .get_schematic(&schematic_name)
      .map_err(|e| inv_error(&tx_id, &e.to_string())));
    let payload = actix_ensure_ok!(msg
      .msg
      .into_multibytes()
      .map_err(|e| inv_error(&tx_id, &e.to_string())));

    let request = crate::schematic_service::messages::Request {
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
      MessageTransport::MessagePack(mp_serialize(user_data)?)
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
