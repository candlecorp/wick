mod integration_test {
  use std::path::PathBuf;

  use anyhow::Result;
  use asset_container::{Asset, AssetManager, Status};
  use tokio_stream::StreamExt;
  use wick_config::config::FetchOptions;
  use wick_config::error::ManifestError;
  use wick_config::*;

  async fn load(path: &str) -> Result<WickConfiguration, ManifestError> {
    let path = PathBuf::from(path);
    WickConfiguration::load_from_file(path).await
  }

  #[test_logger::test(tokio::test)]
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
  async fn test_package_assets() -> Result<()> {
    let opts = FetchOptions::default();
    let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let asset_dir = crate_dir.join("tests/assets/test-application/");

    let config = WickConfiguration::fetch("./tests/assets/test-application/app.wick", opts.clone()).await?;
    for asset in config.assets().iter() {
      let bytes = asset.fetch(opts.clone()).await?;
      let expected_bytes = tokio::fs::read(asset_dir.join(asset.location())).await?;
      assert_eq!(bytes, expected_bytes);
    }

    Ok(())
  }

  // TODO: move to wick-package
  // Commenting out to remove dependency on wick-package for now.
  // fn get_relative_path(path: &str) -> PathBuf {
  //   let mut root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
  //   root.push("tests");
  //   root.push(path);
  //   root
  // }

  // #[test_logger::test(tokio::test)]
  // async fn test_remote_asset_fetch() -> Result<()> {
  //   // Setup: push test-component to local registry
  //   let host = std::env::var("DOCKER_REGISTRY").unwrap();
  //   let reg_host = host.split(':').next().unwrap();
  //   let options = wick_oci_utils::OciOptions::default().allow_insecure(vec![host.to_owned()]);
  //   let test_component = get_relative_path("./assets/test-component/component.wick");

  //   let mut package = wick_package::WickPackage::from_path(&test_component).await?;
  //   let result = package
  //     .push(&format!("{}/test-component/jinja:0.2.0", host), &options)
  //     .await?;
  //   println!("result: {:?}", result);

  //   // Test: Assert that an app config that references the test-component can be loaded

  //   let config = load("./tests/assets/test-application/app.wick").await?;
  //   let assets = config.assets();

  //   // Create a temp directory
  //   let mut basedir = std::env::temp_dir();
  //   println!("basedir: {}", basedir.display());

  //   let options = FetchOptions::default()
  //     .allow_insecure([host.to_owned()])
  //     .artifact_dir(basedir.clone());

  //   basedir.push(wick_xdg::Cache::Assets.basedir());
  //   // Clean up the cache in the temp directory before running test
  //   let _ = tokio::fs::remove_dir_all(&basedir).await;

  //   let _progress = assets.pull(options).await?;

  //   let first = basedir.join(format!("{}/test-component/jinja/0.2.0/component.wick", reg_host));
  //   println!("first: {}", first.display());
  //   assert!(first.exists());
  //   let second = basedir.join(format!("{}/test-component/jinja/0.2.0/assets/test.fake.wasm", reg_host));
  //   println!("second: {}", second.display());
  //   assert!(second.exists());

  //   Ok(())
  // }
}
