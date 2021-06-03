use actix::prelude::*;
use network::ActorPorts;

pub(crate) mod components;
pub(crate) mod connection_downstream;
pub(crate) mod dispatch;
pub mod error;
pub mod manifest;
pub(crate) mod native_actors;
pub(crate) mod network;
pub(crate) mod port_entity;
pub(crate) mod schematic;
pub(crate) mod schematic_response;
pub(crate) mod util;

pub use crate::dispatch::{Invocation, InvocationResponse};
pub use crate::manifest::run_config::RunConfig;
pub use crate::manifest::runtime_definition::RuntimeManifest;
pub use crate::manifest::schematic_definition::SchematicDefinition;
pub use crate::util::serdes::{deserialize, serialize};

use crate::dispatch::MessagePayload;

pub type Result<T> = anyhow::Result<T, error::VinoError>;
#[doc(hidden)]
pub const SYSTEM_ACTOR: &str = "system";
pub const VINO_NAMESPACE: &str = "vino";
pub const SCHEMATIC_INPUT: &str = "vino::schematic_input";
pub const SCHEMATIC_OUTPUT: &str = "vino::schematic_output";

pub use network::request;

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
