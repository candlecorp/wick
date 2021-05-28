use crate::{util::fetch_actor, Result};
use anyhow::Context;
use serde_json::{json, Value::String as JsonString};
use std::collections::HashMap;
use structopt::StructOpt;
use wasmcloud_host::{deserialize, HostBuilder, MessagePayload};

use crate::{commands::init_logger, util::generate_exec_manifest};

use super::LoggingOpts;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct ExecCommand {
    #[structopt(flatten)]
    pub logging: LoggingOpts,

    /// Turn on info logging
    #[structopt(long = "info")]
    pub info: bool,

    /// Allows the use of HTTP registry connections to these registries
    #[structopt(long = "allowed-insecure")]
    pub allowed_insecure: Vec<String>,

    /// Input filename or URL
    #[structopt()]
    actor_ref: String,

    /// JSON data
    #[structopt(default_value = "\"\"")]
    data: String,
}

pub async fn handle_command(command: ExecCommand) -> Result<String> {
    let mut logging = command.logging.clone();
    if !(command.info || command.logging.trace || command.logging.debug) {
        logging.quiet = true;
    }
    init_logger(&logging)?;

    let mut host_builder = HostBuilder::new();
    host_builder = host_builder.oci_allow_latest();

    if !command.allowed_insecure.is_empty() {
        host_builder = host_builder.oci_allow_insecure(command.allowed_insecure.clone());
    }

    let host = host_builder.build();
    let actor_ref = command.actor_ref.to_string();

    let actor = fetch_actor(actor_ref.to_string(), true, command.allowed_insecure).await?;

    let json_string = command.data;
    let json: HashMap<String, serde_json::value::Value> =
        serde_json::from_str(&json_string).context("Could not deserialized JSON input data")?;

    info!("Starting host");
    match host.start().await {
        Ok(_) => {
            info!("Loading dynamic manifest");
            let hm = generate_exec_manifest(actor_ref, actor, &json)?;
            host.apply_manifest(hm).await?;
            debug!("Manifest applied, executing component");
            let raw_result = host.request("dynamic".to_string(), json).await?;
            debug!("Raw result: {:?}", raw_result);
            // let msg_result: HashMap<String, Vec<u8>> = deserialize(&raw_result)?;
            let result: serde_json::Value = raw_result
                .iter()
                .map(|(k, payload)| {
                    (
                        k,
                        match payload {
                            MessagePayload::Bytes(bytes) => {
                                deserialize(&bytes).unwrap_or_else(|e| {
                                    JsonString(format!(
                                        "Error deserializing output payload: {}",
                                        e.to_string(),
                                    ))
                                })
                            }
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
            println!("{}", result);
            info!("Done");
            host.stop().await;
        }
        Err(e) => {
            error!("Failed to start host: {}", e);
        }
    }
    Ok("Done".to_string())
}
