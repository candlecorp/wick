use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) logging: logger::LoggingOptions,

  /// Name of the new component.
  #[clap(action)]
  name: String,

  /// Directory to save the schema into.
  #[clap(action, default_value = "schemas")]
  path: String,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: Options) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;

  let path = PathBuf::from(format!("{}/{}.apex", opts.path, opts.name));

  let apex = format!(
    r#"
namespace "{}"

type Inputs {{

}}

type Outputs {{

}}

type Config {{

}}
"#,
    opts.name
  );

  if path.exists() {
    Err(anyhow!(
      "Refusing to overwrite existing file at {}",
      path.to_string_lossy()
    ))
  } else {
    info!("Creating new schema for {} at {}", opts.name, path.to_string_lossy());
    tokio::fs::write(path, apex).await?;

    Ok(())
  }
}
