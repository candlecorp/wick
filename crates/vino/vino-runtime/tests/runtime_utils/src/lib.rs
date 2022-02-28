use std::fs;

use vino_manifest::{HostManifest, Loadable, NetworkDefinition, NetworkManifest, SchematicDefinition};
use vino_runtime::{Network, NetworkBuilder};
use vino_wascap::KeyPair;
#[macro_use]
extern crate tracing;

#[allow(dead_code)]
pub async fn init_network_from_yaml(path: &str) -> anyhow::Result<(Network, String)> {
  let manifest = HostManifest::from_yaml(&fs::read_to_string(path)?)?;
  let allow_latest = manifest.allow_latest();
  let insecure_registries = manifest.insecure_registries().clone();
  let def = NetworkDefinition::from(manifest.network());
  debug!("Manifest loaded");
  let kp = KeyPair::new_server();

  let builder = NetworkBuilder::from_definition(def, &kp.seed()?)?;

  let builder = builder.allow_latest(allow_latest).allow_insecure(insecure_registries);
  let network = builder.build();

  debug!("Initializing network");
  let init = network.init().await;
  info!("Init status : {:?}", init);
  init?;

  let nuid = network.uid.clone();
  Ok((network, nuid))
}

#[allow(dead_code)]
pub fn load_network_manifest(path: &str) -> anyhow::Result<NetworkDefinition> {
  let manifest = NetworkManifest::V0(vino_manifest::v0::NetworkManifest::from_yaml(&fs::read_to_string(
    path,
  )?)?);
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
