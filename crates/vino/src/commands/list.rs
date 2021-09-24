use std::path::PathBuf;

use logger::LoggingOptions;
use structopt::StructOpt;
use vino_host::HostBuilder;
use vino_manifest::host_definition::HostDefinition;
use vino_provider_cli::cli::{
  DefaultCliOptions,
  LatticeCliOptions,
};
use vino_types::signatures::PortSignature;

use crate::utils::merge_config;
use crate::Result;
#[derive(Debug, Clone, StructOpt)]
pub(crate) struct ListCommand {
  #[structopt(flatten)]
  pub(crate) host: super::HostOptions,

  /// Specifies a manifest file to apply to the host once started.
  #[structopt(parse(from_os_str))]
  pub(crate) manifest: Option<PathBuf>,

  #[structopt(flatten)]
  pub(crate) logging: LoggingOptions,

  #[structopt(flatten)]
  pub(crate) lattice: LatticeCliOptions,

  #[structopt(long)]
  pub(crate) json: bool,

  #[structopt()]
  pub(crate) schematic: Option<String>,
}

pub(crate) async fn handle_command(opts: ListCommand) -> Result<String> {
  vino_provider_cli::init_logging(&opts.logging)?;

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
  let mut list = host.list_schematics().await?;
  if let Some(schematic) = opts.schematic {
    let target = list.into_iter().find(|sc| sc.name == schematic);
    list = match target {
      Some(t) => vec![t],
      None => vec![],
    };
  }
  if opts.json {
    let json = serde_json::to_string(&list)?;
    println!("{}", json);
  } else {
    fn print_component(
      label: &str,
      indent: &str,
      inputs: &[PortSignature],
      outputs: &[PortSignature],
    ) {
      let inputs = inputs
        .iter()
        .map(|p| format!("{}: {}", p.name, p.type_string))
        .collect::<Vec<_>>()
        .join(", ");
      let outputs = outputs
        .iter()
        .map(|p| format!("{}: {}", p.name, p.type_string))
        .collect::<Vec<_>>()
        .join(", ");
      println!("{}{}({}) -> ({})", indent, label, inputs, outputs);
    }
    for schematic in list {
      print!("Schematic: ");
      print_component(&schematic.name, "", &schematic.inputs, &schematic.outputs);
      println!("Available components:");
      for provider in schematic.providers {
        for component in provider.components {
          print_component(
            &format!("{}::{}", provider.name, component.name),
            " ",
            &component.inputs,
            &component.outputs,
          );
        }
      }
    }
  }

  Ok("Done".to_owned())
}
