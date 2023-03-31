use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use seeded_random::Seed;
use uuid::Uuid;
use wick_component_cli::options::{Options as HostOptions, ServerOptions};
use wick_component_cli::ServerState;
use wick_config::config::ComponentConfiguration;
use wick_config::WickConfiguration;
use wick_interface_types::ComponentSignature;
use wick_packet::{Entity, InherentData, Invocation, PacketStream};
use wick_rpc::{RpcHandler, SharedRpcHandler};
use wick_runtime::{Network, NetworkBuilder, NetworkCollection};

use crate::{Error, Result};

type ServiceMap = HashMap<Uuid, SharedRpcHandler>;
static HOST_REGISTRY: Lazy<Mutex<ServiceMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn from_registry(id: Uuid) -> Arc<dyn RpcHandler + Send + Sync + 'static> {
  let mut registry = HOST_REGISTRY.lock();
  let collection = registry
    .entry(id)
    .or_insert_with(|| Arc::new(NetworkCollection::new(id)));
  collection.clone()
}

/// A Wick Host wraps a Wick runtime with server functionality like persistence,.
#[must_use]
#[derive(Debug)]
pub struct ComponentHost {
  id: String,
  network: Option<Network>,
  manifest: ComponentConfiguration,
  server_metadata: Option<ServerState>,
}

impl ComponentHost {
  /// Starts the host. This call is non-blocking, so it is up to the consumer
  /// to wait with a method like `host.wait_for_sigint()`.
  pub async fn start(&mut self, seed: Option<u64>) -> Result<()> {
    debug!("host starting");

    // self.mesh = self.get_mesh().await?;
    self.start_network(seed.map(Seed::unsafe_new)).await?;
    let state = self.start_servers().await?;
    self.server_metadata = Some(state);

    Ok(())
  }

  /// Get the address the host's RPC server is bound to.
  #[must_use]
  pub fn rpc_address(&self) -> Option<SocketAddr> {
    self
      .server_metadata
      .as_ref()
      .and_then(|state| state.rpc.as_ref().map(|rpc| rpc.addr))
  }

  pub fn get_signature(&self) -> Result<ComponentSignature> {
    match &self.network {
      Some(network) => Ok(network.get_signature()?),
      None => Err(Error::NoNetwork),
    }
  }

  #[must_use]
  pub fn get_server_info(&self) -> &Option<ServerState> {
    &self.server_metadata
  }

  /// Stops a running host.
  pub async fn stop(self) {
    debug!("host stopping");
    if let Some(network) = self.network {
      let _ = network.shutdown().await;
    }
  }

  pub fn get_network(&self) -> Result<&Network> {
    self.network.as_ref().ok_or(Error::NoNetwork)
  }

  pub fn get_network_uid(&self) -> Result<Uuid> {
    self.network.as_ref().ok_or(Error::NoNetwork).map(|network| network.uid)
  }

  pub async fn start_network(&mut self, seed: Option<Seed>) -> Result<()> {
    ensure!(
      self.network.is_none(),
      crate::Error::InvalidHostState("Host already has a network running".into())
    );

    let mut network_builder = NetworkBuilder::from_definition(self.manifest.clone())?;
    if let Some(seed) = seed {
      network_builder = network_builder.with_seed(seed);
    }
    network_builder = network_builder.namespace(self.get_host_id());
    network_builder = network_builder.allow_latest(self.manifest.host().allow_latest);
    network_builder = network_builder.allow_insecure(self.manifest.host().insecure_registries.clone());

    let network = network_builder.build().await?;

    self.network = Some(network);
    Ok(())
  }

  async fn start_servers(&mut self) -> Result<ServerState> {
    let nuid = self.get_network_uid()?;

    #[allow(clippy::manual_map)]
    let options = HostOptions {
      rpc: self.manifest.host().rpc.as_ref().map(|config| ServerOptions {
        port: config.port,
        address: config.address,
        pem: config.pem.clone(),
        key: config.key.clone(),
        ca: config.ca.clone(),
        enabled: config.enabled,
      }),
      id: self.get_host_id().to_owned(),
      timeout: self.manifest.host().timeout,
    };

    let collection = from_registry(nuid);

    let metadata = tokio::spawn(wick_component_cli::start_server(collection, Some(options)))
      .await
      .map_err(|e| Error::Other(format!("Join error: {}", e)))?
      .map_err(|e| Error::Other(format!("Socket error: {}", e)))?;

    Ok(metadata)
  }

  pub async fn request(
    &self,
    schematic: &str,
    stream: PacketStream,
    data: Option<InherentData>,
  ) -> Result<PacketStream> {
    match &self.network {
      Some(network) => {
        let invocation = Invocation::new(Entity::host(&self.id), Entity::local(schematic), data);
        Ok(network.invoke(invocation, stream).await?)
      }
      None => Err(crate::Error::InvalidHostState("No network available".into())),
    }
  }

  pub async fn invoke(&self, invocation: Invocation, stream: PacketStream) -> Result<PacketStream> {
    match &self.network {
      Some(network) => Ok(network.invoke(invocation, stream).await?),
      None => Err(crate::Error::InvalidHostState("No network available".into())),
    }
  }

  pub async fn wait_for_sigint(&self) -> Result<()> {
    tokio::signal::ctrl_c().await.unwrap();
    debug!("SIGINT received");
    Ok(())
  }

  #[must_use]
  pub fn get_host_id(&self) -> &str {
    &self.id
  }

  #[must_use]
  pub fn is_started(&self) -> bool {
    self.network.is_some()
  }
}

/// The HostBuilder builds the configuration for a Wick Host.
#[must_use]
#[derive(Debug, Clone)]
pub struct ComponentHostBuilder {
  manifest: ComponentConfiguration,
}

impl Default for ComponentHostBuilder {
  fn default() -> Self {
    Self::new()
  }
}

impl ComponentHostBuilder {
  /// Creates a new host builder.
  pub fn new() -> ComponentHostBuilder {
    ComponentHostBuilder {
      manifest: ComponentConfiguration::default(),
    }
  }

  pub async fn from_manifest_url(location: &str, allow_latest: bool, insecure_registries: &[String]) -> Result<Self> {
    let fetch_options = wick_config::config::FetchOptions::new()
      .allow_latest(allow_latest)
      .allow_insecure(insecure_registries);

    let manifest = WickConfiguration::fetch(wick_config::str_to_url(location, None)?, fetch_options)
      .await?
      .try_component_config()?;
    Ok(Self::from_definition(manifest))
  }

  pub fn from_definition(definition: ComponentConfiguration) -> Self {
    ComponentHostBuilder { manifest: definition }
  }

  /// Constructs an instance of a Wick host.
  pub fn build(self) -> ComponentHost {
    let host_id = Uuid::new_v4().to_string();

    ComponentHost {
      id: host_id,
      network: None,
      manifest: self.manifest,
      server_metadata: None,
    }
  }
}

// impl TryFrom<PathBuf> for ComponentHostBuilder {
//   type Error = Error;

//   fn try_from(file: PathBuf) -> Result<Self> {
//     let manifest = WickConfiguration::load_from_file(file)?.try_component_config()?;
//     Ok(ComponentHostBuilder::from_definition(manifest))
//   }
// }

// impl TryFrom<&str> for ComponentHostBuilder {
//   type Error = Error;

//   fn try_from(value: &str) -> Result<Self> {
//     ComponentHostBuilder::try_from(PathBuf::from(value))
//   }
// }

#[cfg(test)]
mod test {
  use std::net::Ipv4Addr;
  use std::path::PathBuf;
  use std::str::FromStr;

  use anyhow::Result;
  use futures::StreamExt;
  use http::Uri;
  use wick_config::config::HttpConfig;
  use wick_invocation_server::connect_rpc_client;
  use wick_packet::{packet_stream, packets, Entity, Packet};

  use super::*;
  use crate::ComponentHostBuilder;

  #[test]
  fn builds_default() {
    let _h = ComponentHostBuilder::new().build();
  }

  #[test_logger::test(tokio::test)]
  async fn should_start_and_stop() -> Result<()> {
    let mut host = ComponentHostBuilder::new().build();
    host.start(None).await?;
    assert!(host.is_started());
    host.stop().await;

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn request_direct() -> Result<()> {
    let file = PathBuf::from("manifests/logger.yaml");
    let manifest = WickConfiguration::load_from_file(&file).await?.try_component_config()?;
    let mut host = ComponentHostBuilder::from_definition(manifest).build();
    host.start(None).await?;
    let passed_data = "logging output";
    let stream = packet_stream!(("input", passed_data));
    let stream = host.request("logger", stream, None).await?;

    let mut messages: Vec<_> = stream.collect().await;
    assert_eq!(messages.len(), 2);
    messages.pop();
    let output = messages.pop().unwrap().unwrap();

    assert_eq!(output, Packet::encode("output", passed_data));
    host.stop().await;

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn request_rpc_server() -> Result<()> {
    let file = PathBuf::from("manifests/logger.yaml");
    let mut def = WickConfiguration::load_from_file(&file).await?.try_component_config()?;
    def.host_mut().rpc = Some(HttpConfig {
      enabled: true,
      port: None,
      address: Some(Ipv4Addr::from_str("127.0.0.1").unwrap()),
      ..Default::default()
    });

    let mut host = ComponentHostBuilder::from_definition(def).build();
    host.start(None).await?;
    let address = host.rpc_address().unwrap();
    println!("rpc server bound to : {}", address);

    let mut client = connect_rpc_client(Uri::from_str(&format!("http://{}", address)).unwrap()).await?;
    println!("connected to server");
    let passed_data = "logging output";
    let packets = packets![("input", passed_data)];
    let invocation: wick_rpc::rpc::Invocation = Invocation::new(Entity::test("test"), Entity::local("logger"), None)
      .try_into()
      .unwrap();

    let mut msgs = vec![wick_rpc::rpc::InvocationRequest {
      data: Some(wick_rpc::rpc::invocation_request::Data::Invocation(invocation)),
    }];

    for packet in packets {
      msgs.push(wick_rpc::rpc::InvocationRequest {
        data: Some(wick_rpc::rpc::invocation_request::Data::Packet(packet.into())),
      });
    }
    println!("invocation stream msgs: {:?}", msgs);
    let mut response = client.invoke(futures::stream::iter(msgs)).await.unwrap().into_inner();
    println!("got response");
    let next = response.message().await;
    println!("next: {:?}", next);
    let next = next.unwrap().unwrap();
    debug!(?next);

    host.stop().await;

    Ok(())
  }
}
