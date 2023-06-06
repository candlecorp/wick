#![allow(missing_docs)] // delete when we move away from the `property` crate.
use std::net::Ipv4Addr;

use crate::config;

#[derive(Debug, Clone, Default, Builder, property::Property)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[builder(setter(into))]
#[must_use]
/// Configuration options for the host to use at startup.
pub struct HostConfig {
  /// Flag to allow/disallow `:latest` tags for OCI artifacts.
  #[builder(default)]
  pub(crate) allow_latest: bool,

  /// The list of registries to connect via HTTP rather than HTTPS.
  #[builder(default)]
  pub(crate) insecure_registries: Vec<String>,

  /// Configuration for the GRPC server.
  #[builder(setter(strip_option), default)]
  pub(crate) rpc: Option<HttpConfig>,
}

#[derive(Debug, Default, Clone, Builder, property::Property)]
#[builder(setter(into))]
#[property(get(public), set(private), mut(public, suffix = "_mut"))]
/// Configuration for HTTP/S servers.
pub struct HttpConfig {
  /// Enable/disable the server.
  #[builder(default)]
  pub(crate) enabled: bool,

  /// The port to bind to.
  #[builder(default)]
  pub(crate) port: Option<u16>,

  /// The address to bind to.
  #[builder(default)]
  pub(crate) address: Option<Ipv4Addr>,

  /// Path to pem file for TLS.
  #[builder(default)]
  pub(crate) pem: Option<config::AssetReference>,

  /// Path to key file for TLS.
  #[builder(default)]
  pub(crate) key: Option<config::AssetReference>,

  /// Path to CA file.
  #[builder(default)]
  pub(crate) ca: Option<config::AssetReference>,
}
