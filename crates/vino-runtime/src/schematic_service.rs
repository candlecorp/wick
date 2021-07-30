pub(crate) mod default;
pub(crate) mod handlers;

use std::collections::HashMap;
use std::sync::Arc;

use handlers::transaction_update::TransactionUpdate;
use parking_lot::Mutex;
use tokio::sync::mpsc::{
  UnboundedReceiver,
  UnboundedSender,
};
pub(crate) mod input_message;

use self::input_message::InputMessage;
use crate::dev::prelude::*;
use crate::error::SchematicError;
use crate::models::validator::Validator;
use crate::transaction::TransactionExecutor;

type Result<T> = std::result::Result<T, SchematicError>;

#[derive(Debug)]
pub(crate) struct SchematicService {
  name: String,
  recipients: HashMap<String, ProviderChannel>,
  state: Option<State>,
  tx_external: HashMap<String, UnboundedSender<InvocationTransport>>,
  tx_internal: HashMap<String, UnboundedSender<TransactionUpdate>>,
}

#[derive(Debug)]
struct State {
  model: Arc<Mutex<SchematicModel>>,
  kp: KeyPair,
  transactions: TransactionExecutor,
}

impl Supervised for SchematicService {}

impl Default for SchematicService {
  fn default() -> Self {
    SchematicService {
      name: "".to_owned(),
      recipients: HashMap::new(),
      state: None,
      tx_external: HashMap::new(),
      tx_internal: HashMap::new(),
    }
  }
}

impl Actor for SchematicService {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    trace!("SC:Service starting");
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {}
}

impl SchematicService {
  fn validate_model(&mut self) -> Result<()> {
    let mut model = self.get_state_mut().model.lock();
    Validator::validate_late_errors(&model)?;
    Ok(model.partial_initialization()?)
  }

  fn validate_final(&mut self) -> Result<()> {
    let mut model = self.get_state_mut().model.lock();
    Validator::validate_final_errors(&model)?;
    Ok(model.final_initialization()?)
  }

  fn get_state(&self) -> &State {
    if self.state.is_none() {
      panic!("Internal Error: schematic uninitialized");
    }
    let state = self.state.as_ref().unwrap();
    state
  }

  fn get_state_mut(&mut self) -> &mut State {
    if self.state.is_none() {
      panic!("Internal Error: schematic uninitialized");
    }
    let state = self.state.as_mut().unwrap();
    state
  }

  fn start(
    &mut self,
    tx_id: String,
  ) -> (
    UnboundedReceiver<TransactionUpdate>,
    UnboundedSender<TransactionUpdate>,
  ) {
    let state = self.get_state_mut();
    state.transactions.new_transaction(tx_id)
  }

  fn get_component_definition(&self, instance: &str) -> Result<ComponentDefinition> {
    let state = self.get_state();
    let lock = state.model.lock();
    lock
      .get_component_definition(instance)
      .ok_or_else(|| SchematicError::InstanceNotFound(instance.to_owned()))
  }

  fn get_recipient(&self, instance: &str) -> Result<Recipient<Invocation>> {
    let component = self.get_component_definition(instance)?;
    let state = self.get_state();
    let lock = state.model.lock();
    let err = SchematicError::InstanceNotFound(instance.to_owned());
    if !lock.has_component(&component.id) {
      warn!("SC:{}:{} does not have a valid model.", self.name, instance);
      return Err(err);
    }
    drop(lock);
    trace!("SC:{}:{} points to {}", self.name, instance, component.id);
    let channel = self.recipients.get(&component.namespace).ok_or(err)?;
    Ok(channel.recipient.clone())
  }

  fn get_outputs(&self, instance: &str) -> Vec<ConnectionTargetDefinition> {
    let state = self.get_state();
    let lock = state.model.lock();
    lock.get_outputs(instance)
  }

  fn get_port_connections(&self, port: &ConnectionTargetDefinition) -> Vec<ConnectionDefinition> {
    let state = self.get_state();
    let lock = state.model.lock();
    lock.get_port_connections(port).cloned().collect()
  }

  fn get_downstream_connections(&self, instance: &str) -> Vec<ConnectionDefinition> {
    let state = self.get_state();
    let lock = state.model.lock();
    lock.get_downstream_connections(instance).cloned().collect()
  }

  fn update_network_provider(&mut self, model: ProviderModel) -> Result<()> {
    let state = self.get_state_mut();
    let mut lock = state.model.lock();
    lock.commit_self_provider(model);
    lock.final_initialization()?;
    Ok(())
  }

  fn update_transaction(&self, msg: InputMessage) -> Result<()> {
    let inbound_tx = self
      .tx_internal
      .get(&msg.tx_id)
      .ok_or(InternalError(6003))?;
    Ok(inbound_tx.send(TransactionUpdate::Update(msg.handle_default()))?)
  }
}

#[derive(Debug, Clone)]
pub(crate) struct ProviderInitResponse {
  pub(crate) model: ProviderModel,
  pub(crate) channel: ProviderChannel,
}

#[cfg(test)]
mod test {
  use std::env;

  use vino_rpc::{
    bind_new_socket,
    make_rpc_server,
  };
  use vino_transport::message_transport::TransportMap;

  use super::*;
  use crate::schematic_service::handlers::initialize::Initialize;
  use crate::test::prelude::{
    assert_eq,
    *,
  };

  #[test_env_log::test(actix_rt::test)]
  async fn test_basic_schematic() -> TestResult<()> {
    let kp = KeyPair::new_server();
    let schematic = SchematicService::default();
    let addr = schematic.start();
    let schematic_name = "logger";

    let def = load_schematic_manifest("./src/schematic_service/test-schematics/basic.yaml")?;

    addr
      .send(Initialize {
        schematic: def,
        network_provider_channel: None,
        seed: kp.seed().unwrap(),
        allow_latest: true,
        allowed_insecure: vec![],
        global_providers: vec![],
      })
      .await??;
    let mut input: HashMap<String, MessageTransport> = HashMap::new();
    let user_data = "this is test input";
    input.insert(
      "input".to_owned(),
      MessageTransport::MessagePack(mp_serialize(user_data)?),
    );

    let payload = TransportMap::with_map(input);
    let response: InvocationResponse = addr
      .send(Invocation::new(
        &kp,
        Entity::test("basic"),
        Entity::schematic(schematic_name),
        payload,
      ))
      .await?;
    match response {
      InvocationResponse::Stream { mut rx, .. } => {
        debug!("Got stream");
        let mut i = 0;

        while let Some(next) = rx.next().await {
          i += 1;
          let packet = next.payload;
          debug!("Packet {}: {:?}", i, packet);
          let payload: String = packet.try_into()?;
          debug!("Payload {}", payload);
          assert_eq!(payload, user_data);
        }
        debug!("Number of packets received: {}", i);
        assert_eq!(i, 1);
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
    env::set_var("TEST_PORT", port.to_string());
    let def =
      load_schematic_manifest("./src/schematic_service/test-schematics/grpc-provider.yaml")?;

    let schematic = SchematicService::default();
    let addr = schematic.start();
    let schematic_name = "grpc";

    let kp = KeyPair::new_server();

    addr
      .send(Initialize {
        schematic: def,
        seed: kp.seed().unwrap(),
        network_provider_channel: None,
        allow_latest: true,
        global_providers: vec![],
        allowed_insecure: vec![],
      })
      .await??;

    let mut input = TransportMap::new();
    let user_data = "Hello world";
    input.insert("input", MessageTransport::messagepack(user_data));
    let response: InvocationResponse = addr
      .send(Invocation::new(
        &kp,
        Entity::test("basic"),
        Entity::schematic(schematic_name),
        input,
      ))
      .await?;

    match response {
      InvocationResponse::Stream { mut rx, .. } => {
        debug!("Got stream");
        let mut i = 0;

        while let Some(next) = rx.next().await {
          i += 1;
          let packet = next.payload;
          debug!("Packet {}: {:?}", i, packet);
          let payload: String = packet.try_into()?;
          debug!("Payload {}", payload);
          assert_eq!(payload, format!("TEST: {}", user_data));
        }
        debug!("Number of packets received: {}", i);
        assert_eq!(i, 1);
      }
      InvocationResponse::Error { msg, .. } => panic!("{}", msg),
    };
    Ok(())
  }
}
