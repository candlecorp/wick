use crate::dispatch::VinoEntity;
use crate::vino_component::WapcComponent;

use crate::dispatch::{Invocation, InvocationResponse, MessagePayload};
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
    _seed: String,
}

#[derive(Message, Debug)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
    pub actor_bytes: Vec<u8>,
    pub signing_seed: String,
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

    #[cfg(feature = "wasmtime")]
    let engine = { wasmtime_provider::WasmtimeEngineProvider::new(&buf, None) };
    #[cfg(feature = "wasm3")]
    let _engine = wasm3_provider::Wasm3EngineProvider::new(&buf);

    let cloned_claims = claims.clone();
    let seed = msg.signing_seed.to_string();

    let guest = WapcHost::new(
        Box::new(engine),
        move |id, binding, namespace, operation, payload| {
            trace!(
                "wapc callback {}  {}  {}  {} ",
                id,
                binding,   //tx-id
                namespace, // SCHEMATIC_name||reference
                operation  // port
            );
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
            let _entity = VinoEntity::Component(claims.subject.to_string());
            // let b = MessageBus::from_hostlocal_registry(&msg.host_id);
            // let recipient = ctx.address().recipient();
            // let _ = block_on(async move {
            //     b.send(Subscribe {
            //         interest: entity,
            //         subscriber: recipient,
            //     })
            //     .await
            // });

            me.state = Some(State {
                guest_module: g,
                claims: claims.clone(),
                _seed: msg.signing_seed,
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

        debug!(
            "Actor Invocation - From {} to {}: {}",
            msg.origin.url(),
            msg.target.url(),
            msg.operation
        );

        if let VinoEntity::Component(_) = msg.target {
            if let MessagePayload::Bytes(payload) = &msg.msg {
                match state.guest_module.call(&msg.operation, &payload) {
                    Ok(bytes) => InvocationResponse::success(msg.tx_id, bytes),
                    Err(e) => {
                        error!("Error invoking actor: {} (from {})", e, msg.target.url());
                        debug!("Message: {:?}", &msg.msg);
                        InvocationResponse::error(msg.tx_id, e.to_string())
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
