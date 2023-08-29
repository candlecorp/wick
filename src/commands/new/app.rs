use anyhow::Result;
use clap::{Args, ValueEnum};
use structured_output::StructuredOutput;
use wick_config::config::components::{ComponentReference, ManifestComponentBuilder};
use wick_config::config::{
  self,
  AppConfiguration,
  ComponentDefinition,
  ComponentOperationExpressionBuilder,
  ImportBinding,
  ResourceBindingBuilder,
  ScheduleConfigBuilder,
  TcpPort,
};
use wick_config::WickConfiguration;

use crate::io::File;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum TriggerType {
  Time,
  Http,
  Cli,
}

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  /// Name of the project.
  #[clap()]
  name: String,

  #[clap(short = 't', long = "trigger", value_enum)]
  /// Triggers to initialize.
  triggers: Vec<TriggerType>,

  #[clap(long = "dry-run", action)]
  dry_run: bool,
}

pub(crate) async fn handle(
  opts: Options,
  _settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let files: Result<Vec<File>> = span.in_scope(|| {
    info!("initializing wick application: {}", opts.name);
    let mut config = AppConfiguration::default();
    config.set_name(opts.name);
    config.set_metadata(crate::commands::new::generic_metadata("New wick application"));

    for trigger in opts.triggers {
      match trigger {
        TriggerType::Time => {
          let comp_name = "COMPONENT";
          config.import_mut().push(ImportBinding::component(
            comp_name,
            config::ComponentDefinition::Manifest(
              ManifestComponentBuilder::default()
                .reference("path/to/component.wick")
                .build()
                .unwrap(),
            ),
          ));

          config.triggers_mut().push(config::TriggerDefinition::Time(
            config::TimeTriggerConfigBuilder::default()
              .operation(
                ComponentOperationExpressionBuilder::default()
                  .component(ComponentDefinition::Reference(ComponentReference::new(comp_name)))
                  .name("main")
                  .build()
                  .unwrap(),
              )
              .schedule(
                ScheduleConfigBuilder::default()
                  .cron("*/5 * * * *")
                  .repeat(10_u16)
                  .build()?,
              )
              .build()?,
          ));
        }
        TriggerType::Http => {
          let port_name = "HTTP_PORT";
          config.resources_mut().push(
            ResourceBindingBuilder::default()
              .id(port_name)
              .kind(config::ResourceDefinition::TcpPort(TcpPort::new("0.0.0.0", 8080)))
              .build()
              .unwrap(),
          );
          config.triggers_mut().push(config::TriggerDefinition::Http(
            config::HttpTriggerConfigBuilder::default()
              .resource(port_name)
              .build()
              .unwrap(),
          ));
        }
        TriggerType::Cli => {
          let comp_name = "MAIN_COMPONENT";
          config.import_mut().push(ImportBinding::component(
            comp_name,
            config::ComponentDefinition::Manifest(
              ManifestComponentBuilder::default()
                .reference("path/to/component.wick")
                .build()
                .unwrap(),
            ),
          ));

          config.triggers_mut().push(config::TriggerDefinition::Cli(
            config::CliConfigBuilder::default()
              .operation(
                ComponentOperationExpressionBuilder::default()
                  .component(ComponentDefinition::Reference(ComponentReference::new(comp_name)))
                  .name("main")
                  .build()
                  .unwrap(),
              )
              .build()
              .unwrap(),
          ));
        }
      }
    }
    let config = WickConfiguration::App(config);

    Ok(vec![File::new("app.wick", config.into_v1_yaml()?.into())])
  });

  Ok(crate::io::init_files(&files?, opts.dry_run).await?)
}
