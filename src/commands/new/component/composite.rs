use anyhow::Result;
use clap::Args;
use structured_output::StructuredOutput;
use wick_config::config::components::ComponentConfig;
use wick_config::config::{self, ComponentConfiguration, CompositeComponentImplementation, FlowOperationBuilder};
use wick_interface_types::{Field, Type};

use crate::io::File;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  /// Name of the component.
  #[clap()]
  name: String,

  #[clap(long = "dry-run", action)]
  dry_run: bool,
}

pub(crate) async fn handle(
  opts: Options,
  _settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let _span = span.enter();
  let name = crate::commands::new::sanitize_name(&opts.name);
  let files: Result<Vec<File>> = span.in_scope(|| {
    info!("initializing wick composite component: {}", name);

    let mut config = ComponentConfiguration::default();
    config.set_name(name.clone());
    config.set_metadata(crate::commands::new::generic_metadata("New composite wick component"));

    let mut component = CompositeComponentImplementation::default();
    component.operations_mut().push(
      FlowOperationBuilder::default()
        .name("operation_name")
        .expressions(vec!["<>.input -> <>.output".parse().unwrap()])
        .inputs(vec![Field::new("input", Type::Object)])
        .outputs(vec![Field::new("output", Type::Object)])
        .build()
        .unwrap(),
    );

    config.set_component(config::ComponentImplementation::Composite(component));

    let config = wick_config::WickConfiguration::Component(config);

    Ok(vec![File::new(
      crate::commands::new::wickify_filename(&opts.name),
      config.into_v1_yaml()?.into(),
    )])
  });

  Ok(crate::io::init_files(&files?, opts.dry_run).await?)
}
