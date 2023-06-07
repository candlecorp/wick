use clap::Subcommand;
use wick_config::config;

pub(crate) mod app;
pub(crate) mod component;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SubCommands {
  /// Invoke a component in a collection.
  #[clap(subcommand, name = "component", alias = "comp")]
  Component(component::SubCommands),

  /// Query a collection for a list of its components.
  #[clap(name = "application", alias = "app")]
  App(app::Options),
}

fn generic_metadata(description: &str) -> config::Metadata {
  config::MetadataBuilder::default()
    .licenses(["Apache-2.0".to_owned()])
    .version("0.0.1")
    .description(Some(description.to_owned()))
    .build()
    .unwrap()
}
