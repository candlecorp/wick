use std::collections::HashMap;
use std::sync::RwLock;

use nkeys::KeyPair;
use serde::Serialize;
use vino_entity::Entity;
use vino_provider_cli::cli::{
  Options as CliOpts,
  ServerMetadata,
};
use vino_runtime::network::NetworkBuilder;
use vino_runtime::prelude::*;

use crate::{
  Error,
  Result,
};

/// A Vino Host wraps a Vino runtime with server functionality like persistence,
#[derive(Debug)]
pub struct Host {
  pub(crate) host_id: String,
  pub(crate) kp: KeyPair,
  pub(crate) started: RwLock<bool>,
  pub(crate) network: Option<Network>,
}

impl Host {
  /// Starts the host. This call is non-blocking, so it is up to the consumer
  /// to provide some form of parking or waiting (e.g. host.wait_for_sigint()).
  pub async fn start(&self) -> Result<()> {
    debug!("Host starting");
    *self.started.write().unwrap() = true;
    Ok(())
  }

  /// Stops a running host.
  pub async fn stop(&self) {
    debug!("Host stopping");
    *self.started.write().unwrap() = false;
  }

  pub fn get_network(&self) -> Result<&Network> {
    self.network.as_ref().ok_or(Error::NoNetwork)
  }

  pub fn get_network_id(&self) -> Result<String> {
    self
      .network
      .as_ref()
      .ok_or(Error::NoNetwork)
      .map(|network| network.id.clone())
  }

  pub async fn start_network(&mut self, def: NetworkDefinition) -> Result<()> {
    ensure!(
      self.network.is_none(),
      crate::Error::InvalidHostState("Host already has a network running".into())
    );
    let seed = self.kp.seed()?;

    let network = NetworkBuilder::new(def, &seed)?.build();
    network.init().await?;
    self.network = Some(network);
    Ok(())
  }

  pub async fn start_rpc_server(&mut self, opts: Option<CliOpts>) -> Result<ServerMetadata> {
    let network_id = self.get_network_id()?;
    let metadata = tokio::spawn(vino_provider_cli::start_server(
      Box::new(NetworkProvider::new(network_id)),
      opts,
    ))
    .await
    .map_err(|e| Error::Other(format!("Join error: {}", e)))?
    .map_err(|e| Error::Other(format!("Socket error: {}", e)))?;
    Ok(metadata)
  }

  pub async fn request<T: AsRef<str> + Sync + Send, U: AsRef<str> + Sync + Send>(
    &self,
    schematic: T,
    payload: HashMap<U, impl Serialize + Sync + Send>,
  ) -> Result<MessageTransportStream> {
    match &self.network {
      Some(network) => Ok(
        network
          .request(schematic, Entity::host(&self.host_id), &payload)
          .await?,
      ),
      None => Err(crate::Error::InvalidHostState(
        "No network available".into(),
      )),
    }
  }

  pub async fn wait_for_sigint(&self) -> Result<()> {
    tokio::signal::ctrl_c().await.unwrap();
    debug!("SIGINT received");
    Ok(())
  }

  pub fn get_host_id(&self) -> String {
    self.host_id.clone()
  }

  fn _ensure_started(&self) -> Result<()> {
    ensure!(
      *self.started.read().unwrap(),
      crate::Error::InvalidHostState("Host not started".into())
    );
    Ok(())
  }

  pub fn is_started(&self) -> bool {
    *self.started.read().unwrap()
  }
}

#[cfg(test)]
mod test {
  use std::collections::HashMap;
  use std::net::Ipv4Addr;
  use std::path::PathBuf;
  use std::str::FromStr;

  use http::Uri;
  use maplit::hashmap;
  use vino_entity::entity::Entity;
  use vino_macros::transport_map;
  use vino_provider_cli::cli::ServerOptions;
  use vino_rpc::rpc::Invocation;
  use vino_rpc::{
    convert_transport_map,
    make_rpc_client,
  };

  use crate::host_definition::HostDefinition;
  use crate::{
    HostBuilder,
    Result,
  };

  #[test_env_log::test(actix::test)]
  async fn should_start_and_stop() -> Result<()> {
    let host = HostBuilder::new().start().await?;
    host.stop().await;

    assert!(!host.is_started());
    Ok(())
  }

  #[test_env_log::test(actix::test)]
  async fn ensure_started() -> Result<()> {
    let host = HostBuilder::new().start().await?;
    host._ensure_started()?;
    host.stop().await;

    assert!(!host.is_started());
    Ok(())
  }

  #[test_env_log::test(actix::test)]
  async fn request_from_network() -> Result<()> {
    let mut host = HostBuilder::new().start().await?;
    let file = PathBuf::from("src/configurations/logger.yaml");
    let manifest = HostDefinition::load_from_file(&file)?;
    host.start_network(manifest.network).await?;
    let passed_data = "logging output";
    let data: HashMap<&str, &str> = hashmap! {
        "input" => passed_data,
    };
    let mut stream = host.request("logger", data).await?;
    let mut messages: Vec<_> = stream.collect_port("output").await;
    assert_eq!(messages.len(), 1);
    let output = messages.pop().unwrap();
    let result: String = output.payload.try_into()?;
    assert_eq!(result, passed_data);
    host.stop().await;

    assert!(!host.is_started());
    Ok(())
  }

  #[test_env_log::test(actix::test)]
  async fn request_from_rpc_server() -> Result<()> {
    let mut host = HostBuilder::new().start().await?;
    let file = PathBuf::from("src/configurations/logger.yaml");
    let manifest = HostDefinition::load_from_file(&file)?;
    host.start_network(manifest.network).await?;
    let _metadata = host
      .start_rpc_server(Some(super::CliOpts {
        rpc: Some(ServerOptions {
          address: Some(Ipv4Addr::from_str("127.0.0.1").unwrap()),
          port: Some(54321),
          ..Default::default()
        }),
        ..Default::default()
      }))
      .await?;
    let mut client = make_rpc_client(Uri::from_str("https://127.0.0.1:54321").unwrap()).await?;
    let passed_data = "logging output";
    let data = transport_map! {
        "input" => passed_data,
    };
    let mut response = client
      .invoke(Invocation {
        origin: Entity::test("test").url(),
        target: Entity::schematic("logger").url(),
        msg: convert_transport_map(data),
        id: "some inv".to_owned(),
        network_id: host.get_host_id(),
      })
      .await
      .unwrap()
      .into_inner();
    let next = response.message().await.unwrap().unwrap();

    debug!("output: {:?}", next);

    host.stop().await;

    assert!(!host.is_started());
    Ok(())
  }
}
