pub mod run;
pub mod start;
use structopt::{clap::AppSettings, StructOpt};

use crate::{error::VinoError, logger::Logger};
use anyhow::{Context, Result};
use env_logger::WriteStyle;

pub fn get_args() -> Cli {
    Cli::from_args()
}

pub fn init_logger(opts: &LoggingOpts) -> Result<()> {
    Logger::init(&opts).context("Failed to start logger")
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
     global_settings(&[AppSettings::VersionlessSubcommands]),
     name = "vino", about = "Vino host runtime")]
pub struct Cli {
    #[structopt(flatten)]
    pub command: CliCommand,
}

#[derive(Debug, Clone, StructOpt)]
pub enum CliCommand {
    /// Start a host with a manifest and schematics
    #[structopt(name = "start")]
    Start(start::StartCommand),
    /// Run a Vino component on its own
    #[structopt(name = "run")]
    Run(run::RunCli),
}

#[derive(StructOpt, Debug, Clone)]
pub struct LoggingOpts {
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
