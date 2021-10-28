use std::fs;
use std::sync::Arc;
use std::time::Duration;

use vino_lattice::nats::NatsOptions;
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
#[macro_use]
extern crate tracing;

#[allow(dead_code)]
pub async fn init_network_from_yaml(path: &str) -> anyhow::Result<(Network, String)> {
  let manifest = HostManifest::from_yaml(&fs::read_to_string(path)?)?;
  let def = NetworkDefinition::from(manifest.network());
  debug!("Manifest loaded");
  let kp = KeyPair::new_server();

  let mut builder = NetworkBuilder::from_definition(def, &kp.seed()?)?;
  #[cfg(feature = "test-integration")]
  if let Ok(url) = std::env::var("NATS_URL") {
    let lattice = vino_lattice::lattice::Lattice::connect(NatsOptions {
      address: url,
      client_id: "test".to_owned(),
      creds_path: None,
      token: None,
      timeout: Duration::from_secs(5),
    })
    .await?;
    builder = builder.lattice(Arc::new(lattice));
  }
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
