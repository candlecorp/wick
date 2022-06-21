use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, BufReader};

#[test_log::test(tokio::test)]

async fn basic_usage() -> Result<(), std::io::Error> {
  let mut child = tokio_test_bin::get_test_bin("test_bin")
    .stdout(Stdio::piped())
    .spawn()?;

  let stdout = child.stdout.take().unwrap();

  let mut reader = BufReader::new(stdout).lines();

  let line = reader.next_line().await?.unwrap();

  assert_eq!(line, "Output from my CLI app!");
  Ok(())
}
