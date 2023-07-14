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
  let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

  let def = WickConfiguration::load_from_file(crate_dir.join("tests").join(path))
    .await?
    .finish()?
    .try_component_config()?;

  let engine = RuntimeBuilder::from_definition(def).build(None).await?;

  let engine_id = engine.uid;
  trace!(engine_id = %engine_id, "engine uid");
  Ok((engine, engine_id))
}

pub(crate) async fn load_test_manifest(name: &str) -> Result<WickConfiguration> {
  let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let manifest_dir = crate_dir.join("tests/manifests/v1");
  let yaml = manifest_dir.join(name);
  let mut config = wick_config::config::WickConfiguration::fetch(yaml.to_string_lossy(), Default::default()).await?;

  config.set_env(Some(std::env::vars().collect()));

  Ok(config.finish()?)
}

pub(crate) async fn load_example(name: &str) -> Result<WickConfiguration> {
  let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let manifest_dir = crate_dir.join("../../../examples");
  let yaml = manifest_dir.join(name);
  let mut config = wick_config::config::WickConfiguration::fetch(yaml.to_string_lossy(), Default::default()).await?;
  config.set_env(Some(std::env::vars().collect()));

  Ok(config.finish()?)
}
