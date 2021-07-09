pub mod components;
pub mod error;
pub(crate) mod generated;
pub mod provider;

pub(crate) use provider::State;

pub type Error = error::InMemoryCollectionError;
pub type Result<T> = std::result::Result<T, Error>;

#[macro_use]
extern crate tracing;
