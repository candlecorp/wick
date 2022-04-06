use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::PathBuf;
use std::sync::Arc;

use nkeys::KeyPair;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use uuid::Uuid;
use vino_entity::Entity;
use vino_lattice::{Lattice, NatsOptions};
use vino_manifest::host_definition::HostDefinition;
use vino_provider::native::prelude::ProviderSignature;
use vino_provider_cli::options::{LatticeOptions, Options as HostOptions, ServerOptions};
use vino_provider_cli::ServerState;
use vino_random::Seed;
use vino_rpc::{RpcHandler, SharedRpcHandler};
use vino_runtime::prelude::*;
use vino_runtime::NetworkBuilder;
use vino_transport::{InherentData, Invocation, TransportMap};

use crate::{Error, Result};

type ServiceMap = HashMap<Uuid, SharedRpcHandler>;
static HOST_REGISTRY: Lazy<Mutex<ServiceMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn from_registry(id: Uuid) -> Arc<dyn RpcHandler + Send + Sync + 'static> {
  let mut registry = HOST_REGISTRY.lock();
  let provider = registry.entry(id).or_insert_with(|| Arc::new(NetworkProvider::new(id)));
  provider.clone()
}

/// A Vino Host wraps a Vino runtime with server functionality like persistence,.
#[must_use]
#[derive(Debug)]
pub struct Host {
  id: String,
  kp: KeyPair,
  network: Option<Network>,
  lattice: Option<Arc<Lattice>>,
  manifest: HostDefinition,
  server_metadata: Option<ServerState>,
}

impl Host {
  /// Starts the host. This call is non-blocking, so it is up to the consumer
  /// to wait with a method like `host.wait_for_sigint()`.
  pub async fn start(&mut self, seed: Option<u64>) -> Result<()> {
    debug!("host starting");

    self.lattice = self.get_lattice().await?;
    self.start_network(seed.map(Seed::unsafe_new)).await?;
    let state = self.start_servers().await?;
    self.server_metadata = Some(state);

    Ok(())
  }

  pub async fn connect_to_lattice(&mut self) -> Result<()> {
    self.lattice = self.get_lattice().await?;
    Ok(())
  }

  async fn get_lattice(&self) -> Result<Option<Arc<Lattice>>> {
    if let Some(config) = &self.manifest.host.lattice {
      if config.enabled {
        debug!(address=%config.address,"connecting to lattice");
        let lattice = Lattice::connect(NatsOptions {
          address: config.address.clone(),
          client_id: self.get_host_id().to_owned(),
          creds_path: config.creds_path.clone(),
          token: config.token.clone(),
          timeout: self.manifest.host.timeout,
        })
        .await?;
        Ok(Some(Arc::new(lattice)))
      } else {
        Ok(None)
      }
    } else {
      Ok(None)
    }
  }

  pub fn get_signature(&self) -> Result<ProviderSignature> {
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
    if let Some(lattice) = self.lattice {
      let _ = lattice.shutdown().await;
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
    if let Some(lattice) = &self.lattice {
      network_builder = network_builder.lattice(lattice.clone());
    }
    if let Some(seed) = seed {
      network_builder = network_builder.with_seed(seed);
    }
    network_builder = network_builder.namespace(self.get_host_id());
    network_builder = network_builder.allow_latest(self.manifest.host.allow_latest);
    network_builder = network_builder.allow_insecure(self.manifest.host.insecure_registries.clone());
    if let Some(lattice) = &self.lattice {
      network_builder = network_builder.lattice(lattice.clone());
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
      http: match &self.manifest.host.http {
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
      lattice: match &self.manifest.host.lattice {
        Some(config) => Some(LatticeOptions {
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

    let provider = from_registry(nuid);

    let metadata = tokio::spawn(vino_provider_cli::start_server(provider, Some(options)))
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
        let invocation = Invocation::new(Entity::host(&self.id), Entity::schematic(schematic), payload, data);
        Ok(network.invoke(invocation).await?)
      }
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

/// The HostBuilder builds the configuration for a Vino Host.
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
    let manifest_src = vino_loader::get_bytes(location, allow_latest, insecure_registries).await?;

    let manifest = HostDefinition::load_from_bytes(Some(location.to_owned()), &manifest_src)?;
    Ok(Self::from_definition(manifest))
  }

  pub fn from_definition(definition: HostDefinition) -> Self {
    HostBuilder { manifest: definition }
  }

  /// Constructs an instance of a Vino host.
  pub fn build(self) -> Host {
    let kp = KeyPair::new_server();
    let host_id = kp.public_key();

    Host {
      kp,
      id: host_id,
      network: None,
      lattice: None,
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
  use vino_entity::Entity;
  use vino_invocation_server::connect_rpc_client;
  use vino_manifest::host_definition::HttpConfig;
  use vino_rpc::rpc::Invocation;

  use super::*;
  use crate::{HostBuilder, Result};

  #[test]
  fn builds_default() {
    let _h = HostBuilder::new().build();
  }

  #[test_logger::test(tokio::test)]
  async fn should_start_and_stop() -> Result<()> {
    let mut host = HostBuilder::new().build();
    host.start(Some(0)).await?;
    assert!(host.is_started());
    host.stop().await;

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn request_direct() -> Result<()> {
    let file = PathBuf::from("manifests/logger.yaml");
    let manifest = HostDefinition::load_from_file(&file)?;
    let mut host = HostBuilder::from_definition(manifest).build();
    host.start(Some(0)).await?;
    let passed_data = "logging output";
    let payload: TransportMap = vec![("input", passed_data)].into();
    let mut stream = host.request("logger", payload, None).await?;
    let mut messages: Vec<_> = stream.collect_port("output").await;
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
      port: Some(54321),
      address: Some(Ipv4Addr::from_str("127.0.0.1").unwrap()),
      ..Default::default()
    });

    let mut host = HostBuilder::from_definition(def).build();
    host.start(Some(0)).await?;

    let mut client = connect_rpc_client(Uri::from_str("https://127.0.0.1:54321").unwrap()).await?;
    let passed_data = "logging output";
    let data = vec![("input", passed_data)].into();
    let invocation: Invocation =
      vino_transport::Invocation::new(Entity::test("test"), Entity::schematic("logger"), data, None)
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
