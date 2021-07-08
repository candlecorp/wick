pub(crate) mod default;
pub(crate) mod handlers;

use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use handlers::input_message::InputMessage;
use handlers::transaction_update::TransactionUpdate;
use tokio::sync::mpsc::{
  UnboundedReceiver,
  UnboundedSender,
};

use crate::dev::prelude::*;
use crate::error::SchematicError;
use crate::transaction::TransactionMap;
use crate::validator::Validator;

type Result<T> = std::result::Result<T, SchematicError>;

#[derive(Debug)]
pub(crate) struct SchematicService {
  recipients: HashMap<String, ProviderChannel>,
  state: Option<State>,
  tx_external: HashMap<String, UnboundedSender<OutputPacket>>,
  tx_internal: HashMap<String, UnboundedSender<InputMessage>>,
}

#[derive(Debug)]
struct State {
  model: Arc<Mutex<SchematicModel>>,
  kp: KeyPair,
  name: String,
  transaction_map: TransactionMap,
}

impl Supervised for SchematicService {}

impl Default for SchematicService {
  fn default() -> Self {
    SchematicService {
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
    trace!("Schematic started");
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {}
}

impl SchematicService {
  pub(crate) fn get_name(&self) -> String {
    self.get_state().name.clone()
  }

  fn validate_model(&mut self) -> Result<()> {
    let mut model = self.get_state_mut().model.lock()?;
    Validator::validate_late_errors(&model)?;
    Ok(model.partial_initialization()?)
  }

  fn validate_final(&mut self) -> Result<()> {
    let mut model = self.get_state_mut().model.lock()?;
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

  fn new_transaction(
    &mut self,
    tx_id: String,
    rx: UnboundedReceiver<InputMessage>,
  ) -> UnboundedReceiver<TransactionUpdate> {
    let state = self.get_state_mut();
    state.transaction_map.new_transaction(tx_id, rx)
  }

  fn get_component_model(&self, reference: &str) -> Result<ComponentModel> {
    let state = self.get_state();
    let lock = state.model.lock().unwrap();
    let def = lock
      .get_component_definition(reference)
      .ok_or(SchematicError::InvalidModel(1))?;
    let model = lock.get_component_model(&def.id);
    Ok(model.ok_or(SchematicModelError::MissingComponentModel(def.id))?)
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
    let channel = self.recipients.get(&component.namespace)?;
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

  fn update_network_provider(&mut self, model: ProviderModel) {
    let state = self.get_state_mut();
    let mut lock = state.model.lock().unwrap();
    lock.commit_self_provider(model);
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

  use super::*;
  use crate::schematic_service::handlers::initialize::Initialize;
  use crate::test::prelude::*;

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
    let mut input: HashMap<String, Vec<u8>> = HashMap::new();
    let user_data = "this is test input";
    input.insert("input".to_owned(), mp_serialize(user_data)?);
    let payload = MessageTransport::MultiBytes(input);
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
    let mut input: HashMap<String, Vec<u8>> = HashMap::new();
    let user_data = "Hello world";
    input.insert("input".to_owned(), mp_serialize(user_data)?);
    let payload = MessageTransport::MultiBytes(input);
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
          equals!(payload, format!("TEST: {}", user_data));
        }
        debug!("Number of packets received: {}", i);
        equals!(i, 1);
      }
      InvocationResponse::Error { msg, .. } => panic!("{}", msg),
    };
    Ok(())
  }
}
