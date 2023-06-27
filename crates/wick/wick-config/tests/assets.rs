mod integration_test {
  use std::path::PathBuf;

  use anyhow::Result;
  use asset_container::{Asset, AssetFlags, AssetManager, Status};
  use tokio_stream::StreamExt;
  use wick_config::config::FetchOptions;
  use wick_config::error::ManifestError;
  use wick_config::*;

  async fn load(path: &str) -> Result<WickConfiguration, ManifestError> {
    let path = PathBuf::from(path);
    WickConfiguration::load_from_file(path).await?.finish()
  }

  #[test_logger::test(tokio::test)]
  #[ignore = "fetch with progress has been removed as unused for now"]
  async fn test_fetch_with_progress() -> Result<()> {
    let config = load("./tests/manifests/v1/logger.yaml").await?;
    let mut assets = config.assets();
    assert_eq!(assets.len(), 2);
    let mut progress = assets.pull_with_progress(FetchOptions::default());
    let mut num_progress = 0;
    let mut num_error = 0;
    let mut asset_done = 0;
    while let Some(progress) = progress.next().await {
      num_progress += 1;
      for progress in progress {
        match progress.status {
          Status::AssetComplete(_) => {
            asset_done += 1;
          }
          Status::PullFinished => {}
          Status::Progress { .. } => {}
          Status::Error(_e) => {
            num_error += 1;
          }
        }
      }
    }
    assert_eq!(num_error, 2);
    assert_eq!(asset_done, 0);
    assert!(num_progress > 0);

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_app_assets() -> Result<()> {
    let opts = FetchOptions::default();
    let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let asset_dir = crate_dir.join("tests/assets/test-application/");

    let config = WickConfiguration::fetch("./tests/assets/test-application/app.wick", opts.clone())
      .await?
      .finish()?;
    for asset in config.assets().iter() {
      if asset.is_directory() {
        continue;
      }
      let bytes = asset.fetch(opts.clone()).await?;
      let expected_bytes = tokio::fs::read(asset_dir.join(asset.location())).await?;
      assert_eq!(bytes, expected_bytes);
    }

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_lazy_assets() -> Result<()> {
    let opts = FetchOptions::default();

    let config = WickConfiguration::fetch("./tests/assets/test-application/app.wick", opts.clone())
      .await?
      .finish()?
      .try_app_config()?;
    let pkg_files = config.package_files();
    let num_expected = pkg_files.iter().count();
    let mut total = 0;
    let mut count_lazy = 0;
    let mut non_lazy = 0;
    for asset in config.assets().iter() {
      total += 1;
      if asset.get_asset_flags() == AssetFlags::Lazy {
        count_lazy += 1;
      } else {
        non_lazy += 1;
      }
    }
    assert_eq!(count_lazy, num_expected);
    assert_eq!(non_lazy, total - num_expected);

    Ok(())
  }
}
