pub mod commands;
pub mod error;
// pub(crate) mod logger;
pub mod utils;

use std::collections::HashMap;

use error::VinoError;
use serde_json::json;
use serde_json::Value::String as JsonString;
use vino_codec::messagepack::deserialize;
use vino_host::{
  HostBuilder,
  HostDefinition,
};
use vino_transport::MessageTransport;

pub type Result<T> = std::result::Result<T, VinoError>;
pub type Error = VinoError;

#[macro_use]
extern crate log;

pub type JsonMap = HashMap<String, serde_json::value::Value>;

pub async fn run(manifest: HostDefinition, data: JsonMap) -> Result<serde_json::Value> {
  let host_builder = HostBuilder::new();

  let mut host = host_builder.build();

  debug!("Starting host");

  host.start().await?;

  host.start_network(manifest.network).await?;

  info!("Manifest applied");

  let raw_result = host.request(&manifest.default_schematic, data).await?;

  debug!("Raw result: {:?}", raw_result);

  let result: serde_json::Value = raw_result
    .iter()
    .map(|(k, payload)| {
      (
        k.to_string(),
        match payload {
          MessageTransport::MessagePack(bytes) => deserialize(bytes).unwrap_or_else(|e| {
            JsonString(format!(
              "Error deserializing output payload: {}",
              e.to_string(),
            ))
          }),
          MessageTransport::Exception(e) => json!({ "exception": e }),
          MessageTransport::Error(e) => json!({ "error": e }),
          _ => json!({ "error": "Internal error, invalid format" }),
        },
      )
    })
    .collect();

  host.stop().await;

  Ok(result)
}

#[cfg(test)]
mod tests {

  use std::path::PathBuf;

  use maplit::hashmap;

  #[actix_rt::test]
  async fn runs_log_config() -> crate::Result<()> {
    let host_def =
      vino_host::HostDefinition::load_from_file(&PathBuf::from("./manifests/log.vino"))?;
    let input = hashmap! {
      "input".into() => "test-input".into()
    };

    let result = crate::run(host_def, input).await?;
    assert_eq!(result.get("output").unwrap(), "test-input");
    Ok(())
  }
}
