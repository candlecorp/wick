pub(crate) mod prelude {
  pub(crate) use anyhow::Result;
  pub(crate) use futures::StreamExt;
  pub(crate) use pretty_assertions::assert_eq;

  pub(crate) use super::*;
}

use wick_config::WickConfiguration;

use crate::test::prelude::*;
use crate::{Runtime, RuntimeBuilder};

pub(crate) async fn init_engine_from_yaml(path: &str) -> Result<(Runtime, uuid::Uuid)> {
  let def = WickConfiguration::load_from_file(path).await?.try_component_config()?;

  let engine = RuntimeBuilder::from_definition(def).build(None).await?;

  let engine_id = engine.uid;
  trace!(engine_id = %engine_id, "engine uid");
  Ok((engine, engine_id))
}

pub(crate) async fn load_test_manifest(name: &str) -> Result<WickConfiguration> {
  let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let manifest_dir = crate_dir.join("../../../tests/testdata/manifests");
  let yaml = manifest_dir.join(name);

  Ok(wick_config::config::WickConfiguration::fetch(yaml.to_string_lossy(), Default::default()).await?)
}

pub(crate) async fn load_example(name: &str) -> Result<WickConfiguration> {
  let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let manifest_dir = crate_dir.join("../../../examples");
  let yaml = manifest_dir.join(name);

  Ok(wick_config::config::WickConfiguration::fetch(yaml.to_string_lossy(), Default::default()).await?)
}
