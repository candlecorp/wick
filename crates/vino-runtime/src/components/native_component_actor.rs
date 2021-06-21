use std::borrow::BorrowMut;
use std::sync::{
  Arc,
  Mutex,
};

use actix::prelude::*;
use futures::executor::block_on;
use futures::StreamExt;
use nkeys::KeyPair;
use vino_guest::{
  OutputPayload,
  Signal,
};
use vino_transport::serialize;

use crate::components::vino_component::NativeComponent;
use crate::dispatch::{
  native_host_callback,
  Invocation,
  InvocationResponse,
  MessagePayload,
  VinoEntity,
};
use crate::native_actors::State;
use crate::{
  native_actors,
  Result,
};

#[derive(Default)]
pub struct NativeComponentActor {
  name: String,
  seed: String,
  state: Option<Arc<Mutex<State>>>,
}

impl std::fmt::Debug for NativeComponentActor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("NativeComponentActor")
      .field("component", &Some("<removed>".to_string()))
      .field("name", &self.name)
      .field("seed", &self.seed)
      .finish()
  }
}

impl Actor for NativeComponentActor {
  type Context = SyncContext<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    trace!("Native actor started");
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {}
}

pub trait NativeActor {
  fn get_def(&self) -> NativeComponent;
  fn get_name(&self) -> String;
  fn get_input_ports(&self) -> Vec<String>;
  fn get_output_ports(&self) -> Vec<String>;
  fn job_wrapper(&self, data: &[u8]) -> Result<Signal>;
}

pub type NativeCallback = Box<
  dyn Fn(
      u64,
      &str,
      &str,
      &str,
      &OutputPayload,
    ) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>
    + 'static
    + Sync
    + Send,
>;

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) name: String,
  pub(crate) signing_seed: String,
}

impl Handler<Initialize> for NativeComponentActor {
  type Result = Result<()>;

  fn handle(&mut self, msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
    trace!("Native actor initialized");
    self.name = msg.name;
    self.seed = msg.signing_seed;
    self.state = Some(Arc::new(Mutex::new(State {})));
    Ok(())
  }
}

impl Handler<Invocation> for NativeComponentActor {
  type Result = InvocationResponse;

  fn handle(&mut self, msg: Invocation, _ctx: &mut Self::Context) -> Self::Result {
    trace!(
      "Native actor Invocation - From {} to {}",
      msg.origin.url(),
      msg.target.url()
    );
    let target = msg.target.url();

    let inv_id = msg.id;
    let state = self.state.as_ref().unwrap().clone();

    if let VinoEntity::Component(name) = &msg.target {
      trace!("Getting actor by name: {:?}", name);
      let component = native_actors::get_native_actor(name);
      match component {
        Some(component) => {
          if let MessagePayload::MultiBytes(payload) = msg.msg {
            trace!("Payload is : {:?}", payload);
            match serialize(payload) {
              Ok(payload) => {
                trace!("executing actor {}", target);
                // TODO fix async
                let mut receiver = block_on(component.job_wrapper(state, &payload));
                match receiver.borrow_mut() {
                  Err(e) => {
                    error!("{}", e.to_string());
                    InvocationResponse::error(msg.tx_id, e.to_string())
                  }
                  Ok(receiver) => {
                    loop {
                      let next = block_on(receiver.next());
                      if next.is_none() {
                        break;
                      }

                      let (port_name, msg) = next.unwrap();
                      let kp = KeyPair::from_seed(&self.seed).unwrap();
                      trace!(
                        "Native actor got output on port [{}]: result: {:?}",
                        port_name,
                        msg
                      );
                      let _result =
                        native_host_callback(kp, &inv_id, "", &port_name, &msg).unwrap();
                    }
                    InvocationResponse::success(
                      msg.tx_id,
                      serialize("done").unwrap_or_else(|_| serialize(Signal::Done).unwrap()),
                    )
                  }
                }
              }
              Err(e) => {
                InvocationResponse::error(msg.tx_id, format!("Could not serialize payload: {}", e))
              }
            }
          } else {
            trace!("Invalid payload");
            InvocationResponse::error(
              msg.tx_id,
              "Invalid payload sent from native actor".to_string(),
            )
          }
        }
        None => {
          trace!("Actor not found: {:?}", name);

          InvocationResponse::error(msg.tx_id, "Sent invocation for incorrect actor".to_string())
        }
      }
    } else {
      InvocationResponse::error(
        msg.tx_id,
        "Sent invocation for incorrect entity".to_string(),
      )
    }
  }
}
