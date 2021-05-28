use crate::{util::load_runconfig, Result};
use anyhow::Context;
use std::{collections::HashMap, io::Read, path::PathBuf};
use structopt::StructOpt;
use wasmcloud_host::HostBuilder;

use crate::commands::init_logger;

use super::LoggingOpts;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct RunCommand {
    #[structopt(flatten)]
    pub logging: LoggingOpts,

    /// Turn on info logging
    #[structopt(long = "info")]
    pub info: bool,

    /// JWT file for RPC authentication. Must be supplied with rpc_seed.
    #[structopt(long = "rpc-jwt", env = "VINO_RPC_JWT", hide_env_values = true)]
    pub rpc_jwt: Option<String>,

    /// Seed file or literal for RPC authentication. Must be supplied with rpc_jwt.
    #[structopt(long = "rpc-seed", env = "VINO_RPC_SEED", hide_env_values = true)]
    pub rpc_seed: Option<String>,

    /// JWT file for control interface authentication. Must be supplied with control_seed.
    #[structopt(long = "control-jwt", env = "VINO_CONTROL_JWT", hide_env_values = true)]
    pub control_jwt: Option<String>,

    /// Seed file or literal for control interface authentication. Must be supplied with control_jwt.
    #[structopt(
        long = "control-seed",
        env = "VINO_CONTROL_SEED",
        hide_env_values = true
    )]
    pub control_seed: Option<String>,

    /// Manifest file
    manifest: PathBuf,

    /// JSON data
    data: Option<String>,
}

pub async fn handle_command(command: RunCommand) -> Result<String> {
    let mut logging = command.logging.clone();
    if !(command.info || command.logging.trace || command.logging.debug) {
        logging.quiet = true;
    }
    init_logger(&logging)?;

    let data = match command.data {
        None => {
            eprintln!("No input passed, reading from <STDIN>");
            let mut data = String::new();
            std::io::stdin().read_to_string(&mut data)?;
            data
        }
        Some(i) => i,
    };

    let host_builder = HostBuilder::new();

    let host = host_builder.build();

    let json: HashMap<String, serde_json::value::Value> =
        serde_json::from_str(&data).context("Could not deserialized JSON input data")?;

    let config = load_runconfig(command.manifest)?;

    let result = crate::run(config, json).await?;

    debug!("Raw result: {:?}", result);

    println!("{}", result);

    host.stop().await;

    Ok("Done".to_string())
}
