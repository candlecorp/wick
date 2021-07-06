use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};
use std::time::Instant;

use tokio::sync::mpsc::unbounded_channel;
use vino_provider::ComponentSignature;
use vino_rpc::port::{
  Port,
  PortStream,
  Sender,
};
use vino_rpc::HostedType;
use vino_wascap::{
  Claims,
  ComponentClaims,
  KeyPair,
};
use wapc::WapcHost;

use super::{
  ProviderRequest,
  ProviderResponse,
};
use crate::dev::prelude::*;
use crate::error::ComponentError;
use crate::providers::wapc_module::WapcModule;

type Result<T> = std::result::Result<T, ComponentError>;

#[derive(Default)]
pub(crate) struct WapcProviderService {
  state: Option<State>,
  invocation_map: Arc<Mutex<InvocationMap>>,
}

struct State {
  guest_module: Arc<Mutex<WapcHost>>,
  claims: Claims<ComponentClaims>,
}

impl Actor for WapcProviderService {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    trace!("Native actor started");
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {}
}

impl WapcProviderService {
  pub(crate) fn get_components(&self) -> &Vec<ComponentSignature> {
    &self
      .state
      .as_ref()
      .unwrap()
      .claims
      .metadata
      .as_ref()
      .unwrap()
      .interface
      .components
  }
}

#[derive(Message)]
#[rtype(result = "Result<HashMap<String, ComponentModel>>")]
pub(crate) struct Initialize {
  pub(crate) namespace: String,
  pub(crate) bytes: Vec<u8>,
  pub(crate) claims: Claims<ComponentClaims>,
  pub(crate) signing_seed: String,
}

impl Handler<Initialize> for WapcProviderService {
  type Result = Result<HashMap<String, ComponentModel>>;

  fn handle(&mut self, msg: Initialize, ctx: &mut Self::Context) -> Self::Result {
    trace!("Initializing component");
    let namespace = msg.namespace.clone();

    let actor = perform_initialization(self, ctx, msg);

    let components: HashMap<String, ComponentModel> = self
      .get_components()
      .iter()
      .cloned()
      .map(|c| {
        (
          c.name.clone(),
          ComponentModel {
            namespace: namespace.clone(),
            name: c.name,
            inputs: c.inputs,
            outputs: c.outputs,
          },
        )
      })
      .collect();

    match actor {
      Ok(_a) => Ok(components),
      Err(e) => Err(e),
    }
  }
}

fn perform_initialization(
  me: &mut WapcProviderService,
  ctx: &mut Context<WapcProviderService>,
  msg: Initialize,
) -> Result<String> {
  let buf = msg.bytes;
  let actor = WapcModule::from_slice(&buf)?;
  let claims = actor.token.claims.clone();
  let jwt = actor.token.jwt.clone();

  // Ensure that the JWT we found on this actor is valid, not expired, can be used,
  // has a verified signature, etc.
  let _tv = vino_wascap::validate_token::<ComponentClaims>(&jwt)
    .map_err(|e| ComponentError::ClaimsError(e.to_string()));

  let time = Instant::now();
  #[cfg(feature = "wasmtime")]
  let engine = {
    let engine = wasmtime_provider::WasmtimeEngineProvider::new(&buf, None);
    trace!("Wasmtime loaded in {} μs", time.elapsed().as_micros());
    engine
  };
  #[cfg(feature = "wasm3")]
  let engine = {
    let engine = wasm3_provider::Wasm3EngineProvider::new(&buf);
    trace!("wasm3 loaded in {} μs", time.elapsed().as_micros());
    engine
  };

  let invocation_map = Arc::new(Mutex::new(InvocationMap::new(
    claims
      .metadata
      .as_ref()
      .unwrap()
      .interface
      .components
      .clone(),
  )));

  let seed = msg.signing_seed;

  let map = invocation_map.clone();

  let guest = WapcHost::new(Box::new(engine), move |id, inv_id, port, op, payload| {
    let _kp = keypair_from_seed(&seed).unwrap();
    debug!("Payload WaPC host callback: {:?}", payload);

    let invocation_map = invocation_map.lock().unwrap();
    let senders = invocation_map.get(inv_id);
    if senders.is_none() {
      error!("Could not find invocation map for {}", inv_id);
      return Ok(vec![]);
    }
    let sender = senders.unwrap().get(port);
    if sender.is_none() {
      error!(
        "Could not get port sender for {} on transaction {}",
        port, inv_id
      );
      return Ok(vec![]);
    }
    let port = sender.unwrap();
    let packet: Packet = payload.into();
    port.send_message(packet);
    Ok(vec![])
  });

  match guest {
    Ok(g) => {
      me.invocation_map = map;
      me.state = Some(State {
        claims: claims.clone(),
        guest_module: Arc::new(Mutex::new(g)),
      });
      info!(
        "Actor {} initialized",
        &me.state.as_ref().unwrap().claims.subject
      );
      Ok(claims.subject)
    }
    Err(_e) => {
      error!(
        "Failed to create a WebAssembly host for actor {}",
        actor.token.claims.subject
      );
      ctx.stop();
      Err(ComponentError::WapcError)
    }
  }
}

impl Handler<ProviderRequest> for WapcProviderService {
  type Result = ActorResult<Self, Result<ProviderResponse>>;

  fn handle(&mut self, msg: ProviderRequest, _ctx: &mut Self::Context) -> Self::Result {
    let state = self.state.as_ref().unwrap();
    let components = state
      .claims
      .metadata
      .as_ref()
      .unwrap()
      .interface
      .components
      .iter()
      .map(|c| HostedType::Component(c.clone()))
      .collect();

    let task = async move {
      match msg {
        ProviderRequest::Invoke(_invocation) => todo!(),
        ProviderRequest::List(_req) => Ok(ProviderResponse::List(components)),
        ProviderRequest::Statistics(_req) => Ok(ProviderResponse::Stats(vec![])),
      }
    };
    ActorResult::reply_async(task.into_actor(self))
  }
}

impl Handler<Invocation> for WapcProviderService {
  type Result = ActorResult<Self, InvocationResponse>;

  fn handle(&mut self, msg: Invocation, _ctx: &mut Self::Context) -> Self::Result {
    let tx_id = msg.tx_id.clone();
    let target = msg.target.url();
    let component =
      actix_ensure_ok!(msg
        .target
        .into_component()
        .map_err(|_e| InvocationResponse::error(
          tx_id.clone(),
          "WaPC provider sent invalid entity".to_owned()
        )));
    let message =
      actix_ensure_ok!(msg
        .msg
        .into_multibytes()
        .map_err(|_e| InvocationResponse::error(
          tx_id.clone(),
          "WaPC provider sent invalid payload".to_owned()
        )));
    let inv_id = msg.id;

    let state = self.state.as_ref().unwrap();
    let invocation_map = self.invocation_map.clone();
    let guest_module = state.guest_module.clone();
    let payload = actix_ensure_ok!(mp_serialize((inv_id.clone(), message)).map_err(|e| {
      InvocationResponse::error(
        tx_id.clone(),
        format!("Could not serialize payload for WaPC component: {}", e),
      )
    }));

    let request = async move {
      let invocation_id = inv_id.clone();

      let now = Instant::now();
      let mut locked = invocation_map.lock().unwrap();
      let mut receiver = locked.new_invocation(inv_id, &component);
      drop(locked);

      let guest_module = guest_module.lock().unwrap();
      match guest_module.call(&component, &payload) {
        Ok(bytes) => {
          debug!("Actor call took {} μs", now.elapsed().as_micros());
          trace!("Actor responded with {:?}", bytes);
        }
        Err(e) => {
          error!("Error invoking actor: {} (from {})", e, target);
          debug!("Message: {:?}", &payload);
        }
      }

      let (tx, rx) = unbounded_channel();
      actix::spawn(async move {
        loop {
          trace!("Provider component {} waiting for output", component);
          let next = receiver.next().await;
          if next.is_none() {
            break;
          }

          let output = next.unwrap();
          trace!(
            "Native actor {} got output on port [{}]",
            component,
            output.port
          );
          match tx.send(OutputPacket {
            port: output.port.clone(),
            payload: output.packet,
            invocation_id: invocation_id.clone(),
          }) {
            Ok(_) => {
              trace!("Sent output to port '{}' ", output.port);
            }
            Err(e) => {
              error!("Error sending output on channel {}", e.to_string());
              break;
            }
          }
        }
      });
      InvocationResponse::stream(tx_id, rx)
    };
    ActorResult::reply_async(request.into_actor(self))
  }
}

pub(crate) struct OutputSender {
  port: Arc<Mutex<Port>>,
}
impl OutputSender {
  fn new(name: String) -> Self {
    Self {
      port: Arc::new(Mutex::new(Port::new(name))),
    }
  }
}

impl Sender for OutputSender {
  type PayloadType = Vec<u8>;
  fn get_port(&self) -> Arc<Mutex<Port>> {
    self.port.clone()
  }
}

#[derive(Default)]
pub(crate) struct InvocationMap {
  components: HashMap<String, ComponentSignature>,
  map: HashMap<String, HashMap<String, OutputSender>>,
}

impl InvocationMap {
  pub(crate) fn new(components: Vec<ComponentSignature>) -> Self {
    let components = components
      .into_iter()
      .map(|c| (c.name.clone(), c))
      .collect();
    Self {
      components,
      map: HashMap::new(),
    }
  }

  pub(crate) fn get(&self, inv_id: &str) -> Option<&HashMap<String, OutputSender>> {
    trace!("Getting transaction for {:?} in map {:p}", inv_id, self);
    self.map.get(inv_id)
  }

  pub(crate) fn new_invocation(&mut self, inv_id: String, component: &str) -> PortStream {
    trace!("Inserting {:?} in map {:p}", inv_id, self);
    let (tx, rx) = self.make_channel(component);
    self.map.insert(inv_id, tx);
    rx
  }

  pub(crate) fn make_channel(
    &self,
    component_name: &str,
  ) -> (HashMap<String, OutputSender>, PortStream) {
    let component = self.components.get(component_name).unwrap();
    let outputs: HashMap<String, OutputSender> = component
      .outputs
      .iter()
      .map(|port| (port.name.clone(), OutputSender::new(port.name.clone())))
      .collect();
    let ports = outputs.iter().map(|(_, o)| o.port.clone()).collect();
    let receiver = PortStream::new(ports);
    (outputs, receiver)
  }
}
