use anyhow::Result;
use clap::Args;
use wick_component_cli::options::DefaultCliOptions;
use wick_config::WickConfiguration;
use wick_host::ComponentHostBuilder;
use wick_interface_types::Field;
use wick_logger::LoggingOptions;

use crate::utils::merge_config;
#[derive(Debug, Clone, Args)]
pub(crate) struct ListCommand {
  #[clap(flatten)]
  pub(crate) fetch: super::FetchOptions,

  /// The path or OCI URL to a wick manifest or wasm file.
  #[clap(action)]
  pub(crate) location: String,

  #[clap(flatten)]
  pub(crate) logging: LoggingOptions,

  #[clap(long = "json", action)]
  pub(crate) json: bool,
}

pub(crate) async fn handle_command(opts: ListCommand) -> Result<()> {
  let _guard = wick_logger::init(&opts.logging.name(crate::BIN_NAME));

  let fetch_options = wick_config::config::FetchOptions::new()
    .allow_latest(opts.fetch.allow_latest)
    .allow_insecure(&opts.fetch.insecure_registries);

  let manifest = WickConfiguration::fetch(&opts.location, fetch_options)
    .await?
    .try_component_config()?;

  let server_options = DefaultCliOptions { ..Default::default() };

  let mut config = merge_config(&manifest, &opts.fetch, Some(server_options));
  // Disable everything but the mesh
  config.host_mut().rpc = None;

  let host_builder = ComponentHostBuilder::from_definition(config);

  let mut host = host_builder.build();
  // host.connect_to_mesh().await?;
  host.start_engine(None).await?;
  let signature = host.get_signature()?;

  if opts.json {
    let json = serde_json::to_string(&signature)?;
    println!("{}", json);
  } else {
    fn print_component(label: &str, indent: &str, inputs: &[Field], outputs: &[Field]) {
      let inputs = inputs.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ");
      let outputs = outputs.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ");
      println!("{}{}({}) -> ({})", indent, label, inputs, outputs);
    }
    for op in signature.operations {
      print!("Component: ");
      print_component(&op.name, "", &op.inputs, &op.outputs);
    }
  }
  Ok(())
}
