use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use actix::prelude::*;
use futures::StreamExt;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex as AsyncMutex;
use vino_codec::messagepack::serialize;
use vino_component::Packet;
use vino_rpc::port::Sender;
use vino_rpc::HostedType;
use wapc::WapcHost;
use wascap::prelude::{
  Claims,
  KeyPair,
};

use super::{
  ProviderRequest,
  ProviderResponse,
};
use crate::actix::ActorResult;
use crate::component_model::ComponentModel;
use crate::components::vino_component::WapcComponent;
use crate::dispatch::{
  Invocation,
  InvocationResponse,
};
use crate::invocation_map::InvocationMap;
use crate::schematic::ComponentOutput;
use crate::Result;

#[derive(Default)]
pub(crate) struct WapcProvider {
  state: Option<State>,
  invocation_map: Arc<AsyncMutex<InvocationMap>>,
}

struct State {
  guest_module: Arc<AsyncMutex<WapcHost>>,
  claims: Claims<wascap::jwt::Actor>,
  name: String,
  inputs: Vec<String>,
  outputs: Vec<String>,
}

impl Actor for WapcProvider {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    trace!("Native actor started");
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {}
}

impl WapcProvider {}

#[derive(Message)]
#[rtype(result = "Result<HashMap<String, ComponentModel>>")]
pub(crate) struct Initialize {
  pub(crate) namespace: String,
  pub(crate) bytes: Vec<u8>,
  pub(crate) name: String,
  pub(crate) outputs: Vec<String>,
  pub(crate) inputs: Vec<String>,
  pub(crate) signing_seed: String,
}

impl Handler<Initialize> for WapcProvider {
  type Result = Result<HashMap<String, ComponentModel>>;

  fn handle(&mut self, msg: Initialize, ctx: &mut Self::Context) -> Self::Result {
    trace!("Initializing component");
    self.invocation_map = Arc::new(AsyncMutex::new(InvocationMap::new(msg.outputs.clone())));
    let name = msg.name.clone();
    let component = ComponentModel {
      id: format!("{}::{}", msg.namespace, msg.name),
      name: name.clone(),
      inputs: msg.inputs.clone(),
      outputs: msg.outputs.clone(),
    };
    let actor = perform_initialization(self, ctx, msg);
    let mut map = HashMap::new();
    map.insert(name, component);
    match actor {
      Ok(_a) => Ok(map),
      Err(e) => Err(e),
    }
  }
}

struct WapcHostCallback {
  #[allow(dead_code)]
  id: u64,
  #[allow(dead_code)]
  kp: KeyPair,
  #[allow(dead_code)]
  op: String,
  invocation_id: String,
  payload: Vec<u8>,
  port: String,
}

fn perform_initialization(
  me: &mut WapcProvider,
  ctx: &mut Context<WapcProvider>,
  msg: Initialize,
) -> Result<String> {
  let buf = msg.bytes;
  let actor = WapcComponent::from_slice(&buf)?;
  let claims = actor.token.claims.clone();
  let jwt = actor.token.jwt.to_string();

  // Ensure that the JWT we found on this actor is valid, not expired, can be used,
  // has a verified signature, etc.
  let _tv = wascap::jwt::validate_token::<wascap::jwt::Actor>(&jwt)?;

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

  let seed = msg.signing_seed;

  let (tx, mut rx) = unbounded_channel();
  let guest = WapcHost::new(Box::new(engine), move |id, inv_id, port, op, payload| {
    debug!("Payload WaPC host callback: {:?}", payload);
    meh!(tx.send(WapcHostCallback {
      port: port.to_string(),
      invocation_id: inv_id.to_string(),
      payload: payload.to_vec(),
      op: op.to_string(),
      id,
      kp: KeyPair::from_seed(&seed).unwrap(),
    }));
    Ok(vec![])
  });
  let invocation_map = me.invocation_map.clone();
  actix::spawn(async move {
    while let Some(callback_data) = rx.recv().await {
      let invocation_map = invocation_map.lock().await;
      let senders = invocation_map.get(&callback_data.invocation_id);
      if senders.is_none() {
        error!(
          "Could not invocation map for {}",
          callback_data.invocation_id
        );
        continue;
      }
      let senders = senders.unwrap();
      let port = senders.get(&callback_data.port);
      if port.is_none() {
        error!(
          "Could not get port sender for {} on transaction {}",
          callback_data.port, callback_data.invocation_id
        );
        continue;
      }
      let port = port.unwrap();
      let payload = &callback_data.payload;
      let packet: Packet = payload.into();
      port.send_message(packet);
    }
  });

  match guest {
    Ok(g) => {
      me.state = Some(State {
        name: msg.name,
        inputs: msg.inputs,
        outputs: msg.outputs,
        claims: claims.clone(),
        guest_module: Arc::new(AsyncMutex::new(g)),
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
      Err("Failed to create a raw WebAssembly host".into())
    }
  }
}

impl Handler<ProviderRequest> for WapcProvider {
  type Result = ActorResult<Self, Result<ProviderResponse>>;

  fn handle(&mut self, msg: ProviderRequest, _ctx: &mut Self::Context) -> Self::Result {
    let state = self.state.as_ref().unwrap();
    //Temporary until WaPC modules host more than one component
    let component = vino_rpc::Component {
      name: state.name.clone(),
      inputs: state
        .inputs
        .iter()
        .map(|name| vino_rpc::Port {
          name: name.clone(),
          type_string: "TODO".to_string(),
        })
        .collect(),
      outputs: state
        .outputs
        .clone()
        .iter()
        .map(|name| vino_rpc::Port {
          name: name.clone(),
          type_string: "TODO".to_string(),
        })
        .collect(),
    };

    let task = async move {
      returns!(ProviderResponse);
      match msg {
        ProviderRequest::Invoke(_invocation) => todo!(),
        ProviderRequest::List(_req) => Ok(ProviderResponse::List(vec![HostedType::Component(
          component,
        )])),
        ProviderRequest::Statistics(_req) => Ok(ProviderResponse::Stats(vec![])),
      }
    };
    ActorResult::reply_async(task.into_actor(self))
  }
}

impl Handler<Invocation> for WapcProvider {
  type Result = ActorResult<Self, InvocationResponse>;

  fn handle(&mut self, msg: Invocation, _ctx: &mut Self::Context) -> Self::Result {
    let tx_id = msg.tx_id.clone();
    let target = msg.target.url();
    let component = actix_ensure_ok!(msg
      .target
      .into_component()
      .map_err(|_e| InvocationResponse::error(tx_id.clone(), "Sent invalid entity".to_string())));
    let message = actix_ensure_ok!(msg
      .msg
      .into_multibytes()
      .map_err(|_e| InvocationResponse::error(tx_id.clone(), "Sent invalid payload".to_string())));
    let name = component.name;
    let inv_id = msg.id;

    let state = self.state.as_ref().unwrap();
    let invocation_map = self.invocation_map.clone();
    let guest_module = state.guest_module.clone();
    let payload =
      actix_ensure_ok!(serialize((inv_id.clone(), message)).map_err(
        |e| InvocationResponse::error(
          tx_id.clone(),
          format!("Could not serialize payload for WaPC component: {}", e)
        )
      ));

    let request = async move {
      let invocation_id = inv_id.clone();

      let now = Instant::now();
      let guest_module = guest_module.lock().await;
      let mut receiver = {
        let mut invocation_map = invocation_map.lock().await;
        invocation_map.new_invocation(inv_id)
      };

      match guest_module.call("job", &payload) {
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
          trace!("Provider component {} waiting for output", name);
          let next = receiver.next().await;
          if next.is_none() {
            break;
          }

          let (port_name, msg) = next.unwrap();
          trace!("Native actor {} got output on port [{}]", name, port_name);
          match tx.send(ComponentOutput {
            port: port_name.to_string(),
            payload: msg,
            invocation_id: invocation_id.to_string(),
          }) {
            Ok(_) => {
              trace!("Sent output to port '{}' ", port_name);
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
