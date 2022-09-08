use anyhow::Result;
use clap::Args;
use wasmflow_rpc::rpc::StatsRequest;
use wasmflow_rpc::Statistics;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) logging: logger::LoggingOptions,

  #[clap(flatten)]
  pub(crate) connection: super::ConnectOptions,
}

pub(crate) async fn handle(opts: Options) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;
  let mut client = wasmflow_rpc::make_rpc_client(
    opts.connection.uri,
    opts.connection.pem,
    opts.connection.key,
    opts.connection.ca,
    opts.connection.domain,
  )
  .await?;

  let list = client.stats(StatsRequest {}).await?;

  let mut converted: Vec<Statistics> = Vec::new();

  for item in list.stats {
    converted.push(item.into());
  }

  println!("{}", serde_json::to_string(&converted)?);

  Ok(())
}
