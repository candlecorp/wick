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

use super::configuration::Channel;
use crate::dev::prelude::RuntimeError;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CLI {
  location: String,
  component: String,
  link: wasmflow_manifest::v1::CollectionDefinition,
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

impl CLI {
  #[allow(unused)]
  pub fn load(with: serde_value::Value) -> Result<Box<dyn Channel + Send + Sync>> {
    Ok(Box::new(CLI::load_impl(with)?))
  }

  pub fn load_impl(with: serde_value::Value) -> Result<CLI> {
    let cli: CLI = with
      .deserialize_into()
      .map_err(|e| RuntimeError::Serialization(e.to_string()))?;
    Ok(cli)
  }

  async fn handle_command(&self, args: Vec<String>, bytes: Vec<u8>) -> Result<()> {
    let cli_collection = CollectionDefinition {
      namespace: "cli".to_owned(),
      kind: wasmflow_manifest::CollectionKind::Wasm(WasmCollection {
        reference: self.location.clone(),
        config: serde_json::Value::Null,
        permissions: Permissions::default(),
      }),
    };
    let linked_collection: CollectionDefinition = ("linked".to_owned(), self.link.clone()).try_into()?;
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
              name: self.component.clone(),
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

    let mut inputs_map = TransportMap::default();
    inputs_map.insert(
      "argv",
      MessageTransport::Success(Serialized::Json(serde_json::to_string(&args)?)),
    );
    inputs_map.insert(
      "program",
      MessageTransport::Success(Serialized::Json(serde_json::to_string(&link)?)),
    );
    inputs_map.transpose_output_name();

    let invocation = Invocation::new(
      Entity::client("cli_channel"),
      Entity::component("cli", &self.component),
      inputs_map,
      None,
    );

    let _response = network.invoke(invocation).await?;

    // collection.invoke(invocation).await?;

    Ok(())
  }
}

#[async_trait]
impl Channel for CLI {
  async fn run(&self) -> Result<()> {
    debug!("{}", self);

    let insecure: Vec<String> = Vec::new();
    let bytes = wasmflow_loader::get_bytes(&self.location, false, &insecure)
      .await
      .context(format!("Could not load from location {}", self.location))?;

    let mut args: Vec<String> = env::args().collect();
    while !args.is_empty() && &args[0] != "--" {
      args.remove(0);
    }
    if !args.is_empty() && &args[0] == "--" {
      args.remove(0);
    }

    self.handle_command(args, bytes).await?;

    Ok(())
  }

  async fn shutdown_gracefully(&self) -> Result<()> {
    Ok(())
  }
}

impl fmt::Display for CLI {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "component: {}, link: {}", self.component, self.location)
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
    let app_config = from_string(
      "
  name: test
  version: 1.0.0
  channels:
    cli:
      uses: channels.wasmflow.cli@v1
      with:
        location: my_signed.wasm
        component: mycomponent
        link: my_link
",
    )
    .unwrap();
    println!("{:?}", app_config);
    let cli_loader = get_channel_loader("channels.wasmflow.cli@v1").unwrap();
    let cli_config = app_config.channels.get("cli").unwrap();
    let cli_with = cli_config.with.clone();
    let cli = cli_loader(cli_with.clone()).unwrap();

    let cli = CLI::load_impl(cli_with).unwrap();
    assert_eq!(cli.location, "my_signed.wasm");
    assert_eq!(cli.component, "mycomponent");
    // assert_eq!(cli.link, "my_link");
  }
}
