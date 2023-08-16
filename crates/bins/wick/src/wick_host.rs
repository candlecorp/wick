use std::path::PathBuf;

use anyhow::Result;
use seeded_random::Seed;
use tracing::Span;
use wick_component_cli::options::DefaultCliOptions;
use wick_config::WickConfiguration;
use wick_host::{ComponentHost, ComponentHostBuilder};
use wick_packet::RuntimeConfig;

use crate::options::oci::OciOptions as WickOciOptions;
use crate::utils::{get_auth_for_scope, merge_config};

pub(crate) async fn build_component_host(
  path: &str,
  oci: WickOciOptions,
  root_config: Option<RuntimeConfig>,
  settings: wick_settings::Settings,
  seed: Option<u64>,
  server_settings: Option<DefaultCliOptions>,
  span: Span,
) -> Result<ComponentHost> {
  let configured_creds = settings.credentials.iter().find(|c| path.starts_with(&c.scope));

  let (username, password) = get_auth_for_scope(configured_creds, oci.username.as_deref(), oci.password.as_deref());
  let env = wick_xdg::Settings::new();

  let mut fetch_opts: wick_oci_utils::OciOptions = oci.clone().into();
  fetch_opts.set_username(username).set_password(password);

  let pb = PathBuf::from(path);

  if !pb.exists() {
    fetch_opts.set_cache_dir(env.global().cache().clone());
  } else {
    let mut path_dir = pb.clone();
    path_dir.pop();
    fetch_opts.set_cache_dir(path_dir.join(env.local().cache()));
  };

  let mut manifest = WickConfiguration::fetch(path, fetch_opts).await?;
  manifest.set_root_config(root_config);
  let manifest = manifest.finish()?.try_component_config()?;

  let manifest = merge_config(&manifest, &oci, server_settings);

  let mut host = ComponentHostBuilder::default()
    .id(manifest.name().map_or_else(|| "component".to_owned(), |s| s.clone()))
    .manifest(manifest)
    .span(span)
    .build()?;

  host.start_runtime(seed.map(Seed::unsafe_new)).await?;

  Ok(host)
}
