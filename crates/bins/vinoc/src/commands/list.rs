use clap::Args;

use crate::Result;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) logging: super::LoggingOptions,

  #[clap(flatten)]
  pub(crate) connection: super::ConnectOptions,
}

pub(crate) async fn handle(opts: Options) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;
  let mut client = vino_rpc::make_rpc_client(
    format!("http://{}:{}", opts.connection.address, opts.connection.port),
    opts.connection.pem,
    opts.connection.key,
    opts.connection.ca,
    opts.connection.domain,
  )
  .await?;

  let list = client.list().await?;

  println!("{}", serde_json::to_string(&list)?);

  Ok(())
}
