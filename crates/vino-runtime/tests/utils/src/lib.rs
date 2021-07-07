pub use maplit::hashmap;
pub use pretty_assertions::assert_eq as equals;

pub type TestResult<T> = Result<T, TestError>;

#[macro_use]
extern crate tracing;

use std::fs;

use thiserror::Error;
use vino_manifest::{
  Loadable,
  NetworkDefinition,
  NetworkManifest,
  SchematicDefinition,
};
use vino_runtime::error::*;
use vino_runtime::network::Network;
use vino_wascap::KeyPair;

pub async fn init_network_from_yaml(path: &str) -> TestResult<(Network, String)> {
  let manifest = NetworkManifest::V0(vino_manifest::v0::NetworkManifest::from_yaml(
    &fs::read_to_string(path)?,
  )?);
  let def = NetworkDefinition::from(&manifest);
  debug!("Manifest loaded");
  let kp = KeyPair::new_server();

  let network = Network::new(def, &kp.seed()?)?;
  debug!("Initializing network");
  let init = network.init().await;
  info!("Init status : {:?}", init);
  init?;

  let network_id = network.id.clone();
  Ok((network, network_id))
}

pub fn load_network_manifest(path: &str) -> TestResult<NetworkDefinition> {
  let manifest = NetworkManifest::V0(vino_manifest::v0::NetworkManifest::from_yaml(
    &fs::read_to_string(path)?,
  )?);
  let def = NetworkDefinition::from(&manifest);
  debug!("Manifest loaded");
  Ok(def)
}

pub fn new_schematic(name: &str) -> SchematicDefinition {
  SchematicDefinition {
    name: name.to_owned(),
    ..SchematicDefinition::default()
  }
}

#[derive(Error, Debug)]
pub enum TestError {
  #[error(transparent)]
  CommonError(#[from] CommonError),
  #[error(transparent)]
  ComponentError(#[from] ComponentError),
  #[error(transparent)]
  NetworkError(#[from] NetworkError),
  #[error(transparent)]
  VinoError(#[from] VinoError),

  #[error(transparent)]
  OciError(#[from] OciError),
  #[error(transparent)]
  SchematicError(#[from] SchematicError),

  // #[error(transparent)]
  // TonicError(#[from] tonic::transport::Error),
  // #[error(transparent)]
  // RpcUpstreamError(#[from] tonic::Status),
  // #[error(transparent)]
  // EntityError(#[from] vino_entity::Error),
  // #[error(transparent)]
  // RpcError(#[from] vino_rpc::Error),
  #[error(transparent)]
  CodecError(#[from] vino_codec::Error),
  #[error(transparent)]
  ManifestError(#[from] vino_manifest::Error),
  #[error(transparent)]
  TransportError(#[from] vino_transport::Error),
  // #[error(transparent)]
  // OutputError(#[from] vino_component::Error),
  // #[error(transparent)]
  // ActixMailboxError(#[from] MailboxError),
  #[error(transparent)]
  KeyPairError(#[from] nkeys::error::Error),
  #[error(transparent)]
  ValidationError(#[from] ValidationError),
  #[error(transparent)]
  ModelError(#[from] SchematicModelError),

  #[error(transparent)]
  IOError(#[from] std::io::Error),
}
