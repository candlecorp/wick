use anyhow::Result;
use clap::Args;
use structured_output::StructuredOutput;
use wick_config::config::components::{
  SqlComponentConfigBuilder,
  SqlOperationDefinition,
  SqlQueryOperationDefinitionBuilder,
};
use wick_config::config::{self, ComponentConfiguration};
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
    info!("initializing wick sql component: {}", name);

    let mut config = ComponentConfiguration::default();
    config.set_name(name.clone());
    let resource_name = "DB_URL";
    config.resources_mut().push(config::Binding::new(
      resource_name,
      config::ResourceDefinition::Url(config::UrlResource::new(
        "postgres://postgres:postgres@localhost:5432/db_name".parse().unwrap(),
      )),
    ));

    config.set_metadata(crate::commands::new::generic_metadata("New SQL wick component"));

    let component = SqlComponentConfigBuilder::default()
      .resource(resource_name)
      .operations([SqlOperationDefinition::Query(
        SqlQueryOperationDefinitionBuilder::default()
          .name("example_query".to_owned())
          .inputs([Field::new("id", Type::String)])
          .query("SELECT * FROM users WHERE id = $1".to_owned())
          .arguments(["id".to_owned()])
          .build()
          .unwrap(),
      )])
      .build()
      .unwrap();

    config.set_component(config::ComponentImplementation::Sql(component));

    let config = wick_config::WickConfiguration::Component(config);

    Ok(vec![File::new(
      crate::commands::new::wickify_filename(&opts.name),
      config.into_v1_yaml()?.into(),
    )])
  });
  Ok(crate::io::init_files(&files?, opts.dry_run).await?)
}
