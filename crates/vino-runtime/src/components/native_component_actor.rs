use std::sync::{
  Arc,
  Mutex,
};

use actix::prelude::*;
use futures::{
  FutureExt,
  StreamExt,
};
use nkeys::KeyPair;
use vino_guest::{
  OutputPayload,
  Signal,
};
use vino_transport::serialize;

use crate::components::vino_component::NativeComponent;
use crate::dispatch::{
  native_host_callback,
  InvocationResponse,
};
use crate::native_actors::State;
use crate::{
  error,
  native_actors,
  Invocation,
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
  type Context = Context<Self>;

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
  type Result = ResponseFuture<InvocationResponse>;

  fn handle(&mut self, msg: Invocation, _ctx: &mut Self::Context) -> Self::Result {
    trace!(
      "Native actor Invocation - From {} to {}",
      msg.origin.url(),
      msg.target.url()
    );
    let target_url = msg.target.url();
    let target = msg.target;
    let payload = msg.msg;
    let tx_id = msg.tx_id;
    let tx_id2 = tx_id.clone();

    let inv_id = msg.id;
    let state = self.state.as_ref().unwrap().clone();
    let seed = self.seed.clone();
    let fut = async move {
      let entity = target
        .into_component()
        .map_err(|_| "Provider received invalid invocation")?;
      debug!("Getting component: {}", entity);
      let component = native_actors::get_native_actor(&entity.name).ok_or_else(|| {
        error::VinoError::ComponentError(format!("Component {} not found", entity))
      })?;
      let payload = payload
        .into_multibytes()
        .map_err(|_| error::VinoError::ComponentError("Provider sent invalid payload".into()))?;
      trace!("Payload is : {:?}", payload);
      let payload = serialize(payload).map_err(|_| "Could not serialize input payload")?;

      trace!("executing actor {}", target_url);
      let mut receiver = component.job_wrapper(state, &payload).await?;
      actix::spawn(async move {
        loop {
          trace!("Native component {} waiting for output", entity);
          let next = receiver.next().await;
          if next.is_none() {
            break;
          }

          let (port_name, msg) = next.unwrap();
          let kp = KeyPair::from_seed(&seed).unwrap();
          trace!(
            "Native actor {} got output on port [{}]: result: {:?}",
            entity,
            port_name,
            msg
          );
          let _result = native_host_callback(kp, &inv_id, "", &port_name, &msg).unwrap();
        }
      });
      Ok!(InvocationResponse::success(tx_id, vec![],))
    };

    Box::pin(fut.then(|result| async {
      match result {
        Ok(invocation) => invocation,
        Err(e) => InvocationResponse::error(tx_id2, e.to_string()),
      }
    }))
  }
}
