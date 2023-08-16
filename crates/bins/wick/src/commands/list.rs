use std::fmt::Write;

use anyhow::Result;
use clap::Args;
use option_utils::OptionUtils;
use serde_json::json;
use structured_output::StructuredOutput;
use wick_component_cli::options::DefaultCliOptions;
use wick_config::WickConfiguration;
use wick_host::ComponentHostBuilder;
use wick_interface_types::Field;

use crate::utils::merge_config;

#[derive(Debug, Clone, Args)]
#[group(skip)]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) oci: crate::options::oci::OciOptions,

  #[clap(flatten)]
  pub(crate) component: crate::options::component::ComponentOptions,
}

pub(crate) async fn handle(
  opts: Options,
  _settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let fetch_options: wick_oci_utils::OciOptions = opts.oci.clone().into();

  let manifest = WickConfiguration::fetch(&opts.component.path, fetch_options)
    .await?
    .finish()?
    .try_component_config()?;

  let server_options = DefaultCliOptions { ..Default::default() };

  let mut config = merge_config(&manifest, &opts.oci, Some(server_options));
  // Disable everything but the mesh
  config.host_mut().inner_mut(|h| {
    h.set_rpc(None);
  });

  let mut host = ComponentHostBuilder::default().manifest(config).span(span).build()?;

  host.start_runtime(None).await?;
  let mut signature = host.get_signature()?;

  // If we have a name from the manifest, override the host id the component host generates.
  if let Some(name) = manifest.name() {
    signature.name = Some(name.clone());
  }

  let mut output = String::new();
  for op in &signature.operations {
    write!(&mut output, "Component: ")?;
    write_line(&mut output, &op.name, "", &op.inputs, &op.outputs)?;
  }

  Ok(StructuredOutput::new(
    output,
    json!({
      "result": signature,
    }),
  ))
}

fn write_line(
  mut buff: impl Write,
  label: &str,
  indent: &str,
  inputs: &[Field],
  outputs: &[Field],
) -> std::fmt::Result {
  let inputs = inputs.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ");
  let outputs = outputs.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ");
  write!(buff, "{}{}({}) -> ({})", indent, label, inputs, outputs)
}
