mod utils;
use std::path::PathBuf;

use anyhow::Result;
use tokio_stream::StreamExt;
use tracing::Span;
use wick_host::{AppHost, AppHostBuilder, Host};
use wick_packet::{packets, Entity, InherentData, Invocation, Packet};

#[test_logger::test(tokio::test)]
async fn test_deep_invoke() -> Result<()> {
  let app_config = utils::load_root_test_config("run/unit/file-reader.wick", None).await?;
  let rt = AppHost::build_runtime(&app_config, Some(1), Span::current()).await?;
  let app_host = AppHostBuilder::default().manifest(app_config).runtime(rt).build()?;
  let target = Entity::operation("wasi_fs", "read_string");
  let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
  let invocation = Invocation::new(
    Entity::test("test_deep_invoke"),
    target,
    packets!(("filename", "Cargo.toml")),
    InherentData::unsafe_default(),
    &Span::current(),
  );
  let stream = app_host.invoke_deep(Some(&["CLI"]), invocation, None).await?;

  let mut packets: Vec<_> = stream.collect().await;

  assert_eq!(packets.len(), 2);
  packets.pop();
  let output = packets.pop().unwrap().unwrap();

  assert_eq!(output, Packet::encode("output", std::fs::read_to_string(file)?));

  Ok(())
}
