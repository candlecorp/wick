use crate::Result;
use anyhow::Context;
use std::{collections::HashMap, path::Path};
use structopt::StructOpt;
use wasmcloud_host::{deserialize, Actor, HostBuilder};

use crate::{commands::init_logger, oci::fetch_oci_bytes, util::generate_run_manifest};

use super::LoggingOpts;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct RunCli {
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

pub(crate) async fn handle_command(command: RunCli) -> Result<String> {
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
            let hm = generate_run_manifest(actor_ref, actor, &json)?;
            host.apply_manifest(hm).await?;
            debug!("Manifest applied, executing component");
            let raw_result = host.request("dynamic", json).await?;
            debug!("Raw result: {:?}", raw_result);
            let msg_result: HashMap<String, Vec<u8>> = deserialize(&raw_result)?;
            let result: serde_json::Value = msg_result
                .iter()
                .map(|(k, v)| {
                    (
                        k.to_string(),
                        deserialize(&v).unwrap_or_else(|e| {
                            serde_json::Value::String(format!(
                                "Error deserializing output for port {}: {}",
                                k,
                                e.to_string()
                            ))
                        }),
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

async fn fetch_actor(
    actor_ref: String,
    allow_latest: bool,
    allowed_insecure: Vec<String>,
) -> Result<Actor> {
    let p = Path::new(&actor_ref);
    if p.exists() {
        Ok(wasmcloud_host::Actor::from_file(p)?)
    } else {
        let actor_bytes = fetch_oci_bytes(&actor_ref, allow_latest, &allowed_insecure).await?;
        Ok(wasmcloud_host::Actor::from_slice(&actor_bytes)?)
    }
}
