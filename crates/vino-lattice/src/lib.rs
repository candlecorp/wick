pub mod error;
pub mod lattice;
pub mod nats;
pub use error::LatticeError as Error;

#[macro_use]
extern crate log;
