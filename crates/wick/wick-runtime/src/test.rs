pub(crate) mod prelude {
  pub(crate) use anyhow::Result;
  pub(crate) use futures::StreamExt;
  pub(crate) use pretty_assertions::assert_eq;

  pub(crate) use super::*;
}

use wick_config::WickConfiguration;

use crate::test::prelude::*;
use crate::{Runtime, RuntimeBuilder};

pub(crate) async fn init_scope_from_yaml(path: &str) -> Result<(Runtime, uuid::Uuid)> {
  let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

  let def = WickConfiguration::fetch(&crate_dir.join("tests").join(path), Default::default())
    .await?
    .finish()?
    .try_component_config()?;

  let scope = RuntimeBuilder::from_definition(def).build(None).await?;

  let scope_id = scope.uid;
  trace!(scope_id = %scope_id, "scope id");
  Ok((scope, scope_id))
}
