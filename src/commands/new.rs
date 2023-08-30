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

fn wickify_filename(name: &str) -> String {
  if name.contains('.') {
    name.to_owned()
  } else {
    format!("{}.wick", name)
  }
}

fn sanitize_name(name: &str) -> String {
  let non_char = regex::Regex::new(r"[^a-zA-Z0-9_]+").unwrap();
  non_char.replace_all(name.trim_end_matches(".wick"), "_").into_owned()
}
