use std::fmt::Write;

use anyhow::Result;
use clap::Args;
use serde_json::json;
use structured_output::StructuredOutput;
use wick_config::WickConfiguration;
use wick_interface_types::{Field, OperationSignature};

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
  _span: tracing::Span,
) -> Result<StructuredOutput> {
  let fetch_options: wick_oci_utils::OciOptions = opts.oci.clone().into();

  let manifest = WickConfiguration::fetch(&opts.component.path, fetch_options).await?;

  let signature = match manifest.manifest() {
    WickConfiguration::Component(c) => c.signature()?,
    _ => {
      anyhow::bail!("`wick list` only works on component configurations at this time.");
    }
  };

  let mut output = String::new();
  writeln!(&mut output, "Components:")?;
  writeln!(
    &mut output,
    "  └─ {} {}",
    signature.name.as_deref().unwrap_or("<unnamed>"),
    config(&signature.config)
  )?;
  for (i, op) in signature.operations.iter().enumerate() {
    if i < signature.operations.len() - 1 {
      write!(&mut output, "     ├─ ")?;
    } else {
      write!(&mut output, "     └─ ")?;
    }
    write_line(&mut output, op)?;
  }

  Ok(StructuredOutput::new(
    output,
    json!({
      "result": signature,
    }),
  ))
}

fn write_line(mut buff: impl Write, op: &OperationSignature) -> std::fmt::Result {
  write!(
    buff,
    "{} ({}): ({}) {}",
    op.name,
    fields(op.inputs()),
    fields(op.outputs()),
    config(op.config())
  )?;

  Ok(())
}

fn config(config: &[Field]) -> String {
  if config.is_empty() {
    return String::new();
  } else {
    format!(
      "with: {{ {} }}",
      config.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ")
    )
  }
}

fn fields(fields: &[Field]) -> String {
  fields.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ")
}
