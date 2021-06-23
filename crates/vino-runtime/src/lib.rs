#![deny(
    warnings,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    unreachable_pub,
    type_alias_bounds,
    trivial_bounds,
    mutable_transmutes,
    invalid_value,
    explicit_outlives_requirements,
    deprecated,
    clashing_extern_declarations,
    clippy::expect_used,
    clippy::explicit_deref_methods,
    // missing_docs
)]
#![warn(clippy::cognitive_complexity)]

pub mod components;
pub(crate) mod dispatch;
pub mod error;
pub(crate) mod native_actors;
pub mod network;
pub mod network_definition;
pub(crate) mod schematic;
pub mod schematic_definition;
pub(crate) mod schematic_response;
pub(crate) mod util;

pub use crate::dispatch::{
  Invocation,
  InvocationResponse,
  PortEntity,
  VinoEntity,
};
pub use crate::network::{
  request,
  Network,
};
pub use crate::network_definition::NetworkDefinition;
pub use crate::schematic_definition::SchematicDefinition;

pub type Result<T> = std::result::Result<T, error::VinoError>;
pub type Error = error::VinoError;

pub use crate::components::vino_component::WapcComponent;
pub use crate::components::{
  load_wasm,
  load_wasm_from_file,
  load_wasm_from_oci,
};

#[doc(hidden)]
pub const SYSTEM_ACTOR: &str = "system";
pub const VINO_NAMESPACE: &str = "vino";
pub const SCHEMATIC_INPUT: &str = "vino::schematic_input";
pub const SCHEMATIC_OUTPUT: &str = "vino::schematic_output";

#[macro_use]
extern crate log;

#[macro_use]
extern crate derivative;

#[macro_use]
extern crate vino_macros;

#[macro_use]
extern crate derive_new;
