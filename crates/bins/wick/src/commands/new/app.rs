use anyhow::Result;
use clap::{Args, ValueEnum};
use structured_output::StructuredOutput;
use wick_config::config::{self, AppConfiguration};
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
    info!("Initializing wick application: {}", opts.name);
    let mut config = AppConfiguration::default();
    config.set_name(opts.name);
    config.set_metadata(crate::commands::new::generic_metadata("New wick application"));

    for trigger in opts.triggers {
      match trigger {
        TriggerType::Time => {
          config.triggers_mut().push(config::TriggerDefinition::Time(
            config::app_config::TimeTriggerConfigBuilder::default().build().unwrap(),
          ));
        }
        TriggerType::Http => {
          config.triggers_mut().push(config::TriggerDefinition::Http(
            config::HttpTriggerConfigBuilder::default().build().unwrap(),
          ));
        }
        TriggerType::Cli => {
          config.triggers_mut().push(config::TriggerDefinition::Cli(
            config::CliConfigBuilder::default().build().unwrap(),
          ));
        }
      }
    }
    let config = WickConfiguration::App(config);

    Ok(vec![File::new("app.wick", config.into_v1_yaml()?.into())])
  });

  Ok(crate::io::init_files(&files?, opts.dry_run).await?)
}
