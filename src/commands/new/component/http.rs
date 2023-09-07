use anyhow::Result;
use clap::Args;
use structured_output::StructuredOutput;
use wick_config::config::components::{HttpClientComponentConfigBuilder, HttpClientOperationDefinitionBuilder};
use wick_config::config::{self, Binding, Codec, ComponentConfiguration, ResourceDefinition, UrlResource};
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
    info!("initializing wick http component: {}", name);

    let resource_name = "HTTP_URL";

    let mut config = ComponentConfiguration::default();
    config.set_name(name.clone());

    config.resources_mut().push(Binding::new(
      resource_name,
      ResourceDefinition::Url(UrlResource::new("http://localhost:8080".parse().unwrap())),
    ));

    config.set_metadata(crate::commands::new::generic_metadata("New HTTP Client wick component"));

    let component = HttpClientComponentConfigBuilder::default()
      .codec(Codec::Json)
      .resource(resource_name)
      .operations([HttpClientOperationDefinitionBuilder::default()
        .name("example_request".to_owned())
        .method(config::HttpMethod::Get)
        .path("/user/{id:string}".to_owned())
        .inputs([Field::new("id", Type::String)])
        .build()
        .unwrap()])
      .build()
      .unwrap();

    config.set_component(config::ComponentImplementation::HttpClient(component));

    let config = wick_config::WickConfiguration::Component(config);

    Ok(vec![File::new(
      crate::commands::new::wickify_filename(&opts.name),
      config.into_v1_yaml()?.into(),
    )])
  });
  Ok(crate::io::init_files(&files?, opts.dry_run).await?)
}
