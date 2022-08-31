use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use std::{env, fmt};

use anyhow::{Context, Result};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use wasmflow_collection_wasm::collection::Collection;
use wasmflow_collection_wasm::helpers::WapcModule;
use wasmflow_manifest::collection_definition::WasmCollection;
use wasmflow_manifest::flow_definition::PortReference;
use wasmflow_manifest::host_definition::HostConfig;
use wasmflow_manifest::{
  CollectionDefinition,
  ComponentDefinition,
  ConnectionDefinition,
  ConnectionTargetDefinition,
  Flow,
  Permissions,
};
use wasmflow_rpc::RpcHandler;
use wasmflow_sdk::v1::transport::{MessageTransport, Serialized, TransportMap};
use wasmflow_sdk::v1::{CollectionLink, Entity, InherentData, Invocation};
use wasmflow_wascap::KeyPair;
use {serde_value, serde_yaml};

use super::configuration::{ApplicationContext, Channel};
use crate::dev::prelude::RuntimeError;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  location: String,
  component: String,
  link: wasmflow_manifest::v1::CollectionDefinition,
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
  #[allow(unused)]
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
    let cli_collection = CollectionDefinition {
      namespace: "cli".to_owned(),
      kind: wasmflow_manifest::CollectionKind::Wasm(WasmCollection {
        reference: self.config.location.clone(),
        config: serde_json::Value::Null,
        permissions: Permissions::default(),
      }),
    };
    let linked_collection: CollectionDefinition = ("linked".to_owned(), self.config.link.clone()).try_into()?;
    let manifest = wasmflow_manifest::WasmflowManifestBuilder::new()
      .add_collection("cli", cli_collection)
      .add_collection("linked", linked_collection)
      .add_flow(
        "cli-component",
        Flow {
          name: "cli-component".to_owned(),
          instances: HashMap::from([(
            "cli-instance".to_owned(),
            ComponentDefinition {
              name: self.config.component.clone(),
              namespace: "cli".to_owned(),
              data: None,
            },
          )]),
          connections: vec![
            ConnectionDefinition {
              from: ConnectionTargetDefinition::new("<>", "argv"),
              to: ConnectionTargetDefinition::new("cli-instance", "argv"),
              default: None,
            },
            ConnectionDefinition {
              from: ConnectionTargetDefinition::new("<>", "isInteractive"),
              to: ConnectionTargetDefinition::new("cli-instance", "program"),
              default: None,
            },
            ConnectionDefinition {
              from: ConnectionTargetDefinition::new("<>", "program"),
              to: ConnectionTargetDefinition::new("cli-instance", "program"),
              default: None,
            },
          ],
          ..Default::default()
        },
      )
      .build();

    let builder = crate::NetworkBuilder::from_definition(manifest, &KeyPair::new_user().seed()?)?;
    let network = builder.build().await?;

    let link = CollectionLink::new(Entity::component("cli", "cli-instance"), Entity::collection(&"linked"));
    let is_interactive = IsInteractive {
      stdin: atty::is(atty::Stream::Stdin),
      stdout: atty::is(atty::Stream::Stdout),
      stderr: atty::is(atty::Stream::Stderr),
    };

    let mut inputs_map = TransportMap::default();
    inputs_map.insert(
      "args",
      MessageTransport::Success(Serialized::Json(serde_json::to_string(&args)?)),
    );
    inputs_map.insert(
      "isInteractive",
      MessageTransport::Success(Serialized::Json(serde_json::to_string(&is_interactive)?)),
    );
    inputs_map.insert(
      "program",
      MessageTransport::Success(Serialized::Json(serde_json::to_string(&link)?)),
    );
    inputs_map.transpose_output_name();

    let invocation = Invocation::new(
      Entity::client("cli_channel"),
      Entity::component("cli", &self.config.component),
      inputs_map,
      self.app.inherent_data,
    );

    let _response = network.invoke(invocation).await?;

    Ok(())
  }
}

#[async_trait]
impl Channel for CLI {
  async fn run(&self) -> Result<()> {
    debug!("{}", self);

    let insecure: Vec<String> = Vec::new();
    let bytes = wasmflow_loader::get_bytes(&self.config.location, false, &insecure)
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
      uses: channels.wasmflow.cli@v1
      with:
        location: my_signed.wasm
        component: mycomponent
        link:
          kind: Manifest
          reference: ./anyq.wafl
",
    )
    .unwrap();
    let cli_loader = get_channel_loader("channels.wasmflow.cli@v1").unwrap();
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
