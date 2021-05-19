pub(crate) mod commands;
pub(crate) mod error;
pub(crate) mod logger;

use anyhow::Result;

use commands::load;
use commands::{get_args, CliCommand};

#[macro_use]
extern crate log;

#[actix_rt::main]
async fn main() -> Result<()> {
    let cli = get_args();

    let res = match cli.command {
        CliCommand::Load(loadcmd) => load::handle_command(loadcmd).await,
    };

    std::process::exit(match res {
        Ok(out) => {
            println!("{}", out);
            0
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            1
        }
    });
}
