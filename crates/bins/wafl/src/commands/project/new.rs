use anyhow::Result;
use clap::Args;
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) logging: logger::LoggingOptions,

  /// Name of the component to execute.
  project: String,

  /// Name of the component to execute.
  name: String,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: Options) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;

  crate::git_template::pull_into_dir(opts.project, opts.name)?;

  Ok(())
}
