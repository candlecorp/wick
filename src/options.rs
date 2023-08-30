use clap::Args;
pub(crate) mod component;
pub(crate) mod logging;
pub(crate) mod oci;

#[derive(Args, Debug, Default, Clone)]
/// Global output options.
pub(crate) struct GlobalOptions {
  /// Print CLI output as JSON.
  #[clap(long = "json", short = 'j', global = true, action)]
  pub(crate) json: bool,
}
