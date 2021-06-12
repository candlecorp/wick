#![deny(
    warnings,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    unreachable_pub
    // missing_docs
)]

#[macro_use]
extern crate log;

#[macro_use]
pub(crate) mod macros;

mod builder;
mod error;
mod host;
pub mod host_definition;

pub use builder::HostBuilder;
pub use host::Host;
pub use host_definition::HostDefinition;

pub type Result<T> = std::result::Result<T, error::VinoHostError>;
pub type Error = error::VinoHostError;

#[cfg(test)]
mod test {}
