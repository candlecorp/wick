use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use std::{env, fmt};

use anyhow::{Context, Result};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use wick_component_wasm::collection::Collection;
use wick_component_wasm::helpers::WickWasmModule;
use wick_config_component::collection_definition::WasmComponent;
use wick_config_component::flow_definition::PortReference;
use wick_config_component::host_definition::HostConfig;
use wick_config_component::{
  CollectionDefinition,
  ComponentDefinition,
  ConnectionDefinition,
  ConnectionTargetDefinition,
  Flow,
  Permissions,
};
use wick_packet::{CollectionLink, Entity, Invocation, Observer, Packet, PacketStream};
use wick_rpc::RpcHandler;
use {serde_value, serde_yaml};

use super::configuration::{ApplicationContext, Channel};
use crate::dev::prelude::RuntimeError;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  location: String,
  component: String,
  link: wick_config_component::v1::ComponentDefinition,
}

#[derive(Debug)]
pub struct CLI {
  app: ApplicationContext,
  config: Config,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct CliOptions {
  args: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Flag {
  name: String,
  value: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct IsInteractive {
  stdin: bool,
  stdout: bool,
  stderr: bool,
}

impl CLI {
  pub fn load(app: ApplicationContext, with: serde_value::Value) -> Result<Box<dyn Channel + Send + Sync>> {
    Ok(Box::new(CLI::load_impl(app, with)?))
  }

  pub fn load_impl(app: ApplicationContext, with: serde_value::Value) -> Result<CLI> {
    let config: Config = with
      .deserialize_into()
      .map_err(|e| RuntimeError::Serialization(e.to_string()))?;
    Ok(CLI { app, config })
  }

  async fn handle_command(&self, args: Vec<String>, bytes: Vec<u8>) -> Result<()> {
    let cli_namespace = "cli".to_owned();
    let linked_namespace = "linked".to_owned();

    let cli_collection = CollectionDefinition::new(
      &cli_namespace,
      wick_config_component::CollectionKind::wasm(&self.config.location, None, None),
    );
    let linked_collection = CollectionDefinition::new(&linked_namespace, self.config.link.clone().into());

    let network = crate::NetworkBuilder::new()
      .add_collection(cli_collection)
      .add_collection(linked_collection)
      .build()
      .await?;

    let link = CollectionLink::new(
      Entity::operation(&cli_namespace, &self.config.component),
      Entity::collection(&linked_namespace),
    );
    let is_interactive = IsInteractive {
      stdin: atty::is(atty::Stream::Stdin),
      stdout: atty::is(atty::Stream::Stdout),
      stderr: atty::is(atty::Stream::Stderr),
    };

    let (tx, packet_stream) = PacketStream::new_channels();
    tx.send(Packet::encode("args", &args));
    tx.send(Packet::encode("isInteractive", &is_interactive));
    tx.send(Packet::encode("program", &link));

    let invocation = Invocation::new(
      Entity::client("cli_channel"),
      Entity::operation(&cli_namespace, &self.config.component),
      self.app.inherent_data,
    );

    let _response = network.invoke(invocation, packet_stream).await?;

    Ok(())
  }
}

#[async_trait]
impl Channel for CLI {
  async fn run(&self) -> Result<()> {
    debug!("{}", self);

    let insecure: Vec<String> = Vec::new();
    let bytes = wick_loader_utils::get_bytes(&self.config.location, false, &insecure)
      .await
      .context(format!("Could not load from location {}", self.config.location))?;

    let mut args: Vec<String> = env::args().collect();
    // Preserve only the arguments after `--`.
    while !args.is_empty() && &args[0] != "--" {
      args.remove(0);
    }
    if !args.is_empty() && &args[0] == "--" {
      args.remove(0);
    }
    // Insert app name as the first argument.
    args.insert(0, self.app.name.clone());

    self.handle_command(args, bytes).await?;

    Ok(())
  }

  async fn shutdown_gracefully(&self) -> Result<()> {
    Ok(())
  }
}

impl fmt::Display for CLI {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "component: {}, link: {}",
      self.config.component, self.config.location
    )
  }
}

#[cfg(test)]
mod tests {
  use futures::executor::block_on;

  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::super::configuration::{from_string, get_channel_loader};
  use super::*;

  #[tokio::test]
  async fn test_initialize() {
    let context = ApplicationContext {
      name: "myapp".to_owned(),
      version: "1.0.0".to_owned(),
      inherent_data: None,
    };
    let app_config = from_string(
      "
  name: myapp
  version: 1.0.0
  channels:
    cli:
      uses: channels.wick.cli@v1
      with:
        location: my_signed.wasm
        component: mycomponent
        link:
          kind: Manifest
          reference: ./anyq.wafl
",
    )
    .unwrap();
    let cli_loader = get_channel_loader("channels.wick.cli@v1").unwrap();
    let cli_config = app_config.channels.get("cli").unwrap();
    let cli_with = cli_config.with.clone();
    let cli = cli_loader(context.clone(), cli_with.clone()).unwrap();

    let cli = CLI::load_impl(context, cli_with).unwrap();
    assert_eq!(cli.app.name, "myapp");
    assert_eq!(cli.app.version, "1.0.0");
    assert_eq!(cli.config.location, "my_signed.wasm");
    assert_eq!(cli.config.component, "mycomponent");
    // assert_eq!(cli.link, "my_link");
  }
}
