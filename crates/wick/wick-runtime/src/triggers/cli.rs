use std::{env, fmt};

use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use wick_config::{CliConfig, ComponentDefinition, TriggerDefinition};
use wick_packet::{packet_stream, CollectionLink, Entity, Invocation};

use super::{Trigger, TriggerKind};
use crate::dev::prelude::RuntimeError;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Config {
  location: String,
  component: String,
  link: wick_config::v1::ComponentDefinition,
}

#[derive(Debug)]
pub(crate) struct Cli {}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub(crate) struct CliOptions {
  args: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub(crate) struct Flag {
  name: String,
  value: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct IsInteractive {
  stdin: bool,
  stdout: bool,
  stderr: bool,
}

impl Cli {
  pub(crate) fn load() -> Result<Box<dyn Trigger + Send + Sync>, RuntimeError> {
    Ok(Box::new(Cli::load_impl()?))
  }

  pub(crate) fn load_impl() -> Result<Cli, RuntimeError> {
    Ok(Self {})
  }

  async fn handle_command(&self, config: CliConfig, args: Vec<String>) -> Result<(), RuntimeError> {
    let cli_namespace = "cli".to_owned();
    let linked_namespace = "linked".to_owned();

    if config.component().is_none() {
      unimplemented!("CLI Component definition is required.");
    }
    if config.app().is_none() {
      unimplemented!("CLI App component is required.");
    }

    let component = config.component().cloned().take().unwrap();
    let app = config.app().cloned().take().unwrap();

    let cli_collection = ComponentDefinition::new(&cli_namespace, component);
    let linked_collection = ComponentDefinition::new(&linked_namespace, app);

    let network = crate::NetworkBuilder::new()
      .add_collection(cli_collection)
      .add_collection(linked_collection)
      .build()
      .await?;

    let link = CollectionLink::new(
      Entity::operation(&cli_namespace, config.operation()).url(),
      &linked_namespace,
    );
    let is_interactive = IsInteractive {
      stdin: atty::is(atty::Stream::Stdin),
      stdout: atty::is(atty::Stream::Stdout),
      stderr: atty::is(atty::Stream::Stderr),
    };

    let packet_stream = packet_stream!(("args", args), ("isInteractive", is_interactive), ("program", link));

    let invocation = Invocation::new(
      Entity::client("cli_channel"),
      Entity::operation(&cli_namespace, config.operation()),
      None,
    );

    let mut response = network.invoke(invocation, packet_stream).await?;
    while let Some(packet) = response.next().await {
      trace!(?packet, "trigger:cli:response");
    }

    Ok(())
  }
}

#[async_trait]
impl Trigger for Cli {
  async fn run(&self, name: String, config: TriggerDefinition) -> Result<(), RuntimeError> {
    #[allow(irrefutable_let_patterns)]
    let config = if let TriggerDefinition::Cli(config) = config {
      config
    } else {
      return Err(RuntimeError::InvalidTriggerConfig(TriggerKind::Cli));
    };

    let mut args: Vec<String> = env::args().collect();
    // Preserve only the arguments after `--`.
    while !args.is_empty() && &args[0] != "--" {
      args.remove(0);
    }
    if !args.is_empty() && &args[0] == "--" {
      args.remove(0);
    }

    // Insert app name as the first argument.
    args.insert(0, name);

    self.handle_command(config, args).await?;

    Ok(())
  }

  async fn shutdown_gracefully(&mut self) -> Result<(), RuntimeError> {
    Ok(())
  }
}

impl fmt::Display for Cli {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Cli Trigger",)
  }
}
