pub(crate) mod prelude {
  pub(crate) use maplit::hashmap;
  pub(crate) use pretty_assertions::assert_eq as equals;

  pub(crate) use super::*;
  pub(crate) use crate::dev::prelude::*;

  pub(crate) type TestResult<T> = Result<T, TestError>;
}

use std::fs;

use thiserror::Error;
use vino_manifest::{
  Loadable,
  NetworkManifest,
};
use wascap::prelude::KeyPair;

use crate::error::CommonError;
use crate::test::prelude::*;

pub(crate) async fn init_network_from_yaml(path: &str) -> TestResult<(Network, String)> {
  let manifest = NetworkManifest::V0(vino_manifest::v0::NetworkManifest::from_yaml(
    &fs::read_to_string(path)?,
  )?);
  let def = NetworkDefinition::new(&manifest);
  debug!("Manifest loaded");
  let kp = KeyPair::new_server();

  let network = Network::new(def, &kp.seed()?);
  network.init().await?;

  trace!("Manifest applied");

  let network_id = network.id.clone();
  Ok((network, network_id))
}

pub(crate) fn load_network_manifest(path: &str) -> TestResult<NetworkDefinition> {
  let manifest = NetworkManifest::V0(vino_manifest::v0::NetworkManifest::from_yaml(
    &fs::read_to_string(path)?,
  )?);
  let def = NetworkDefinition::new(&manifest);
  debug!("Manifest loaded");
  Ok(def)
}

pub(crate) fn new_schematic(name: &str) -> SchematicDefinition {
  SchematicDefinition {
    name: name.to_owned(),
    ..SchematicDefinition::default()
  }
}

#[derive(Error, Debug)]
pub(crate) enum TestError {
  #[error("Invocation error: {0}")]
  InvocationError(String),
  #[error(transparent)]
  CommonError(#[from] CommonError),
  #[error("Conversion error {0}")]
  ConversionError(&'static str),
  #[error("URL parse error {0}")]
  ParseError(String),
  #[error(transparent)]
  ComponentError(#[from] ComponentError),
  #[error(transparent)]
  NetworkError(#[from] NetworkError),
  #[error(transparent)]
  VinoError(#[from] VinoError),
  #[error("Dispatch error: {0}")]
  DispatchError(String),
  #[error("Provider error {0}")]
  ProviderError(String),
  #[error("WaPC WebAssembly Component error: {0}")]
  WapcError(String),
  #[error("Job error: {0}")]
  JobError(String),
  #[error("invalid configuration")]
  ConfigurationError,
  #[error("Could not start host: {0}")]
  HostStartFailure(String),
  #[error("Failed to deserialize configuration {0}")]
  ConfigurationDeserialization(String),
  #[error("Failed to serialize payload {0}")]
  SerializationError(rmp_serde::encode::Error),
  #[error("Failed to deserialize payload {0}")]
  DeserializationError(rmp_serde::decode::Error),

  #[error(transparent)]
  OciError(#[from] OciError),
  #[error(transparent)]
  SchematicError(#[from] SchematicError),

  #[error(transparent)]
  TonicError(#[from] tonic::transport::Error),
  #[error(transparent)]
  RpcUpstreamError(#[from] tonic::Status),
  #[error(transparent)]
  EntityError(#[from] vino_entity::Error),
  #[error(transparent)]
  RpcError(#[from] vino_rpc::Error),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error(transparent)]
  ManifestError(#[from] vino_manifest::Error),
  #[error(transparent)]
  TransportError(#[from] vino_transport::Error),
  #[error(transparent)]
  OutputError(#[from] vino_component::Error),
  #[error(transparent)]
  ActixMailboxError(#[from] MailboxError),
  #[error(transparent)]
  KeyPairError(#[from] nkeys::error::Error),
  #[error(transparent)]
  ValidationError(#[from] ValidationError),

  #[error(transparent)]
  OtherUpstream(#[from] BoxedErrorSyncSend),
  #[error("General error : {0}")]
  Other(String),

  #[error(transparent)]
  IOError(#[from] std::io::Error),
}
