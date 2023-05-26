use anyhow::Result;
use clap::Args;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct RpcListCommand {
  #[clap(flatten)]
  pub(crate) connection: super::ConnectOptions,
}

pub(crate) async fn handle(
  opts: RpcListCommand,
  _settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<()> {
  let _span = span.enter();
  let mut client = wick_rpc::make_rpc_client(
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
