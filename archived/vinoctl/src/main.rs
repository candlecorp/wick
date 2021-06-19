use vinoctl::commands::{get_args, CliCommand};

use vinoctl::Result;

#[macro_use]
extern crate log;

#[actix_rt::main]
async fn main() -> Result<()> {
  let cli = get_args();

  let res = match cli.command {
    CliCommand::Start(cmd) => vinoctl::commands::start::handle_command(cmd).await,
  };

  std::process::exit(match res {
    Ok(out) => {
      info!("{}", out);
      0
    }
    Err(e) => {
      eprintln!("Vino exiting with error: {}", e);
      println!("Run with --info, --debug, or --trace for more information.");
      1
    }
  });
}
