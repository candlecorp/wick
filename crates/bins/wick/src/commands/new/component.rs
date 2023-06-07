use clap::Subcommand;

pub(crate) mod composite;
pub(crate) mod http;
pub(crate) mod sql;
pub(crate) mod wasmrs;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SubCommands {
  /// Create a new HTTP client component
  #[clap(name = "http")]
  Http(http::Options),

  /// Create a new wick composite component
  #[clap(name = "composite")]
  Composite(composite::Options),

  /// Create a new SQL DB component
  #[clap(name = "sql")]
  Sql(sql::Options),

  /// Create a new wick wasmrs component
  #[clap(name = "wasmrs")]
  WasmRS(wasmrs::Options),
}
