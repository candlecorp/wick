use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;
use wick_rpc::rpc::StatsRequest;
use wick_rpc::Statistics;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) connection: super::ConnectOptions,
}

pub(crate) async fn handle(
  opts: Options,
  _settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let _span = span.enter();
  let mut client = wick_rpc::make_rpc_client(
    format!("http://{}:{}", opts.connection.address, opts.connection.port),
    opts.connection.pem,
    opts.connection.key,
    opts.connection.ca,
    opts.connection.domain,
  )
  .await?;

  let list = client.stats(StatsRequest {}).await?;

  let mut converted: Vec<Statistics> = Vec::with_capacity(list.stats.len());

  for item in list.stats {
    converted.push(item.into());
  }

  Ok(StructuredOutput::new(
    serde_json::to_string(&converted)?,
    json! ({"stats": converted }),
  ))
}
