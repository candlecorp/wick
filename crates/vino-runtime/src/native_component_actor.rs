use crate::dispatch::{Invocation, InvocationResponse, MessagePayload, VinoEntity};
use crate::{serialize, Result};

use crate::native_actors;
use crate::network::ActorPorts;
use crate::vino_component::NativeComponent;
use actix::prelude::*;
use vino_guest::Signal;

#[derive(Default)]
pub(crate) struct NativeComponentActor {
    actor: Option<Box<dyn NativeActor>>,
    name: String,
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

#[derive(Message)]
#[rtype(result = "Result<ActorPorts>")]
pub(crate) struct Initialize {
    pub name: String,
}

impl Handler<Initialize> for NativeComponentActor {
    type Result = Result<ActorPorts>;

    fn handle(&mut self, msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
        trace!("Native actor initialized");
        let name = msg.name;
        self.name = name.to_string();
        let actor = native_actors::get_native_actor(name.to_string());
        if actor.is_none() {
            return Err(anyhow!("Unknown actor {}", name).into());
        }
        let (inputs, outputs) = match &actor {
            Some(actor) => (actor.get_input_ports(), actor.get_output_ports()),
            None => (vec![], vec![]),
        };
        self.actor = actor;

        Ok(ActorPorts::new(inputs, outputs))
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

        if let VinoEntity::Component(_) = &msg.target {
            match &self.actor {
                Some(actor) => {
                    if let MessagePayload::Bytes(payload) = msg.msg {
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
                                serialize(p).unwrap_or_else(|_| serialize(Signal::Done).unwrap()),
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
