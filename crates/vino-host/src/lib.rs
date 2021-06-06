#[macro_use]
extern crate log;

#[macro_use]
pub(crate) mod macros;

mod builder;
mod error;
mod host;
pub mod manifest;

pub use builder::HostBuilder;
pub use host::Host;
pub use manifest::HostManifest;

pub type Result<T> = std::result::Result<T, error::VinoHostError>;
pub type Error = error::VinoHostError;

#[cfg(test)]
mod test {}
