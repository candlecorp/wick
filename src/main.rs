use vino::commands::{get_args, CliCommand};

use vino::Result;

#[macro_use]
extern crate log;

#[actix_rt::main]
async fn main() -> Result<()> {
    let cli = get_args();

    let res = match cli.command {
        CliCommand::Start(cmd) => vino::commands::start::handle_command(cmd).await,
        CliCommand::Run(cmd) => vino::commands::run::handle_command(cmd).await,
    };

    std::process::exit(match res {
        Ok(out) => {
            info!("{}", out);
            0
        }
        Err(e) => {
            println!("Vino exiting with error: {}", e);
            println!("Run with --info, --debug, or --trace for more information.");
            1
        }
    });
}
