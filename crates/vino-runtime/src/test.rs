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
  SchematicManifest,
};
use vino_wascap::KeyPair;

use crate::error::CommonError;
use crate::test::prelude::*;

pub(crate) async fn init_network_from_yaml(path: &str) -> TestResult<(Network, String)> {
  let manifest = NetworkManifest::V0(vino_manifest::v0::NetworkManifest::from_yaml(
    &fs::read_to_string(path)?,
  )?);
  let def = NetworkDefinition::new(&manifest);
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

pub(crate) fn load_network_manifest(path: &str) -> TestResult<NetworkDefinition> {
  let manifest = NetworkManifest::V0(vino_manifest::v0::NetworkManifest::from_yaml(
    &fs::read_to_string(path)?,
  )?);
  let def = NetworkDefinition::new(&manifest);
  debug!("Manifest loaded");
  Ok(def)
}

pub(crate) fn load_schematic_manifest(path: &str) -> TestResult<SchematicDefinition> {
  let manifest = SchematicManifest::V0(vino_manifest::v0::SchematicManifest::from_yaml(
    &fs::read_to_string(path)?,
  )?);
  let def = SchematicDefinition::from_manifest(&manifest)?;
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
  ModelError(#[from] SchematicModelError),

  #[error(transparent)]
  OtherUpstream(#[from] BoxedErrorSyncSend),

  #[error(transparent)]
  IOError(#[from] std::io::Error),
}
