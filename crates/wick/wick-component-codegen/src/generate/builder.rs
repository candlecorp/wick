use std::io::Write;
use std::path::{Path, PathBuf};

pub fn configure() -> Builder {
  Builder::default()
}

#[derive(Debug, Default, Clone)]
#[must_use]
pub struct Builder {
  out_dir: Option<PathBuf>,
  raw: bool,
}

impl Builder {
  pub fn new() -> Self {
    Self::default()
  }

  /// Set the output directory to generate code to.
  ///
  /// Defaults to the `OUT_DIR` environment variable.
  pub fn out_dir(mut self, out_dir: impl AsRef<Path>) -> Self {
    self.out_dir = Some(out_dir.as_ref().to_path_buf());
    self
  }

  /// Generates code that does not automatically deserialize the packet.
  pub fn raw(mut self) -> Self {
    self.raw = true;
    self
  }

  pub fn generate(self, spec: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut config = super::config::Config::new();
    let out_dir = self.out_dir.as_ref().map_or_else(
      || PathBuf::from(std::env::var("OUT_DIR").unwrap()),
      |out_dir| out_dir.clone(),
    );

    let config = config.out_dir(out_dir).raw(self.raw).spec(spec);

    super::build(config)?;

    Ok(())
  }
}
