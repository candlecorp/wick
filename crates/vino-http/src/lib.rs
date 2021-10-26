pub use config::Config;
use vino_rpc::SharedRpcHandler;

mod config;
mod cors;
mod error;
pub mod service;

use std::future::Future;
use std::pin::Pin;

pub use error::HttpError as Error;

use crate::service::ProviderService;

#[macro_use]
extern crate tracing;

/// enable a vino provider to handle http web requests with the default configuration.
///
/// Shortcut for `vino_http::config().enable(service)`
pub fn enable(provider: SharedRpcHandler) -> ProviderService {
  config().enable(provider)
}

/// returns a default [`Config`] instance for configuring services.
///
/// ## Example
///
/// ```
/// let config = vino_http::config()
///      .allow_origins(vec!["http://foo.com"])
///      .allow_credentials(false)
///      .expose_headers(vec!["x-request-id"]);
///
/// // let greeter = config.enable(Greeter);
/// ```
pub fn config() -> Config {
  Config::default()
}

// type BoxError = Box<dyn std::error::Error + Send + Sync>;
type BoxFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send>>;
