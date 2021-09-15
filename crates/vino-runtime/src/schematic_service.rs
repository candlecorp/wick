pub(crate) mod default;
pub(crate) mod handlers;

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use tokio::sync::mpsc::{
  UnboundedReceiver,
  UnboundedSender,
};
pub(crate) mod input_message;

use crate::dev::prelude::*;
use crate::error::SchematicError;
use crate::models::validator::Validator;
use crate::transaction::executor::TransactionExecutor;

type Result<T> = std::result::Result<T, SchematicError>;

#[derive(Debug)]
pub(crate) struct SchematicService {
  name: String,
  recipients: HashMap<String, ProviderChannel>,
  state: Option<State>,
  executor: HashMap<String, UnboundedSender<TransactionUpdate>>,
}

#[derive(Debug)]
struct State {
  model: Arc<RwLock<SchematicModel>>,
  transactions: TransactionExecutor,
}

impl Supervised for SchematicService {}

impl Default for SchematicService {
  fn default() -> Self {
    SchematicService {
      name: "".to_owned(),
      recipients: HashMap::new(),
      state: None,
      executor: HashMap::new(),
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
    let model = &mut self.get_model().write();
    Validator::validate_late_errors(model)?;
    Ok(model.partial_initialization()?)
  }

  fn validate_final(&mut self) -> Result<()> {
    let model = &mut self.get_model().write();
    Validator::validate_final_errors(model)?;
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

  fn get_model(&self) -> &SharedModel {
    &self.get_state().model
  }

  fn get_recipient(&self, instance: &str) -> Result<Recipient<Invocation>> {
    let component = get_component_definition(self.get_model(), instance)?;
    let model = self.get_model().read();
    let err = SchematicError::InstanceNotFound(instance.to_owned());
    if !model.has_component(&component.id) {
      warn!("SC:{}:{} does not have a valid model.", self.name, instance);
      return Err(err);
    }
    trace!(
      "SC:{}:INSTANCE[{}]->[{}]",
      self.name,
      instance,
      component.id
    );
    let channel = self.recipients.get(&component.namespace).ok_or(err)?;
    Ok(channel.recipient.clone())
  }

  fn update_network_provider(&mut self, provider_model: ProviderModel) -> Result<()> {
    let mut model = self.get_model().write();
    model.commit_self_provider(provider_model);
    model.final_initialization()?;
    Ok(())
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
  use std::time::Duration;

  use vino_invocation_server::{
    bind_new_socket,
    make_rpc_server,
  };
  use vino_transport::message_transport::TransportMap;

  use super::*;
  use crate::schematic_service::handlers::initialize::Initialize;
  use crate::test::prelude::{
    assert_eq,
    transport_map,
    *,
  };

  static TIMEOUT: u64 = 100;

  #[test_logger::test(actix_rt::test)]
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
        lattice: None,
        allowed_insecure: vec![],
        global_providers: vec![],
        timeout: Duration::from_millis(TIMEOUT),
      })
      .await??;
    let user_data = "this is test input";
    let payload = transport_map! {"input"=> user_data};

    let response: InvocationResponse = addr
      .send(Invocation::new(
        Entity::test("basic"),
        Entity::schematic(schematic_name),
        payload,
      ))
      .await?;
    let mut rx = response.ok()?;
    debug!("Got stream");
    let mut packets: Vec<_> = rx.collect_port("output").await;
    println!("Packets: {:?}", packets);
    assert_eq!(packets.len(), 1);

    let packet = packets.remove(0);
    debug!("Packet {:?}", packet);
    let payload: String = packet.try_into()?;
    debug!("Payload {}", payload);
    assert_eq!(payload, user_data);
    assert_eq!(rx.buffered_size(), (0, 0));

    Ok(())
  }

  #[test_logger::test(actix_rt::test)]
  async fn test_grpc_url_provider() -> TestResult<()> {
    let socket = bind_new_socket()?;
    let port = socket.local_addr()?.port();
    let _ = make_rpc_server(socket, Box::new(test_vino_provider::Provider::default()));
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
        lattice: None,
        global_providers: vec![],
        allowed_insecure: vec![],
        timeout: Duration::from_millis(TIMEOUT),
      })
      .await??;

    let mut input = TransportMap::new();
    let user_data = "Hello world";
    input.insert("input", MessageTransport::messagepack(user_data));
    let response: InvocationResponse = addr
      .send(Invocation::new(
        Entity::test("basic"),
        Entity::schematic(schematic_name),
        input,
      ))
      .await?;
    let mut result = response.ok()?;

    debug!("Got stream");
    let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
    debug!("Messages {:?}", messages);
    assert_eq!(messages.len(), 1);
    let payload: String = messages.remove(0).payload.try_into()?;
    debug!("Payload {}", payload);
    assert_eq!(payload, format!("TEST: {}", user_data));
    assert_eq!(result.buffered_size(), (0, 0));

    Ok(())
  }
}
