pub(crate) mod components;
pub(crate) mod dispatch;
pub mod error;
pub mod manifest;
pub(crate) mod native_actors;
pub mod network;
pub(crate) mod schematic;
pub(crate) mod schematic_response;
pub(crate) mod util;

pub use crate::dispatch::{Invocation, InvocationResponse};
pub use crate::manifest::network_manifest::NetworkManifest;
pub use crate::manifest::schematic_definition::SchematicDefinition;
pub use crate::network::{request, Network};
pub use crate::util::serdes::{deserialize, serialize};

pub use crate::dispatch::MessagePayload;

pub type Result<T> = anyhow::Result<T, error::VinoError>;
pub type Error = error::VinoError;

pub use crate::components::{
    load_wasm, load_wasm_from_file, load_wasm_from_oci, vino_component::WapcComponent,
};

#[doc(hidden)]
pub const SYSTEM_ACTOR: &str = "system";
pub const VINO_NAMESPACE: &str = "vino";
pub const SCHEMATIC_INPUT: &str = "vino::schematic_input";
pub const SCHEMATIC_OUTPUT: &str = "vino::schematic_output";

#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;
