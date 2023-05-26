use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use wick_config::config::{AppConfiguration, ComponentConfiguration};

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct InitCommand {
  /// Name of the project.
  #[clap()]
  name: String,

  /// Initialize a new component project.
  #[clap(short = 'c', long = "component", action)]
  component: bool,
}

#[allow(clippy::field_reassign_with_default)]
pub(crate) async fn handle(opts: InitCommand, _settings: wick_settings::Settings, span: tracing::Span) -> Result<()> {
  let files: Result<Vec<(PathBuf, String)>> = span.in_scope(|| {
    if opts.component {
      info!("Initializing wick component project: {}", opts.name);
      let mut config = ComponentConfiguration::default();
      config.set_name(opts.name);
      Ok(vec![("component.wick".into(), config.into_v1_yaml()?)])
    } else {
      info!("Initializing wick application: {}", opts.name);
      let mut config = AppConfiguration::default();
      config.set_name(opts.name);
      Ok(vec![("app.wick".into(), config.into_v1_yaml()?)])
    }
  });
  let files = files?;

  for (file, _) in &files {
    if file.exists() {
      anyhow::bail!("File already exists: {}", file.display());
    }
  }

  for (file, contents) in files {
    span.in_scope(|| info!("Writing file: {}", file.display()));

    crate::io::write_bytes(file, contents.as_bytes()).await?;
  }

  Ok(())
}
