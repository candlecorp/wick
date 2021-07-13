use std::fs;

pub use anyhow::Result;
pub use maplit::hashmap;
pub use pretty_assertions::assert_eq as equals;
pub use tracing::*;
use vino_manifest::{
  Loadable,
  NetworkDefinition,
  NetworkManifest,
  SchematicDefinition,
};
use vino_runtime::network::Network;
use vino_wascap::KeyPair;

#[allow(dead_code)]
pub async fn init_network_from_yaml(path: &str) -> Result<(Network, String)> {
  let manifest = NetworkManifest::V0(vino_manifest::v0::NetworkManifest::from_yaml(
    &fs::read_to_string(path)?,
  )?);
  let def = NetworkDefinition::from(manifest);
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

#[allow(dead_code)]
pub fn load_network_manifest(path: &str) -> Result<NetworkDefinition> {
  let manifest = NetworkManifest::V0(vino_manifest::v0::NetworkManifest::from_yaml(
    &fs::read_to_string(path)?,
  )?);
  let def = NetworkDefinition::from(manifest);
  debug!("Manifest loaded");
  Ok(def)
}

#[allow(dead_code)]
pub fn new_schematic(name: &str) -> SchematicDefinition {
  SchematicDefinition {
    name: name.to_owned(),
    ..SchematicDefinition::default()
  }
}
