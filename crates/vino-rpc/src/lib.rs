pub mod error;
pub mod handlers;
pub mod peer;
pub mod rpc;

pub type Error = error::RpcError;
pub type Result<T> = std::result::Result<T, Error>;

#[cfg_attr(test, macro_use)]
extern crate vino_macros;

#[macro_use]
extern crate log;
