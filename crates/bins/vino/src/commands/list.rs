use std::path::PathBuf;

use clap::Args;
use logger::LoggingOptions;
use vino_host::HostBuilder;
use vino_manifest::host_definition::HostDefinition;
use vino_provider_cli::options::{DefaultCliOptions, LatticeCliOptions};
use vino_types::{MapWrapper, TypeMap};

use crate::utils::merge_config;
use crate::Result;
#[derive(Debug, Clone, Args)]
pub(crate) struct ListCommand {
  #[clap(flatten)]
  pub(crate) host: super::HostOptions,

  /// Specifies a manifest file to apply to the host once started.
  #[clap(parse(from_os_str))]
  pub(crate) manifest: Option<PathBuf>,

  #[clap(flatten)]
  pub(crate) logging: LoggingOptions,

  #[clap(flatten)]
  pub(crate) lattice: LatticeCliOptions,

  #[clap(long)]
  pub(crate) json: bool,
}

pub(crate) async fn handle_command(opts: ListCommand) -> Result<()> {
  let _guard = logger::init(&opts.logging.name("vino"));

  let config = match opts.manifest {
    Some(file) => HostDefinition::load_from_file(&file)?,
    None => HostDefinition::default(),
  };

  let server_options = DefaultCliOptions {
    lattice: opts.lattice,
    ..Default::default()
  };

  let mut config = merge_config(config, opts.host, Some(server_options));
  // Disable everything but the lattice
  config.host.rpc = None;
  config.host.http = None;

  let host_builder = HostBuilder::from_definition(config);

  let mut host = host_builder.build();
  host.connect_to_lattice().await?;
  host.start_network().await?;
  let signature = host.get_signature()?;

  if opts.json {
    let json = serde_json::to_string(&signature)?;
    println!("{}", json);
  } else {
    fn print_component(label: &str, indent: &str, inputs: &TypeMap, outputs: &TypeMap) {
      let inputs = inputs
        .inner()
        .iter()
        .map(|(name, _type)| format!("{}: {:?}", name, _type))
        .collect::<Vec<_>>()
        .join(", ");
      let outputs = outputs
        .inner()
        .iter()
        .map(|(name, _type)| format!("{}: {:?}", name, _type))
        .collect::<Vec<_>>()
        .join(", ");
      println!("{}{}({}) -> ({})", indent, label, inputs, outputs);
    }
    for (_name, schematic) in signature.components.inner().iter() {
      print!("Schematic: ");
      print_component(&schematic.name, "", &schematic.inputs, &schematic.outputs);
    }
  }

  Ok(())
}
