pub mod commands;
pub mod error;
pub(crate) mod logger;
pub(crate) mod oci;
pub mod util;

use std::collections::HashMap;

use error::VinoError;
use serde_json::{json, Value::String as JsonString};
use vino_runtime::run_config::RunConfig;
use wasmcloud_host::{deserialize, HostBuilder, MessagePayload};

pub type Result<T> = anyhow::Result<T, VinoError>;
pub type Error = VinoError;

#[macro_use]
extern crate log;

pub type JsonMap = HashMap<String, serde_json::value::Value>;

pub async fn run(manifest: RunConfig, data: JsonMap) -> Result<serde_json::Value> {
    let host_builder = HostBuilder::new();

    let host = host_builder.build();

    debug!("Starting host");

    host.start()
        .await
        .map_err(|e| VinoError::HostStartFailure(e.to_string()))?;

    host.apply_manifest(manifest.manifest)
        .await
        .map_err(|e| VinoError::HostStartFailure(e.to_string()))?;

    info!("Manifest applied");

    let raw_result = host
        .request(manifest.default_schematic.to_string(), data)
        .await?;

    debug!("Raw result: {:?}", raw_result);

    let result: serde_json::Value = raw_result
        .iter()
        .map(|(k, payload)| {
            (
                k.to_string(),
                match payload {
                    MessagePayload::Bytes(bytes) => deserialize(&bytes).unwrap_or_else(|e| {
                        JsonString(format!(
                            "Error deserializing output payload: {}",
                            e.to_string(),
                        ))
                    }),
                    MessagePayload::Exception(e) => {
                        json!({ "exception": e })
                    }
                    MessagePayload::Error(e) => {
                        json!({ "error": e })
                    }
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
    use std::collections::HashMap;

    #[actix_rt::test]
    async fn runs_crud_api_config() -> crate::Result<()> {
        let manifest = include_bytes!("../examples/crud-api.vino");
        let config = crate::util::parse_runconfig(String::from_utf8_lossy(manifest).into())?;
        let mut input: HashMap<String, serde_json::Value> = HashMap::new();
        input.insert(
            "content_id".to_string(),
            "1441c9dd-0c4e-46bc-a53d-ae5455968015".into(),
        );
        input.insert("collection_id".to_string(), "someuser_209821".into());

        let result = crate::run(config, input).await?;
        assert_eq!(result.get("document").unwrap(), "6Xf;JunM4$~v@");
        Ok(())
    }
}
