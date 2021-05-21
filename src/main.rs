pub(crate) mod commands;
pub(crate) mod error;
pub(crate) mod logger;
pub(crate) mod oci;
pub(crate) mod util;

use commands::{get_args, CliCommand};
use error::VinoError;

pub type Result<T> = anyhow::Result<T, VinoError>;
pub type Error = VinoError;

#[macro_use]
extern crate log;

#[actix_rt::main]
async fn main() -> Result<()> {
    let cli = get_args();

    let res = match cli.command {
        CliCommand::Start(cmd) => commands::start::handle_command(cmd).await,
        CliCommand::Run(cmd) => commands::run::handle_command(cmd).await,
    };

    std::process::exit(match res {
        Ok(out) => {
            info!("{}", out);
            0
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            println!("Run with --info, --debug, or --trace for more information.");
            1
        }
    });
}
