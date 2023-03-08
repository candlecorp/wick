use anyhow::Result;
use clap::Args;
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct ProjectNewCommand {
  #[clap(flatten)]
  pub(crate) logging: logger::LoggingOptions,

  /// The name of the directory to start a new project in.
  #[clap(action)]
  name: String,

  /// The language (or git URL to clone).
  #[clap(action)]
  language: String,
}

#[derive(clap::ValueEnum, Debug, Clone)]
enum LanguageOptions {
  Rust,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: ProjectNewCommand) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;

  let url = match opts.language.as_str() {
    "rust" => "https://github.com/wasmflow/rust-component-boilerplate.git",
    x => x,
  };

  let git_dir = format!("{}/.git", opts.name);
  info!("Cloning {} into {}", url, opts.name);
  crate::git_template::pull_into_dir(url, opts.name)?;
  std::fs::remove_dir_all(git_dir)?;

  Ok(())
}
