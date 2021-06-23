pub mod run;
pub mod start;

use logger::options::LoggingOptions;
use structopt::{clap::AppSettings, StructOpt};

pub fn get_args() -> Cli {
  Cli::from_args()
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(
     global_settings(&[AppSettings::VersionlessSubcommands]),
     name = "vino", about = "Vino host runtime")]
pub struct Cli {
  #[structopt(flatten)]
  pub command: CliCommand,
}

#[derive(Debug, Clone, StructOpt)]
pub enum CliCommand {
  /// Start a long-running host with a manifest and schematics
  #[structopt(name = "start")]
  Start(start::StartCommand),
  /// Load a manifest and run the default schematic
  #[structopt(name = "run")]
  Run(run::RunCommand),
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct NatsOptions {
  /// Host for RPC connection, default = 0.0.0.0
  #[structopt(long = "rpc-host", env = "VINO_RPC_HOST")]
  pub rpc_host: Option<String>,

  /// Port for RPC connection, default = 4222
  #[structopt(long = "rpc-port", env = "VINO_RPC_PORT")]
  pub rpc_port: Option<String>,

  /// JWT file for RPC authentication. Must be supplied with rpc_seed.
  #[structopt(long = "rpc-jwt", env = "VINO_RPC_JWT", hide_env_values = true)]
  pub rpc_jwt: Option<String>,

  /// Seed file or literal for RPC authentication. Must be supplied with rpc_jwt.
  #[structopt(long = "rpc-seed", env = "VINO_RPC_SEED", hide_env_values = true)]
  pub rpc_seed: Option<String>,

  /// Credsfile for RPC authentication
  #[structopt(long = "rpc-credsfile", env = "VINO_RPC_CREDS", hide_env_values = true)]
  pub rpc_credsfile: Option<String>,

  /// Host for control interface, default = 0.0.0.0
  #[structopt(long = "control-host", env = "VINO_RPC_HOST")]
  pub control_host: Option<String>,

  /// Port for control interface, default = 4222
  #[structopt(long = "control-port", env = "VINO_RPC_PORT")]
  pub control_port: Option<String>,

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
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct HostOptions {
  /// Allows the use of "latest" artifact tag
  #[structopt(long = "allow-oci-latest", env = "VINO_ALLOW_LATEST")]
  pub allow_oci_latest: Option<bool>,

  /// Allows the use of HTTP registry connections to these registries
  #[structopt(long = "allowed-insecure")]
  pub allowed_insecure: Vec<String>,
}
