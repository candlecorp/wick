use std::collections::HashMap;
use std::convert::{
  TryFrom,
  TryInto,
};
use std::path::PathBuf;
use std::sync::Arc;

use nkeys::KeyPair;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use vino_entity::Entity;
use vino_lattice::lattice::Lattice;
use vino_lattice::nats::NatsOptions;
use vino_manifest::host_definition::HostDefinition;
use vino_provider::native::prelude::ProviderSignature;
use vino_provider_cli::cli::{
  LatticeOptions,
  Options as HostOptions,
  ServerOptions,
  ServerState,
};
use vino_rpc::{
  RpcHandler,
  SharedRpcHandler,
};
use vino_runtime::core_data::InitData;
use vino_runtime::network::NetworkBuilder;
use vino_runtime::prelude::*;
use vino_transport::TransportMap;

use crate::{
  Error,
  Result,
};

type ServiceMap = HashMap<String, SharedRpcHandler>;
static HOST_REGISTRY: Lazy<Mutex<ServiceMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn from_registry(id: &str) -> Arc<dyn RpcHandler + Send + Sync + 'static> {
  trace!("HOST:PROV:NETWORK:GET:{}", id);
  let mut registry = HOST_REGISTRY.lock();
  let provider = registry.entry(id.to_owned()).or_insert_with(|| {
    trace!("HOST:PROV:NETWORK:CREATE:{}", id);
    Arc::new(NetworkProvider::new(id.to_owned()))
  });
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
  pub async fn start(&mut self) -> Result<()> {
    debug!("Host starting");

    self.lattice = self.get_lattice().await?;
    self.start_network().await?;
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
        debug!("Connecting to lattice at {}", config.address);
        let lattice = Lattice::connect(NatsOptions {
          address: config.address.clone(),
          client_id: self.get_host_id(),
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
  pub async fn stop(&mut self) {
    debug!("Host stopping");
    self.lattice = None;
    self.network = None;
    self.server_metadata = None;
    // TODO: Need to gracefully shutdown started servers.
  }

  pub fn get_network(&self) -> Result<&Network> {
    self.network.as_ref().ok_or(Error::NoNetwork)
  }

  pub fn get_network_uid(&self) -> Result<String> {
    self
      .network
      .as_ref()
      .ok_or(Error::NoNetwork)
      .map(|network| network.uid.clone())
  }

  pub async fn start_network(&mut self) -> Result<()> {
    ensure!(
      self.network.is_none(),
      crate::Error::InvalidHostState("Host already has a network running".into())
    );
    let seed = self.kp.seed()?;

    let mut network_builder =
      NetworkBuilder::from_definition(self.manifest.network.clone(), &seed)?;
    if let Some(lattice) = &self.lattice {
      network_builder = network_builder.lattice(lattice.clone());
    }
    network_builder = network_builder.allow_latest(self.manifest.host.allow_latest);
    network_builder =
      network_builder.allow_insecure(self.manifest.host.insecure_registries.clone());
    if let Some(lattice) = &self.lattice {
      network_builder = network_builder.lattice(lattice.clone());
    }

    let network = network_builder.build();
    network.init().await?;
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
      id: self
        .manifest
        .host
        .id
        .clone()
        .unwrap_or_else(|| self.get_host_id()),
      timeout: self.manifest.host.timeout,
    };

    let provider = from_registry(&nuid);

    let metadata = tokio::spawn(vino_provider_cli::start_server(provider, Some(options)))
      .await
      .map_err(|e| Error::Other(format!("Join error: {}", e)))?
      .map_err(|e| Error::Other(format!("Socket error: {}", e)))?;

    Ok(metadata)
  }

  pub async fn request<T, U>(
    &self,
    schematic: T,
    payload: U,
    data: Option<InitData>,
  ) -> Result<TransportStream>
  where
    T: AsRef<str> + Sync + Send,
    U: TryInto<TransportMap> + Send + Sync,
  {
    match &self.network {
      Some(network) => Ok(
        network
          .request_with_data(schematic, Entity::host(&self.id), payload, data)
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

  #[must_use]
  pub fn get_host_id(&self) -> String {
    self.id.clone()
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

  pub fn from_definition(definition: HostDefinition) -> Self {
    HostBuilder {
      manifest: definition,
      ..Default::default()
    }
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
  use vino_entity::entity::Entity;
  use vino_invocation_server::connect_rpc_client;
  use vino_manifest::host_definition::HttpConfig;
  use vino_rpc::convert_transport_map;
  use vino_rpc::rpc::Invocation;

  use super::*;
  use crate::{
    HostBuilder,
    Result,
  };

  #[test]
  fn builds_default() {
    let _h = HostBuilder::new().build();
  }

  #[test_logger::test(tokio::test)]
  async fn should_start_and_stop() -> Result<()> {
    let mut host = HostBuilder::new().build();
    host.start().await?;
    host.stop().await;

    assert!(!host.is_started());
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn request_direct() -> Result<()> {
    let file = PathBuf::from("manifests/logger.yaml");
    let manifest = HostDefinition::load_from_file(&file)?;
    let mut host = HostBuilder::from_definition(manifest).build();
    host.start().await?;
    let passed_data = "logging output";
    let payload: TransportMap = vec![("input", passed_data)].into();
    let mut stream = host.request("logger", payload, None).await?;
    let mut messages: Vec<_> = stream.collect_port("output").await;
    assert_eq!(messages.len(), 1);
    let output = messages.pop().unwrap();
    let result: String = output.payload.try_into()?;
    assert_eq!(result, passed_data);
    host.stop().await;

    assert!(!host.is_started());
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
    host.start().await?;

    let mut client = connect_rpc_client(Uri::from_str("https://127.0.0.1:54321").unwrap()).await?;
    let passed_data = "logging output";
    let data = vec![("input", passed_data)].into();
    let mut response = client
      .invoke(Invocation {
        origin: Entity::test("test").url(),
        target: Entity::schematic("logger").url(),
        msg: convert_transport_map(data),
        id: "some inv".to_owned(),
      })
      .await
      .unwrap()
      .into_inner();
    let next = response.message().await;
    println!("next: {:?}", next);
    let next = next.unwrap().unwrap();
    debug!("result: {:?}", next);

    host.stop().await;

    assert!(!host.is_started());
    Ok(())
  }
}
