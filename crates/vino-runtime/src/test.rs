pub(crate) mod prelude {
  pub(crate) use maplit::hashmap;
  pub(crate) use pretty_assertions::assert_eq as equals;

  pub(crate) use super::*;
  pub(crate) use crate::dev::prelude::*;
}

use std::fs;

use vino_manifest::{
  Loadable,
  NetworkManifest,
};
use wascap::prelude::KeyPair;

use crate::test::prelude::*;

pub(crate) async fn init_network_from_yaml(path: &str) -> Result<(Network, String)> {
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

pub(crate) fn load_network_manifest(path: &str) -> Result<NetworkDefinition> {
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
