use std::path::PathBuf;

use crate::{
    util::{load_runconfig, nats_connection},
    Result,
};

use vino_runtime::run_config::RunConfig;

use futures::try_join;
use structopt::StructOpt;
use wasmcloud_host::HostBuilder;

use crate::commands::init_logger;

use super::LoggingOpts;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct StartCommand {
    #[structopt(flatten)]
    pub logging: LoggingOpts,

    /// Host for RPC connection
    #[structopt(long = "rpc-host", default_value = "0.0.0.0", env = "VINO_RPC_HOST")]
    pub rpc_host: String,

    /// Port for RPC connection
    #[structopt(long = "rpc-port", default_value = "4222", env = "VINO_RPC_PORT")]
    pub rpc_port: String,

    /// JWT file for RPC authentication. Must be supplied with rpc_seed.
    #[structopt(long = "rpc-jwt", env = "VINO_RPC_JWT", hide_env_values = true)]
    pub rpc_jwt: Option<String>,

    /// Seed file or literal for RPC authentication. Must be supplied with rpc_jwt.
    #[structopt(long = "rpc-seed", env = "VINO_RPC_SEED", hide_env_values = true)]
    pub rpc_seed: Option<String>,

    /// Credsfile for RPC authentication
    #[structopt(long = "rpc-credsfile", env = "VINO_RPC_CREDS", hide_env_values = true)]
    pub rpc_credsfile: Option<String>,

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

    /// Credsfile for control interface authentication
    #[structopt(
        long = "control-credsfile",
        env = "VINO_CONTROL_CREDS",
        hide_env_values = true
    )]
    pub control_credsfile: Option<String>,

    /// Allows live updating of actors
    #[structopt(long = "allow-live-updates")]
    pub allow_live_updates: bool,

    /// Allows the use of "latest" artifact tag
    #[structopt(long = "allow-oci-latest")]
    pub allow_oci_latest: bool,

    /// Disables strict comparison of live updated actor claims
    #[structopt(long = "disable-strict-update-check")]
    pub disable_strict_update_check: bool,

    /// Allows the use of HTTP registry connections to these registries
    #[structopt(long = "allowed-insecure")]
    pub allowed_insecure: Vec<String>,

    /// Specifies a manifest file to apply to the host once started
    #[structopt(parse(from_os_str))]
    pub manifest: Option<PathBuf>,
}

pub async fn handle_command(command: StartCommand) -> Result<String> {
    init_logger(&command.logging)?;

    let config = match command.manifest {
        Some(file) => load_runconfig(file)?,
        None => RunConfig::default(),
    };

    debug!("Attempting connection to NATS server");
    let nats_url = &format!("{}:{}", command.rpc_host, command.rpc_port);
    let nc_rpc = nats_connection(
        nats_url,
        command.rpc_jwt,
        command.rpc_seed,
        command.rpc_credsfile.or(config.config.rpc_credentials),
    );
    let nc_control = nats_connection(
        nats_url,
        command.control_jwt,
        command.control_seed,
        command
            .control_credsfile
            .or(config.config.control_credentials),
    );

    let mut host_builder = HostBuilder::new();

    match try_join!(nc_rpc, nc_control) {
        Ok((nc_rpc, nc_control)) => {
            host_builder = host_builder
                .with_rpc_client(nc_rpc)
                .with_control_client(nc_control);
        }
        Err(e) => warn!("Could not connect to NATS, operating locally ({})", e),
    }

    if command.allow_oci_latest || config.config.allow_oci_latest {
        debug!("Enabling :latest tag");
        host_builder = host_builder.oci_allow_latest();
    }

    if !command.allowed_insecure.is_empty() {
        host_builder = host_builder.oci_allow_insecure(command.allowed_insecure);
    }

    if !config.config.allowed_insecure.is_empty() {
        host_builder = host_builder.oci_allow_insecure(config.config.allowed_insecure);
    }

    let host = host_builder.build();

    debug!("Starting host");
    match host.start().await {
        Ok(_) => {
            debug!("Applying manifest");
            host.apply_manifest(config.manifest).await?;
            info!("Manifest applied");
        }
        Err(e) => {
            error!("Failed to start host: {}", e);
        }
    }

    actix_rt::signal::ctrl_c().await.unwrap();
    info!("Ctrl-C received, shutting down");
    host.stop().await;
    Ok("Done".to_string())
}
