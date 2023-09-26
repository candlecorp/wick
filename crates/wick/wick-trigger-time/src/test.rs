use anyhow::Result;
use wick_config::WickConfiguration;

pub(crate) async fn load_example(name: &str) -> Result<WickConfiguration> {
  let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let manifest_dir = crate_dir.join("../../../examples");
  let yaml = manifest_dir.join(name);
  let mut config = wick_config::config::WickConfiguration::fetch(&yaml, Default::default()).await?;
  config.set_env(Some(std::env::vars().collect()));

  Ok(config.finish()?)
}
