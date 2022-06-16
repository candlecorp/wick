use std::collections::HashMap;
use std::convert::TryFrom;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use nkeys::KeyPair;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use seeded_random::Seed;
use uuid::Uuid;
use wasmflow_collection_cli::options::{MeshOptions, Options as HostOptions, ServerOptions};
use wasmflow_collection_cli::ServerState;
use wasmflow_entity::Entity;
use wasmflow_interface::CollectionSignature;
use wasmflow_invocation::{InherentData, Invocation};
use wasmflow_manifest::host_definition::HostDefinition;
use wasmflow_mesh::{Mesh, NatsOptions};
use wasmflow_rpc::{RpcHandler, SharedRpcHandler};
use wasmflow_runtime::prelude::*;
use wasmflow_runtime::NetworkBuilder;
use wasmflow_transport::TransportMap;

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

/// A Wasmflow Host wraps a Wasmflow runtime with server functionality like persistence,.
#[must_use]
#[derive(Debug)]
pub struct Host {
  id: String,
  kp: KeyPair,
  network: Option<Network>,
  mesh: Option<Arc<Mesh>>,
  manifest: HostDefinition,
  server_metadata: Option<ServerState>,
}

impl Host {
  /// Starts the host. This call is non-blocking, so it is up to the consumer
  /// to wait with a method like `host.wait_for_sigint()`.
  pub async fn start(&mut self, seed: Option<u64>) -> Result<()> {
    debug!("host starting");

    self.mesh = self.get_mesh().await?;
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

  pub async fn connect_to_mesh(&mut self) -> Result<()> {
    self.mesh = self.get_mesh().await?;
    Ok(())
  }

  async fn get_mesh(&self) -> Result<Option<Arc<Mesh>>> {
    if let Some(config) = &self.manifest.host.mesh {
      if config.enabled {
        debug!(address=%config.address,"connecting to mesh");
        let mesh = Mesh::connect(NatsOptions {
          address: config.address.clone(),
          client_id: self.get_host_id().to_owned(),
          creds_path: config.creds_path.clone(),
          token: config.token.clone(),
          timeout: self.manifest.host.timeout,
        })
        .await?;
        Ok(Some(Arc::new(mesh)))
      } else {
        Ok(None)
      }
    } else {
      Ok(None)
    }
  }

  pub fn get_signature(&self) -> Result<CollectionSignature> {
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
    if let Some(mesh) = self.mesh {
      let _ = mesh.shutdown().await;
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
    let kp_seed = self.kp.seed()?;

    let mut network_builder = NetworkBuilder::from_definition(self.manifest.clone(), &kp_seed)?;
    if let Some(mesh) = &self.mesh {
      network_builder = network_builder.mesh(mesh.clone());
    }
    if let Some(seed) = seed {
      network_builder = network_builder.with_seed(seed);
    }
    network_builder = network_builder.namespace(self.get_host_id());
    network_builder = network_builder.allow_latest(self.manifest.host.allow_latest);
    network_builder = network_builder.allow_insecure(self.manifest.host.insecure_registries.clone());
    if let Some(mesh) = &self.mesh {
      network_builder = network_builder.mesh(mesh.clone());
    }

    let network = network_builder.build().await?;

    self.network = Some(network);
    Ok(())
  }

  async fn start_servers(&mut self) -> Result<ServerState> {
    let nuid = self.get_network_uid()?;

    #[allow(clippy::manual_map)]
    let options = HostOptions {
      rpc: match &self.manifest.host.rpc {
        Some(config) => Some(ServerOptions {
          port: config.port,
          address: config.address,
          pem: config.pem.clone(),
          key: config.key.clone(),
          ca: config.ca.clone(),
          enabled: config.enabled,
        }),
        None => None,
      },
      mesh: match &self.manifest.host.mesh {
        Some(config) => Some(MeshOptions {
          enabled: config.enabled,
          address: config.address.clone(),
          creds_path: config.creds_path.clone(),
          token: config.token.clone(),
        }),
        None => None,
      },
      id: self.get_host_id().to_owned(),
      timeout: self.manifest.host.timeout,
    };

    let collection = from_registry(nuid);

    let metadata = tokio::spawn(wasmflow_collection_cli::start_server(collection, Some(options)))
      .await
      .map_err(|e| Error::Other(format!("Join error: {}", e)))?
      .map_err(|e| Error::Other(format!("Socket error: {}", e)))?;

    Ok(metadata)
  }

  pub async fn request(
    &self,
    schematic: &str,
    payload: TransportMap,
    data: Option<InherentData>,
  ) -> Result<TransportStream> {
    match &self.network {
      Some(network) => {
        let invocation = Invocation::new(Entity::host(&self.id), Entity::local(schematic), payload, data);
        Ok(network.invoke(invocation).await?)
      }
      None => Err(crate::Error::InvalidHostState("No network available".into())),
    }
  }

  pub async fn exec_main(&self, argv: Vec<String>) -> Result<u32> {
    match &self.network {
      Some(network) => Ok(network.exec_main(argv).await),
      None => Err(crate::Error::InvalidHostState("No network available".into())),
    }
  }

  pub async fn invoke(&self, invocation: Invocation) -> Result<TransportStream> {
    match &self.network {
      Some(network) => Ok(network.invoke(invocation).await?),
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
    self.manifest.host.id.as_ref().unwrap_or(&self.id)
  }

  #[must_use]
  pub fn is_started(&self) -> bool {
    self.network.is_some()
  }
}

/// The HostBuilder builds the configuration for a Wasmflow Host.
#[must_use]
#[derive(Debug, Clone)]
pub struct HostBuilder {
  manifest: HostDefinition,
}

impl Default for HostBuilder {
  fn default() -> Self {
    Self::new()
  }
}

impl HostBuilder {
  /// Creates a new host builder.
  pub fn new() -> HostBuilder {
    HostBuilder {
      manifest: HostDefinition::default(),
    }
  }

  pub async fn from_manifest_url(location: &str, allow_latest: bool, insecure_registries: &[String]) -> Result<Self> {
    let manifest_src = wasmflow_loader::get_bytes(location, allow_latest, insecure_registries).await?;

    let manifest = HostDefinition::load_from_bytes(Some(location.to_owned()), &manifest_src)?;
    Ok(Self::from_definition(manifest))
  }

  pub fn from_definition(definition: HostDefinition) -> Self {
    HostBuilder { manifest: definition }
  }

  /// Constructs an instance of a Wasmflow host.
  pub fn build(self) -> Host {
    let kp = KeyPair::new_server();
    let host_id = kp.public_key();

    Host {
      kp,
      id: host_id,
      network: None,
      mesh: None,
      manifest: self.manifest,
      server_metadata: None,
    }
  }
}

impl TryFrom<PathBuf> for HostBuilder {
  type Error = Error;

  fn try_from(file: PathBuf) -> Result<Self> {
    let manifest = HostDefinition::load_from_file(&file)?;
    Ok(HostBuilder::from_definition(manifest))
  }
}

impl TryFrom<&str> for HostBuilder {
  type Error = Error;

  fn try_from(value: &str) -> Result<Self> {
    HostBuilder::try_from(PathBuf::from(value))
  }
}

#[cfg(test)]
mod test {
  use std::net::Ipv4Addr;
  use std::path::PathBuf;
  use std::str::FromStr;

  use http::Uri;
  use wasmflow_entity::Entity;
  use wasmflow_invocation_server::connect_rpc_client;
  use wasmflow_manifest::host_definition::HttpConfig;
  use wasmflow_rpc::rpc::Invocation;

  use super::*;
  use crate::{HostBuilder, Result};

  #[test]
  fn builds_default() {
    let _h = HostBuilder::new().build();
  }

  #[test_logger::test(tokio::test)]
  async fn should_start_and_stop() -> Result<()> {
    let mut host = HostBuilder::new().build();
    host.start(None).await?;
    assert!(host.is_started());
    host.stop().await;

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn request_direct() -> Result<()> {
    let file = PathBuf::from("manifests/logger.yaml");
    let manifest = HostDefinition::load_from_file(&file)?;
    let mut host = HostBuilder::from_definition(manifest).build();
    host.start(None).await?;
    let passed_data = "logging output";
    let payload: TransportMap = vec![("input", passed_data)].into();
    let mut stream = host.request("logger", payload, None).await?;
    let mut messages: Vec<_> = stream.drain_port("output").await?;
    assert_eq!(messages.len(), 1);
    let output = messages.pop().unwrap();
    let result: String = output.payload.deserialize()?;
    assert_eq!(result, passed_data);
    host.stop().await;

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn request_rpc_server() -> Result<()> {
    let file = PathBuf::from("manifests/logger.yaml");
    let mut def = HostDefinition::load_from_file(&file)?;
    def.host.rpc = Some(HttpConfig {
      enabled: true,
      port: None,
      address: Some(Ipv4Addr::from_str("127.0.0.1").unwrap()),
      ..Default::default()
    });

    let mut host = HostBuilder::from_definition(def).build();
    host.start(None).await?;
    let address = host.rpc_address().unwrap();
    println!("rpc server bound to : {}", address);

    let mut client = connect_rpc_client(Uri::from_str(&format!("http://{}", address)).unwrap()).await?;
    let passed_data = "logging output";
    let data = vec![("input", passed_data)].into();
    let invocation: Invocation = super::Invocation::new(Entity::test("test"), Entity::local("logger"), data, None)
      .try_into()
      .unwrap();
    let mut response = client.invoke(invocation).await.unwrap().into_inner();
    let next = response.message().await;
    println!("next: {:?}", next);
    let next = next.unwrap().unwrap();
    debug!(?next);

    host.stop().await;

    Ok(())
  }
}