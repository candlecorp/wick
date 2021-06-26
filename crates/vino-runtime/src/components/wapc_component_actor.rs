use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};
use std::time::Instant;

use actix::prelude::*;
use futures::StreamExt;
use log::info;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex as AsyncMutex;
use vino_codec::messagepack::serialize;
use vino_component::Packet;
use vino_rpc::port::{
  Port,
  Receiver,
  Sender,
};
use wapc::WapcHost;
use wascap::prelude::{
  Claims,
  KeyPair,
};

use crate::actix::ActorResult;
use crate::components::vino_component::WapcComponent;
use crate::dispatch::{
  Invocation,
  InvocationResponse,
};
use crate::schematic::NativeOutputReady;
use crate::Result;

#[derive(Default)]
pub(crate) struct WapcComponentActor {
  state: Option<State>,
  invocation_map: Arc<AsyncMutex<InvocationMap>>,
}

struct State {
  guest_module: Arc<AsyncMutex<WapcHost>>,
  claims: Claims<wascap::jwt::Actor>,
}

impl Actor for WapcComponentActor {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    trace!("Native actor started");
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {}
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

impl WapcComponentActor {}

#[derive(Default)]
struct InvocationMap {
  outputs: Vec<String>,
  map: HashMap<String, HashMap<String, OutputSender>>,
}

impl InvocationMap {
  fn new(outputs: Vec<String>) -> Self {
    Self {
      outputs,
      ..InvocationMap::default()
    }
  }
  fn new_invocation(&mut self, inv_id: String) -> Receiver {
    let (tx, rx) = self.make_channel();
    self.map.insert(inv_id, tx);
    rx
  }
  fn make_channel(&self) -> (HashMap<String, OutputSender>, Receiver) {
    let outputs: HashMap<String, OutputSender> = self
      .outputs
      .iter()
      .map(|name| (name.clone(), OutputSender::new(name.clone())))
      .collect();
    let ports = outputs.iter().map(|(_, o)| o.port.clone()).collect();
    let receiver = Receiver::new(ports);
    (outputs, receiver)
  }
}

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) bytes: Vec<u8>,
  pub(crate) outputs: Vec<String>,
  pub(crate) signing_seed: String,
}

impl Handler<Initialize> for WapcComponentActor {
  type Result = Result<()>;

  fn handle(&mut self, msg: Initialize, ctx: &mut Self::Context) -> Self::Result {
    trace!("Initializing component");
    self.invocation_map = Arc::new(AsyncMutex::new(InvocationMap::new(msg.outputs.clone())));
    let actor = perform_initialization(self, ctx, msg);
    match actor {
      Ok(_a) => Ok(()),
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
  me: &mut WapcComponentActor,
  ctx: &mut Context<WapcComponentActor>,
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

    match tx.send(WapcHostCallback {
      port: port.to_string(),
      invocation_id: inv_id.to_string(),
      payload: payload.to_vec(),
      op: op.to_string(),
      id,
      kp: KeyPair::from_seed(&seed).unwrap(),
    }) {
      Ok(_) => {
        trace!("Successfully sent output from Wapc host");
      }
      Err(e) => error!("Error sending output from WaPC host: {}", e),
    };
    Ok(vec![])
  });
  let invocation_map = me.invocation_map.clone();
  actix::spawn(async move {
    while let Some(callback_data) = rx.recv().await {
      let invocation_map = invocation_map.lock().await;
      let senders = invocation_map.map.get(&callback_data.invocation_id);
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

impl Handler<Invocation> for WapcComponentActor {
  type Result = ActorResult<Self, InvocationResponse>;

  fn handle(&mut self, msg: Invocation, _ctx: &mut Self::Context) -> Self::Result {
    let tx_id = msg.tx_id.clone();
    let target = msg.target.url();
    let component = actix_bail!(msg
      .target
      .into_component()
      .map_err(|_e| InvocationResponse::error(tx_id.clone(), "Sent invalid entity".to_string())));
    let message = actix_bail!(msg
      .msg
      .into_multibytes()
      .map_err(|_e| InvocationResponse::error(tx_id.clone(), "Sent invalid payload".to_string())));
    let name = component.name;
    let inv_id = msg.id;

    let state = self.state.as_ref().unwrap();
    let invocation_map = self.invocation_map.clone();
    let guest_module = state.guest_module.clone();
    let payload =
      actix_bail!(
        serialize((inv_id.clone(), message)).map_err(|e| InvocationResponse::error(
          tx_id.clone(),
          format!("Could not serialize payload for WaPC component: {}", e)
        ))
      );

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
          match tx.send(NativeOutputReady {
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

// impl Handler<Invocation> for WapcComponentActor {
//   type Result = InvocationResponse;

//   fn handle(&mut self, msg: Invocation, _ctx: &mut Self::Context) -> Self::Result {
//     let state = self.state.as_ref().unwrap();
//     let inv_id = msg.id.to_string();

//     debug!(
//       "Actor Invocation - From {} to {}",
//       msg.origin.url(),
//       msg.target.url(),
//     );

//     if let VinoEntity::Component(_) = msg.target {
//       if let MessageTransport::MultiBytes(payload) = &msg.msg {
//         let now = Instant::now();
//         match serialize((inv_id, payload)) {
//           Ok(bytes) => {
//             trace!("Serialized job input in {} μs", now.elapsed().as_micros());
//             let now = Instant::now();
//             match state.guest_module.call("job", &bytes) {
//               Ok(bytes) => {
//                 trace!("Actor call took {} μs", now.elapsed().as_micros());
//                 InvocationResponse::success(msg.tx_id, bytes)
//               }
//               Err(e) => {
//                 error!("Error invoking actor: {} (from {})", e, msg.target.url());
//                 debug!("Message: {:?}", &msg.msg);
//                 InvocationResponse::error(msg.tx_id, e.to_string())
//               }
//             }
//           }
//           Err(e) => {
//             InvocationResponse::error(msg.tx_id, format!("Error serializing payload: {}", e))
//           }
//         }
//       } else {
//         InvocationResponse::error(
//           msg.tx_id,
//           "Invalid payload sent from wapc actor".to_string(),
//         )
//       }
//     } else {
//       InvocationResponse::error(
//         msg.tx_id,
//         "Invalid entity invoked from wapc actor".to_string(),
//       )
//     }
//   }
// }
