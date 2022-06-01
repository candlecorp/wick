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
  println!("cloning: {}", opts.project);

  let args = cargo_generate::Args {
    allow_commands: false,
    list_favorites: false,
    favorite: Some(opts.project.clone()),
    subfolder: None,
    git: None,
    path: None,
    branch: None,
    name: Some(opts.name),
    force: false,
    verbose: false,
    template_values_file: None,
    silent: false,
    config: None,
    vcs: cargo_generate::Vcs::Git,
    lib: false,
    bin: false,
    ssh_identity: None,
    define: vec![],
    init: false,
    force_git_init: false,
  };

  cargo_generate::generate(args).unwrap();
  Ok(())
}
