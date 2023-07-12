use anyhow::Result;
use clap::Args;
use structured_output::StructuredOutput;
use wick_config::config::{
  self,
  ComponentConfiguration,
  OperationDefinitionBuilder,
  WasmComponentImplementationBuilder,
};
use wick_config::AssetReference;
use wick_interface_types::{Field, StructDefinition, Type, TypeDefinition};

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
    info!("Initializing wick http component: {}", name);
    info!("Note: WebAssembly components are often better suited by cloning a boilerplate project. See https://github.com/candlecorp/wick for directions.");

    let mut config = ComponentConfiguration::default();
    config.set_name(name.clone());

    let example_typedef = TypeDefinition::Struct(StructDefinition::new(
      "user_object",
      vec![
        Field::new("id", Type::String),
        Field::new("name", Type::String),
        Field::new("email", Type::String),
      ],
    ));

    config.types_mut().push(example_typedef);

    config.set_metadata(crate::commands::new::generic_metadata("New WebAssembly wick component"));

    let component = WasmComponentImplementationBuilder::default()
      .reference(AssetReference::new(format!("./build/{}", &name)))
      .operations([
        OperationDefinitionBuilder::default()
          .name("operation_name".to_owned())
          .inputs([Field::new("id", Type::String)])
          .outputs([Field::new("output", Type::Named("user_object".to_owned()))])
          .build()
          .unwrap(),
      ])
      .build()
      .unwrap();

    config.set_component(config::ComponentImplementation::Wasm(component));

    let config = wick_config::WickConfiguration::Component(config);

    Ok(vec![File::new(
      crate::commands::new::wickify_filename(&opts.name),
      config.into_v1_yaml()?.into(),
    )])
  });
  Ok(crate::io::init_files(&files?, opts.dry_run).await?)
}
