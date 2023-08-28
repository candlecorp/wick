use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;

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

  let list = client.list().await?;

  Ok(StructuredOutput::new(
    serde_json::to_string(&list)?,
    json! ({"list": list }),
  ))
}
