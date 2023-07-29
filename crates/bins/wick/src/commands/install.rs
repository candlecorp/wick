use anyhow::Result;
use clap::Args;
use structured_output::StructuredOutput;
use wick_config::WickConfiguration;

use crate::options::reconcile_fetch_options;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) oci: crate::oci::Options,

  /// Path or OCI url to manifest or wasm file.
  #[clap(action)]
  path: String,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: Options,
  settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let xdg = wick_xdg::Settings::new();
  let bin_dir = xdg.global().root().join("bin");

  span.in_scope(|| info!(path = opts.path, to = %bin_dir.display(), "installing wick app"));

  let oci_opts = reconcile_fetch_options(&opts.path, &settings, opts.oci, false, None);
  let package = crate::oci::pull(opts.path, oci_opts).await?;
  let path = package.path();
  let config = WickConfiguration::fetch(path.to_string_lossy(), Default::default())
    .await?
    .into_inner();

  let config = match config {
    WickConfiguration::App(config) => config,
    _ => anyhow::bail!("{} is not an application configuration", path.display()),
  };

  let bin_path = bin_dir.join(config.name());

  #[cfg(not(target_os = "windows"))]
  {
    let sh = format!(
      r#"
    #!/bin/sh

    wick run {}
    "#,
      path.display()
    )
    .replace("    ", "");

    use std::os::unix::fs::PermissionsExt;

    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o755);

    std::fs::write(&bin_path, sh)?;

    std::fs::set_permissions(&bin_path, perms)?;
  }
  #[cfg(target_os = "windows")]
  {
    let link = mslnk::ShellLink::new(path)?;
    link.create_lnk(bin_path)?;
  }

  let text = format!("installed {} to {}", config.name(), bin_path.display());
  let json = serde_json::json!({
    "name": config.name(),
    "path": bin_path,
  });

  let output = StructuredOutput::new(text, json);

  Ok(output)
}
