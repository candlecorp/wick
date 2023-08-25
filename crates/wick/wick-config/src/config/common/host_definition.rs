#![allow(missing_docs)] // delete when we move away from the `property` crate.
use std::net::Ipv4Addr;

use crate::config;

#[derive(Debug, Clone, Default, derive_builder::Builder, property::Property, serde::Serialize)]
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
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) insecure_registries: Vec<String>,

  /// Configuration for the GRPC server.
  #[builder(setter(strip_option), default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) rpc: Option<HttpConfig>,
}

#[derive(Debug, Default, Clone, derive_builder::Builder, property::Property, serde::Serialize)]
#[builder(setter(into))]
#[property(get(public), set(private), mut(public, suffix = "_mut"))]
/// Configuration for HTTP/S servers.
pub struct HttpConfig {
  /// Enable/disable the server.
  #[builder(default)]
  pub(crate) enabled: bool,

  /// The port to bind to.
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) port: Option<u16>,

  /// The address to bind to.
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) address: Option<Ipv4Addr>,

  /// Path to pem file for TLS.
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) pem: Option<config::AssetReference>,

  /// Path to key file for TLS.
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) key: Option<config::AssetReference>,

  /// Path to CA file.
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) ca: Option<config::AssetReference>,
}
