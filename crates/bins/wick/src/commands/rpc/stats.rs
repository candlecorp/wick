use anyhow::Result;
use clap::Args;
use wick_rpc::rpc::StatsRequest;
use wick_rpc::Statistics;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct RpcStatsCommand {
  #[clap(flatten)]
  pub(crate) logging: logger::LoggingOptions,

  #[clap(flatten)]
  pub(crate) connection: super::ConnectOptions,
}

pub(crate) async fn handle(opts: RpcStatsCommand) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;
  let mut client = wick_rpc::make_rpc_client(
    format!("http://{}:{}", opts.connection.address, opts.connection.port),
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
