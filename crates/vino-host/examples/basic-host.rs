use vino_host::{
  Error,
  HostBuilder,
};

#[actix_rt::main]
async fn main() -> Result<(), Error> {
  let mut host = HostBuilder::new().build();
  host.start().await?;

  println!("Host started, waiting for ctrl-c / SIGINT");
  host.wait_for_sigint().await?;

  println!("SIGINT received, shutting down host");
  host.stop().await;

  Ok(())
}
