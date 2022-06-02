use std::time::{SystemTime, UNIX_EPOCH};

use runtime_testutils::*;
use tracing::debug;
type Result<T> = anyhow::Result<T, anyhow::Error>;
use pretty_assertions::assert_eq;

#[test_logger::test(tokio::test)]
async fn basic_main_impl() -> Result<()> {
  let argv = vec!["test_file.txt".to_owned()];

  let tempdir = std::env::temp_dir();
  let tempfile = tempdir.join("test_file.txt");
  let now = SystemTime::now();
  let time = now.duration_since(UNIX_EPOCH).unwrap().as_millis().to_string();
  debug!("Writing '{}' to test file {:?}", time, tempfile);
  std::fs::write(&tempfile, &time).unwrap();
  std::env::set_var("TEST_TEMPDIR", tempdir);

  let (network, _) = init_network_from_yaml("./manifests/v0/main.yaml").await?;
  let result = network.exec_main(argv).await;

  std::env::remove_var("TEST_TEMPDIR");

  assert_eq!(result, 0);

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn main_impl_with_network_call() -> Result<()> {
  let argv = vec!["test_file.txt".to_owned()];

  let tempdir = std::env::temp_dir();
  let tempfile = tempdir.join("test_file.txt");
  let now = SystemTime::now();
  let time = now.duration_since(UNIX_EPOCH).unwrap().as_millis().to_string();
  debug!("Writing '{}' to test file {:?}", time, tempfile);
  std::fs::write(&tempfile, &time).unwrap();
  std::env::set_var("TEST_TEMPDIR", tempdir);

  let (network, _) = init_network_from_yaml("./manifests/v0/main-with-network.yaml").await?;
  let result = network.exec_main(argv).await;

  std::env::remove_var("TEST_TEMPDIR");

  assert_eq!(result, 0);

  Ok(())
}
