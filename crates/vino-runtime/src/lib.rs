use actix::prelude::*;
use network::ActorPorts;

pub(crate) mod connection_downstream;
pub(crate) mod dispatch;
pub mod error;
pub(crate) mod hlreg;
pub mod manifest;
pub(crate) mod native_actors;
pub(crate) mod native_component_actor;
pub(crate) mod network;
pub(crate) mod oci;
pub(crate) mod port_entity;
pub(crate) mod schematic;
pub(crate) mod schematic_response;
pub(crate) mod serdes;
pub(crate) mod vino_component;
pub(crate) mod wapc_component_actor;

pub use self::manifest::runtime_definition::RuntimeManifest;
pub use self::manifest::schematic_definition::SchematicDefinition;
use crate::dispatch::MessagePayload;
pub(crate) use native_component_actor::NativeComponentActor;

pub use crate::manifest::run_config::RunConfig;
pub use dispatch::{Invocation, InvocationResponse};
pub use serdes::{deserialize, serialize};

pub type Result<T> = anyhow::Result<T, error::VinoError>;
#[doc(hidden)]
pub const SYSTEM_ACTOR: &str = "system";
pub const VINO_NAMESPACE: &str = "vino";
pub const SCHEMATIC_INPUT: &str = "vino::schematic_input";
pub const SCHEMATIC_OUTPUT: &str = "vino::schematic_output";

pub use network::request;

#[macro_use]
mod native_macro;

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

#[derive(Message)]
#[rtype(result = "()")]
pub(crate) struct Initialize {
    pub host_id: String,
    pub seed: String,
}

#[derive(Message)]
#[rtype(result = "bool")]
pub(crate) struct HasSchematic {
    pub schematic: String,
}
#[derive(Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct RegisterReference {
    pub namespace: String,
    pub id: String,
    pub reference: String,
    pub ports: ActorPorts,
}

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct RegisterSchematic {
    pub schematic: SchematicDefinition,
}

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct SchematicFuture {
    pub tx_id: String,
    pub schematic: String,
}
