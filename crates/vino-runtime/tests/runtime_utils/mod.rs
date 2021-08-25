use std::fs;

pub use anyhow::Result;
pub use log::*;
pub use maplit::hashmap;
pub use pretty_assertions::assert_eq as equals;
use vino_manifest::{
  HostManifest,
  Loadable,
  NetworkDefinition,
  NetworkManifest,
  SchematicDefinition,
};
use vino_runtime::network::{
  Network,
  NetworkBuilder,
};
use vino_wascap::KeyPair;

#[allow(dead_code)]
pub async fn init_network_from_yaml(path: &str) -> Result<(Network, String)> {
  let manifest = HostManifest::from_yaml(&fs::read_to_string(path)?)?;
  let def = NetworkDefinition::from(manifest.network());
  debug!("Manifest loaded");
  let kp = KeyPair::new_server();

  let builder = NetworkBuilder::new(def, &kp.seed()?)?;
  let network = builder.from_env().build();

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
