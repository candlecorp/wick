#![deny(
    // warnings,
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

#[macro_use]
mod macros {
  macro_rules! actix_try {
    ($expr:expr $(,)?) => {
      match $expr {
        Ok(val) => val,
        Err(err) => {
          error!("Unexpected error: {}", err);
          return ActorResult::reply(Err(From::from(err)));
        }
      }
    };
  }

  macro_rules! actix_ensure_ok {
    ($expr:expr $(,)?) => {
      match $expr {
        Ok(val) => val,
        Err(err) => {
          return ActorResult::reply(err);
        }
      }
    };
  }
}

mod actix;
pub mod component_model;
pub mod components;
pub(crate) mod dispatch;
pub mod error;
pub(crate) mod invocation_map;
pub mod network;
pub mod network_definition;
// pub mod network_provider;
pub mod provider_model;
pub(crate) mod schematic;
pub mod schematic_definition;
pub mod schematic_model;
mod transaction;
pub(crate) mod util;

// pub use network_provider::Provider as NetworkProvider;

pub mod prelude {
  pub use vino_component::Packet;
  pub use vino_transport::MessageTransport;

  pub use crate::dispatch::{
    Invocation,
    InvocationResponse,
    ResponseStream,
  };
  pub use crate::{
    Error,
    Result,
    SCHEMATIC_INPUT,
    SCHEMATIC_OUTPUT,
  };
}

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
pub use crate::schematic::SchematicOutput;
pub use crate::schematic_definition::SchematicDefinition;

pub type Result<T> = std::result::Result<T, error::VinoError>;
pub type Error = error::VinoError;

pub use crate::components::vino_component::WapcComponent;
pub use crate::components::{
  load_wasm,
  load_wasm_from_file,
  load_wasm_from_oci,
};

/// The reserved reference name for schematic input. Used in schematic manifests to denote schematic input.
pub const SCHEMATIC_INPUT: &str = "<input>";
/// The reserved reference name for schematic output. Used in schematic manifests to denote schematic output.
pub const SCHEMATIC_OUTPUT: &str = "<output>";
/// The reserved port name to use when sending an asynchronous error from a component.
pub const COMPONENT_ERROR: &str = "<error>";

#[macro_use]
extern crate derivative;

#[macro_use]
extern crate vino_macros;

#[macro_use]
extern crate tracing;
