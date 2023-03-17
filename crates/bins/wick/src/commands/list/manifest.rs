use anyhow::Result;
use wick_component_cli::options::DefaultCliOptions;
use wick_config::ComponentConfiguration;
use wick_host::ComponentHostBuilder;
use wick_interface_types::Field;

use crate::utils::merge_config;

pub(crate) async fn handle_command(opts: super::ListCommand, bytes: Vec<u8>) -> Result<()> {
  let manifest = ComponentConfiguration::load_from_bytes(Some(opts.location), &bytes)?;

  let server_options = DefaultCliOptions { ..Default::default() };

  let mut config = merge_config(&manifest, &opts.fetch, Some(server_options));
  // Disable everything but the mesh
  config.host_mut().rpc = None;

  let host_builder = ComponentHostBuilder::from_definition(config);

  let mut host = host_builder.build();
  // host.connect_to_mesh().await?;
  host.start_network(None).await?;
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
