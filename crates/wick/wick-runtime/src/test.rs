pub(crate) mod prelude {
  pub(crate) use anyhow::Result;
  pub(crate) use futures::StreamExt;
  pub(crate) use pretty_assertions::assert_eq;

  pub(crate) use super::*;
}

use wick_config::WickConfiguration;

use crate::test::prelude::*;
use crate::{Engine, EngineBuilder};

pub(crate) async fn init_engine_from_yaml(path: &str) -> Result<(Engine, uuid::Uuid)> {
  let def = WickConfiguration::load_from_file(path).await?.try_component_config()?;

  let engine = EngineBuilder::from_definition(def)?.build().await?;

  let engine_id = engine.uid;
  trace!(engine_id = %engine_id, "engine uid");
  Ok((engine, engine_id))
}
