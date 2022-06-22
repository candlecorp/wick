pub(crate) mod prelude {
  pub(crate) use anyhow::Result as TestResult;
  pub(crate) use pretty_assertions::assert_eq;

  pub(crate) use super::*;
  pub(crate) use crate::dev::prelude::*;
}

use wasmflow_manifest::WasmflowManifest;
use wasmflow_wascap::KeyPair;

use crate::test::prelude::*;
use crate::NetworkBuilder;

pub(crate) async fn init_network_from_yaml(path: &str) -> TestResult<(Network, uuid::Uuid)> {
  let def = WasmflowManifest::load_from_file(path)?;
  let kp = KeyPair::new_server();

  let network = NetworkBuilder::from_definition(def, &kp.seed().map_err(|_| crate::Error::NoSeed)?)?
    .build()
    .await?;

  let network_id = network.uid;
  trace!(network_id = %network_id, "network uid");
  Ok((network, network_id))
}
