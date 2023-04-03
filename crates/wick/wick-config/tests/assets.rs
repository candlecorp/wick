use std::path::PathBuf;

use assets::{AssetManager, Status};
use tokio_stream::StreamExt;
use wick_config::config::FetchOptions;
use wick_config::error::ManifestError;
use wick_config::*;

async fn load(path: &str) -> Result<WickConfiguration, ManifestError> {
  let path = PathBuf::from(path);
  WickConfiguration::load_from_file(path).await
}

#[test_logger::test(tokio::test)]
async fn test_basics() -> Result<(), ManifestError> {
  let config = load("./tests/manifests/v1/logger.yaml").await?;
  let assets = config.assets();
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
