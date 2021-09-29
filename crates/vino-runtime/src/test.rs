pub(crate) mod prelude {
  pub(crate) use anyhow::Result as TestResult;
  pub(crate) use maplit::hashmap;
  pub(crate) use pretty_assertions::assert_eq;

  pub(crate) use super::*;
  pub(crate) use crate::dev::prelude::*;
}

use std::fs;

use vino_manifest::{
  HostManifest,
  Loadable,
};
use vino_wascap::KeyPair;

use crate::error::CommonError;
use crate::test::prelude::*;
pub(crate) async fn init_network_from_yaml(path: &str) -> TestResult<(Network, String)> {
  let def = load_network_definition(path)?;
  let kp = KeyPair::new_server();
  let network = Network::new(def, &kp.seed().map_err(|_| CommonError::NoSeed)?)?;
  debug!("Initializing network");
  let init = network.init().await;
  info!("Init status : {:?}", init);
  init?;

  let network_id = network.uid.clone();
  Ok((network, network_id))
}

pub(crate) fn load_network_definition(path: &str) -> TestResult<NetworkDefinition> {
  let manifest = HostManifest::from_yaml(&fs::read_to_string(path)?)?;
  let def = NetworkDefinition::from(manifest.network());
  debug!("Manifest loaded");
  Ok(def)
}

pub(crate) fn new_schematic(name: &str) -> SchematicDefinition {
  SchematicDefinition {
    name: name.to_owned(),
    ..SchematicDefinition::default()
  }
}
