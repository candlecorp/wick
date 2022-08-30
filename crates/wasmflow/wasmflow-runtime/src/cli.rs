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
use wasmflow_manifest::Permissions;
use wasmflow_rpc::RpcHandler;
use wasmflow_sdk::v1::transport::{MessageTransport, Serialized, TransportMap};
use wasmflow_sdk::v1::{CollectionLink, Entity, InherentData, Invocation};
use {serde_value, serde_yaml};

use super::configuration::Channel;
use crate::dev::prelude::RuntimeError;

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CLI {
  location: String,
  component: String,
  link: String,
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
    let component = WapcModule::from_slice(&bytes)?;
    let permissions = Permissions {
      dirs: HashMap::from([(".".to_string(), "/".to_string())]),
    };
    let collection = Collection::try_load(&component, 1, None, Some(permissions), None)?;
    let inherent_data = InherentData::new(
      0,
      SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap(),
    );

    let link = CollectionLink::new(Entity::local("cli_channel"), Entity::collection(&self.link));

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
      Entity::local(&self.component),
      inputs_map,
      Some(inherent_data),
    );

    collection.invoke(invocation).await?;

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
      .context("Could not load from location")?;

    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    args.remove(0);
    args.remove(0);

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
      &"
  name: test
  version: 1.0.0
  channels:
    cli:
      uses: channels.wasmflow.cli@v1
      with:
        location: my_signed.wasm
        component: mycomponent
        link: my_link
"
      .to_owned(),
    )
    .unwrap();
    println!("{:?}", app_config);
    let cli_loader = get_channel_loader("channels.wasmflow.cli@v1").unwrap();
    let cli_config = app_config.channels.get("cli").unwrap();
    let cli_with = cli_config.with.to_owned();
    let cli = cli_loader(cli_with.to_owned()).unwrap();

    let cli = CLI::load_impl(cli_with).unwrap();
    assert_eq!(cli.location, "my_signed.wasm");
    assert_eq!(cli.component, "mycomponent");
    assert_eq!(cli.link, "my_link");
  }
}
