use std::time::Instant;

use crate::components::vino_component::WapcComponent;
use crate::dispatch::VinoEntity;

use crate::dispatch::{Invocation, InvocationResponse, MessagePayload};
use crate::serialize;
use crate::Result;
use actix::prelude::*;
use log::info;
use wapc::WapcHost;
use wascap::prelude::{Claims, KeyPair};

#[derive(Default)]
pub(crate) struct WapcComponentActor {
  state: Option<State>,
}

struct State {
  guest_module: WapcHost,
  claims: Claims<wascap::jwt::Actor>,
}

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) actor_bytes: Vec<u8>,
  pub(crate) signing_seed: String,
}

impl Handler<Initialize> for WapcComponentActor {
  type Result = Result<()>;

  fn handle(&mut self, msg: Initialize, ctx: &mut Self::Context) -> Self::Result {
    trace!("Initializing component");
    let actor = perform_initialization(self, ctx, msg);
    match actor {
      Ok(_a) => Ok(()),
      Err(e) => Err(e),
    }
  }
}

fn perform_initialization(
  me: &mut WapcComponentActor,
  ctx: &mut SyncContext<WapcComponentActor>,
  msg: Initialize,
) -> Result<String> {
  let buf = msg.actor_bytes.clone();
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

  let cloned_claims = claims.clone();
  let seed = msg.signing_seed;

  let guest = WapcHost::new(
    Box::new(engine),
    move |_id, binding, namespace, operation, payload| {
      crate::dispatch::wapc_host_callback(
        KeyPair::from_seed(&seed).unwrap(),
        cloned_claims.clone(),
        binding,
        namespace,
        operation,
        payload,
      )
    },
  );

  match guest {
    Ok(g) => {
      me.state = Some(State {
        guest_module: g,
        claims: claims.clone(),
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

impl Actor for WapcComponentActor {
  type Context = SyncContext<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    trace!("Component started");
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {
    // NOTE: do not attempt to log asynchronously in a stopped function,
    // resources (including stdout) may not be available
  }
}

impl Handler<Invocation> for WapcComponentActor {
  type Result = InvocationResponse;

  fn handle(&mut self, msg: Invocation, _ctx: &mut Self::Context) -> Self::Result {
    let state = self.state.as_ref().unwrap();
    let inv_id = msg.id.to_string();

    debug!(
      "Actor Invocation - From {} to {}: {}",
      msg.origin.url(),
      msg.target.url(),
      msg.operation
    );

    if let VinoEntity::Component(_) = msg.target {
      if let MessagePayload::MultiBytes(payload) = &msg.msg {
        let now = Instant::now();
        match serialize((inv_id, payload)) {
          Ok(bytes) => {
            trace!("Serialized job input in {} μs", now.elapsed().as_micros());
            let now = Instant::now();
            match state.guest_module.call(&msg.operation, &bytes) {
              Ok(bytes) => {
                trace!("Actor call took {} μs", now.elapsed().as_micros());
                InvocationResponse::success(msg.tx_id, bytes)
              }
              Err(e) => {
                error!("Error invoking actor: {} (from {})", e, msg.target.url());
                debug!("Message: {:?}", &msg.msg);
                InvocationResponse::error(msg.tx_id, e.to_string())
              }
            }
          }
          Err(e) => {
            InvocationResponse::error(msg.tx_id, format!("Error serializing payload: {}", e))
          }
        }
      } else {
        InvocationResponse::error(
          msg.tx_id,
          "Invalid payload sent from wapc actor".to_string(),
        )
      }
    } else {
      InvocationResponse::error(
        msg.tx_id,
        "Invalid entity invoked from wapc actor".to_string(),
      )
    }
  }
}
