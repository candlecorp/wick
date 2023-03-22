use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
#[must_use]
pub struct Config {
  pub(crate) out_dir: PathBuf,
  pub(crate) raw: bool,
  pub(crate) spec: PathBuf,
}

impl Config {
  pub fn new() -> Self {
    Self::default()
  }

  /// Set the output directory to generate code to.
  ///
  /// Defaults to the `OUT_DIR` environment variable.
  pub fn out_dir(mut self, out_dir: impl AsRef<Path>) -> Self {
    self.out_dir = out_dir.as_ref().to_path_buf();
    self
  }

  /// Generates code that does not automatically deserialize packets.
  ///

  pub fn raw(mut self, value: bool) -> Self {
    self.raw = value;
    self
  }

  pub fn spec(mut self, spec: impl AsRef<Path>) -> Self {
    self.spec = spec.as_ref().to_path_buf();
    self
  }
}
