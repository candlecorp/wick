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
#![warn(clippy::cognitive_complexity)]

#[macro_use]
mod macros {
  macro_rules! meh_actix {
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

  macro_rules! actix_bail {
    ($expr:expr $(,)?) => {
      match $expr {
        Ok(val) => val,
        Err(err) => {
          return ActorResult::reply(err);
        }
      }
    };
  }

  #[allow(unused_macros)]
  macro_rules! log_tap {
    ($expr:expr $(,)?) => {{
      let _e = $expr;
      trace!("{:?}", $expr);
      _e
    }};
  }

  macro_rules! meh {
    ($expr:expr $(,)?) => {{
      match $expr {
        Ok(_) => {}
        Err(e) => {
          error!("Unexpected error: {}", e);
        }
      }
    }};
  }
}

mod actix;
pub(crate) mod component_model;
pub mod components;
pub(crate) mod dispatch;
pub mod error;
pub mod network;
pub mod network_definition;
pub(crate) mod provider_model;
pub(crate) mod schematic;
pub mod schematic_definition;
pub(crate) mod schematic_model;
pub(crate) mod schematic_response;
mod transaction;
pub(crate) mod util;

pub mod prelude {
  pub use crate::dispatch::Invocation;
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
pub const SCHEMATIC_INPUT: &str = "::input";
/// The reserved reference name for schematic output. Used in schematic manifests to denote schematic output.
pub const SCHEMATIC_OUTPUT: &str = "::output";

#[macro_use]
extern crate log;

#[macro_use]
extern crate derivative;

#[macro_use]
extern crate vino_macros;

#[allow(unused_imports)]
#[macro_use]
extern crate log_derive;
