use vinoc::commands::{
  get_args,
  CliCommand,
};
use vinoc::Result;

#[macro_use]
extern crate log;

#[actix_rt::main]
async fn main() -> Result<()> {
  let cli = get_args();

  let res = match cli.command {
    CliCommand::Invoke(cmd) => vinoc::commands::invoke::handle_command(cmd).await,
    CliCommand::Stats(cmd) => vinoc::commands::stats::handle_command(cmd).await,
    CliCommand::List(cmd) => vinoc::commands::list::handle_command(cmd).await,
    CliCommand::Sign(cmd) => vinoc::commands::sign::handle_command(cmd).await,
    CliCommand::Inspect(cmd) => vinoc::commands::inspect::handle_command(cmd).await,
  };

  std::process::exit(match res {
    Ok(out) => {
      debug!("{}", out);
      0
    }
    Err(e) => {
      eprintln!("Vino exiting with error: {}", e);
      eprintln!("Run with --info, --debug, or --trace for more information.");
      1
    }
  });
}
