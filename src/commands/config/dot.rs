use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;
use wick_config::WickConfiguration;
use wick_host::ComponentHostBuilder;

use crate::utils::{get_auth_for_scope, merge_config, parse_config_string};

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) oci: crate::options::oci::OciOptions,

  #[clap(flatten)]
  pub(crate) component: crate::options::component::ComponentOptions,
  /// Operation to render.
  #[clap(action)]
  pub(crate) operation: String,

  /// Pass configuration necessary to invoke the operation (JSON).
  #[clap(long = "op-with", action)]
  op_with: Option<String>,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(
  opts: Options,
  settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  span.in_scope(|| debug!("generate dotviz graph"));
  let configured_creds = settings
    .credentials
    .iter()
    .find(|c| opts.component.path.starts_with(&c.scope));

  let (username, password) = get_auth_for_scope(
    configured_creds,
    opts.oci.username.as_deref(),
    opts.oci.password.as_deref(),
  );
  let env = wick_xdg::Settings::new();

  let mut fetch_opts: wick_oci_utils::OciOptions = opts.oci.clone().into();
  fetch_opts.set_username(username).set_password(password);

  let path = PathBuf::from(&opts.component.path);

  if !path.exists() {
    fetch_opts.set_cache_dir(env.global().cache().clone());
  } else {
    let mut path_dir = path.clone();
    path_dir.pop();
    fetch_opts.set_cache_dir(path_dir.join(env.local().cache()));
  };

  let root_config = parse_config_string(opts.component.with.as_deref())?;

  let mut config = WickConfiguration::fetch(&opts.component.path, fetch_opts).await?;
  config.set_root_config(root_config);
  let manifest = config.finish()?.try_component_config()?;

  let manifest = merge_config(&manifest, &opts.oci, None);

  let mut host = ComponentHostBuilder::default().manifest(manifest).span(span).build()?;

  host.start_runtime(None).await?;
  let dotviz = host.render_dotviz(&opts.operation)?;
  let json = json!({"dotviz": dotviz});
  Ok(StructuredOutput::new(dotviz, json))
}
