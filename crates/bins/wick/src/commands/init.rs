use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

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

  let (file, contents) = if opts.component {
    let mut config = wick_config::ComponentConfiguration::default();
    config.name = Some(opts.name);
    ("component.yaml", serde_yaml::to_string(&config)?)
  } else {
    let mut config = wick_config::AppConfiguration::default();
    config.name = opts.name;
    ("app.yaml", serde_yaml::to_string(&config)?)
  };

  let dir = std::env::current_dir()?;
  let currdir_files: Vec<_> = std::fs::read_dir(dir)?.collect();

  if !currdir_files.is_empty() {
    anyhow::bail!("Current directory is not empty. Aborting.");
  }

  let path = PathBuf::from(file);
  if path.exists() {
    anyhow::bail!("File already exists: {}", file);
  }

  crate::io::write_bytes(path, contents.as_bytes()).await?;

  Ok(())
}
