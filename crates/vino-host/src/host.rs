use std::collections::HashMap;
use std::sync::{
  Arc,
  RwLock,
};

use nkeys::KeyPair;
use serde::Serialize;
use vino_entity::Entity;
use vino_lattice::lattice::Lattice;
use vino_lattice::nats::NatsOptions;
use vino_manifest::host_definition::HostDefinition;
use vino_provider_cli::cli::{
  LatticeOptions,
  Options as HostOptions,
  ServerMetadata,
  ServerOptions,
};
use vino_runtime::network::NetworkBuilder;
use vino_runtime::prelude::*;

use crate::{
  Error,
  Result,
};

/// A Vino Host wraps a Vino runtime with server functionality like persistence,.
#[must_use]
#[derive(Debug)]
pub struct Host {
  started: RwLock<bool>,
  id: String,
  kp: KeyPair,
  network: Option<Network>,
  lattice: Option<Arc<Lattice>>,
  manifest: HostDefinition,
  server_metadata: Option<ServerMetadata>,
}

impl Host {
  /// Starts the host. This call is non-blocking, so it is up to the consumer
  /// to wait with a method like `host.wait_for_sigint()`.
  pub async fn start(&mut self) -> Result<()> {
    debug!("Host starting");
    *self.started.write().unwrap() = true;

    if let Some(lconfig) = &self.manifest.host.lattice {
      if lconfig.enabled {
        info!("Connecting to lattice at {}", lconfig.address);
        let lattice = Lattice::connect(NatsOptions {
          address: lconfig.address.clone(),
          client_id: self.get_host_id(),
          creds_path: lconfig.creds_path.clone(),
          token: lconfig.token.clone(),
          timeout: self.manifest.host.timeout,
        })
        .await?;
        self.lattice = Some(Arc::new(lattice));
      }
    }

    self.start_network().await?;
    let metadata = self.start_servers().await?;
    self.server_metadata = Some(metadata);

    Ok(())
  }

  pub fn get_server_info(&self) -> &Option<ServerMetadata> {
    &self.server_metadata
  }

  /// Stops a running host.
  pub async fn stop(&self) {
    debug!("Host stopping");
    *self.started.write().unwrap() = false;
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

  async fn start_network(&mut self) -> Result<()> {
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

    let network = network_builder.build();
    network.init().await?;
    self.network = Some(network);
    Ok(())
  }

  async fn start_servers(&mut self) -> Result<ServerMetadata> {
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

    let metadata = tokio::spawn(vino_provider_cli::start_server(
      Box::new(move || Box::new(NetworkProvider::new(nuid.clone()))),
      Some(options),
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
  ) -> Result<TransportStream> {
    match &self.network {
      Some(network) => Ok(
        network
          .request(schematic, Entity::host(&self.id), &payload)
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
    self.id.clone()
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

/// The HostBuilder builds the configuration for a Vino Host.
#[must_use]
#[derive(Debug, Clone)]
pub struct HostBuilder {
  pk: String,
  id: String,

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
    let kp = KeyPair::new_server();
    let id = kp.public_key();

    HostBuilder {
      id,
      pk: kp.public_key(),
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
      started: RwLock::new(false),
      network: None,
      lattice: None,
      manifest: self.manifest,
      server_metadata: None,
    }
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
  use vino_invocation_server::make_rpc_client;
  use vino_macros::transport_map;
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
  #[test_logger::test(actix::test)]
  async fn should_start_and_stop() -> Result<()> {
    let mut host = HostBuilder::new().build();
    host.start().await?;
    host.stop().await;

    assert!(!host.is_started());
    Ok(())
  }

  #[test_logger::test(actix::test)]
  async fn ensure_started() -> Result<()> {
    let mut host = HostBuilder::new().build();
    host.start().await?;
    host._ensure_started()?;
    host.stop().await;

    assert!(!host.is_started());
    Ok(())
  }

  #[test_logger::test(actix::test)]
  async fn request_from_network() -> Result<()> {
    let file = PathBuf::from("src/configurations/logger.yaml");
    let manifest = HostDefinition::load_from_file(&file)?;
    let mut host = HostBuilder::from_definition(manifest).build();
    host.start().await?;
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

  #[test_logger::test(actix::test)]
  async fn request_from_rpc_server() -> Result<()> {
    let file = PathBuf::from("src/configurations/logger.yaml");
    let mut def = HostDefinition::load_from_file(&file)?;
    def.host.rpc = Some(HttpConfig {
      enabled: true,
      port: Some(54321),
      address: Some(Ipv4Addr::from_str("127.0.0.1").unwrap()),
      ..Default::default()
    });

    let mut host = HostBuilder::from_definition(def).build();
    host.start().await?;

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
