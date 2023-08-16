use clap::Args;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct ComponentOptions {
  /// Path or OCI url to manifest or wasm file.
  #[clap(action)]
  pub(crate) path: String,

  /// Pass a seed along with the invocation.
  #[clap(long = "seed", short = 's', env = "WICK_SEED", action)]
  pub(crate) seed: Option<u64>,

  /// Pass configuration necessary to instantiate the component (JSON).
  #[clap(long = "with", short = 'w', action)]
  pub(crate) with: Option<String>,
}

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct OperationOptions {
  /// Name of the operation.
  #[clap(action)]
  pub(crate) operation_name: String,

  /// Pass configuration necessary to invoke the operation (JSON).
  #[clap(long = "op-with", action)]
  pub(crate) op_with: Option<String>,
}
