use std::collections::HashSet;
use std::path::{Path, PathBuf};

use derive_builder::Builder;

use super::dependency::Dependency;

#[derive(Debug, Default, Builder, Clone)]
#[must_use]
pub struct Config {
  #[builder(setter(into))]
  #[builder(default = "std::env::var(\"OUT_DIR\").map(Into::into).unwrap_or_default()")]
  pub(crate) out_dir: PathBuf,
  #[builder(default)]
  pub(crate) raw: bool,
  #[builder(setter(into))]
  pub(crate) spec: PathBuf,
  #[builder(default = "true")]
  pub(crate) op_traits: bool,
  #[builder(default = "true")]
  pub(crate) components: bool,
  #[builder(default = "true")]
  pub(crate) output_structs: bool,
  #[builder(setter(skip))]
  pub(crate) deps: HashSet<Dependency>,
}

impl Config {
  pub fn new() -> Self {
    Self::default()
  }

  pub(crate) fn add_dep(&mut self, dep: Dependency) {
    self.deps.insert(dep);
  }

  pub fn exec(self) -> anyhow::Result<()> {
    super::build(self)?;
    Ok(())
  }
}

impl ConfigBuilder {
  pub fn new() -> Self {
    Self::default()
  }
  pub fn generate(&mut self, spec: impl AsRef<Path>) -> anyhow::Result<()> {
    let config = self.spec(spec.as_ref().to_path_buf()).build()?;
    super::build(config)?;

    Ok(())
  }
}

#[must_use]
pub fn configure() -> ConfigBuilder {
  ConfigBuilder::default()
}
