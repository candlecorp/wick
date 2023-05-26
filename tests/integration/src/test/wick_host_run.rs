mod slow_test {

  use std::path::PathBuf;
  use std::time::Duration;

  use anyhow::Result;
  use serde_json::{json, Value};
  use tracing::debug;
  use wick_config::WickConfiguration;
  use wick_host::AppHostBuilder;

  #[test_logger::test(tokio::test)]
  async fn test_wick_run() -> Result<()> {
    let manifest = "testdata/manifests/app-http-component.wick";
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
      .join("..")
      .join(manifest);

    let app_config = WickConfiguration::fetch_all(manifest.to_string_lossy(), Default::default())
      .await?
      .try_app_config()?;

    let mut host = AppHostBuilder::default().manifest(app_config.clone()).build()?;
    host.start(None)?;
    debug!("Waiting on triggers to finish or interrupt...");
    // host.wait_for_done().await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    let client = reqwest::Client::new();
    let res = client
      .post("http://localhost:8080/v1/")
      .json(&json!({"token":"this is my secret token"}))
      .send()
      .await?;
    let result = res.bytes().await?;
    let result = String::from_utf8(result.to_vec())?;
    println!("result: {}", result);
    let result: serde_json::Map<String, Value> = serde_json::from_str(&result)?;
    let args = result.get("args");

    assert_eq!(
      args,
      Some(&json!({"secret":"foobar","token":"this is my secret token"}))
    );

    Ok(())
  }
}
