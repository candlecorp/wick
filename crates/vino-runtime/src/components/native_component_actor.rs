use crate::dispatch::{Invocation, InvocationResponse, MessagePayload, VinoEntity};
use crate::native_actors;
use crate::{serialize, Result};

use crate::components::vino_component::NativeComponent;
use actix::prelude::*;
use nkeys::KeyPair;
use vino_guest::{OutputPayload, Signal};

#[derive(Default)]
pub(crate) struct NativeComponentActor {
    component: Option<Box<dyn NativeActor>>,
    name: String,
    seed: String,
}

impl Actor for NativeComponentActor {
    type Context = SyncContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        trace!("Native actor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {}
}

pub(crate) trait NativeActor {
    fn get_def(&self) -> NativeComponent;
    fn get_name(&self) -> String;
    fn get_input_ports(&self) -> Vec<String>;
    fn get_output_ports(&self) -> Vec<String>;
    fn job_wrapper(&self, data: &[u8]) -> Result<Signal>;
}

pub(crate) type NativeCallback = Box<
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
        let seed = self.seed.to_string();

        let callback: NativeCallback = Box::new(
            move |_id: u64, bind: &str, ns: &str, op: &str, payload: &OutputPayload| {
                crate::dispatch::native_host_callback(
                    KeyPair::from_seed(&seed).unwrap(),
                    bind,
                    ns,
                    op,
                    payload,
                )
            },
        );

        self.component = native_actors::new_native_actor(&self.name, callback);

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

        if let VinoEntity::Component(_) = &msg.target {
            match &self.component {
                Some(actor) => {
                    if let MessagePayload::MultiBytes(payload) = msg.msg {
                        match serialize((inv_id, payload)) {
                            Ok(payload) => {
                                trace!("executing actor {}", target);
                                let result = actor.job_wrapper(&payload);
                                trace!("done executing actor {}: result: {:?}", target, result);
                                match result {
                                    Err(e) => {
                                        error!("{}", e.to_string());
                                        InvocationResponse::error(msg.tx_id, e.to_string())
                                    }
                                    Ok(p) => InvocationResponse::success(
                                        msg.tx_id,
                                        serialize(p)
                                            .unwrap_or_else(|_| serialize(Signal::Done).unwrap()),
                                    ),
                                }
                            }
                            Err(e) => InvocationResponse::error(
                                msg.tx_id,
                                format!("Could not serialize payload: {}", e),
                            ),
                        }
                    } else {
                        InvocationResponse::error(
                            msg.tx_id,
                            "Invalid payload sent from native actor".to_string(),
                        )
                    }
                }
                None => InvocationResponse::error(
                    msg.tx_id,
                    "Sent invocation for incorrect actor".to_string(),
                ),
            }
        } else {
            InvocationResponse::error(
                msg.tx_id,
                "Sent invocation for incorrect entity".to_string(),
            )
        }
    }
}
