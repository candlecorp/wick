use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use wick_config::config::{AppConfiguration, ComponentConfiguration};

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct InitCommand {
  #[clap(flatten)]
  pub(crate) logging: super::LoggingOptions,

  /// Name of the project.
  #[clap()]
  name: String,

  /// Initialize a new component project.
  #[clap(short = 'c', long = "component", action)]
  component: bool,
}

#[allow(clippy::field_reassign_with_default)]
pub(crate) async fn handle_command(mut opts: InitCommand) -> Result<()> {
  let logging = &mut opts.logging;
  let _guard = logger::init(&logging.name(crate::BIN_NAME));

  let files: Vec<(PathBuf, String)> = if opts.component {
    info!("Initializing wick component project: {}", opts.name);
    let mut config = ComponentConfiguration::default();
    config.name = Some(opts.name);
    vec![("component.yaml".into(), config.into_v1_yaml()?)]
  } else {
    info!("Initializing wick application: {}", opts.name);
    let mut config = AppConfiguration::default();
    config.name = opts.name;
    vec![("app.yaml".into(), config.into_v1_yaml()?)]
  };

  for (file, _) in &files {
    if file.exists() {
      anyhow::bail!("File already exists: {}", file.display());
    }
  }

  for (file, contents) in files {
    info!("Writing file: {}", file.display());
    crate::io::write_bytes(file, contents.as_bytes()).await?;
  }

  Ok(())
}
