use std::path::PathBuf;

use crate::error::VinoError;
use env_logger::WriteStyle;
use structopt::{clap::AppSettings, StructOpt};

pub fn get_args() -> Cli {
    Cli::from_args()
}

fn parse_write_style(spec: &str) -> std::result::Result<WriteStyle, VinoError> {
    match spec {
        "auto" => Ok(WriteStyle::Auto),
        "always" => Ok(WriteStyle::Always),
        "never" => Ok(WriteStyle::Never),
        _ => Err(VinoError::ConfigurationError),
    }
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(
     global_settings(&[AppSettings::ColoredHelp, AppSettings::VersionlessSubcommands]),
     name = "vino")]
pub struct Cli {
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
    #[structopt(long = "manifest", short = "m", parse(from_os_str))]
    pub manifest: Option<PathBuf>,

    /// Disables logging
    #[structopt(long = "quiet", short = "q")]
    pub quiet: bool,

    /// Outputs the version
    #[structopt(long = "version", short = "v")]
    pub version: bool,

    /// Turns on verbose logging
    #[structopt(long = "verbose", short = "V")]
    pub verbose: bool,

    /// Turns on debug logging
    #[structopt(long = "debug")]
    pub debug: bool,

    /// Turns on trace logging
    #[structopt(long = "trace")]
    pub trace: bool,

    /// Log as JSON
    #[structopt(long = "json")]
    pub json: bool,

    /// Log style
    #[structopt(
        long = "log-style",
        env = "VINO_LOG_STYLE",
        parse(try_from_str = parse_write_style),
        default_value="auto"
    )]
    pub log_style: WriteStyle,
}
