use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use flow_component::SharedComponent;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use seeded_random::Seed;
use tracing::Span;
use uuid::Uuid;
use wick_component_cli::options::{Options as HostOptions, ServerOptions};
use wick_component_cli::ServerState;
use wick_config::config::ComponentConfiguration;
use wick_interface_types::ComponentSignature;
use wick_packet::{Entity, GenericConfig, InherentData, Invocation, PacketStream};
use wick_runtime::{EngineComponent, Runtime, RuntimeBuilder};

use crate::{Error, Result};

type ServiceMap = HashMap<Uuid, SharedComponent>;
static HOST_REGISTRY: Lazy<Mutex<ServiceMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn from_registry(id: Uuid) -> SharedComponent {
  let mut registry = HOST_REGISTRY.lock();
  let collection = registry.entry(id).or_insert_with(|| Arc::new(EngineComponent::new(id)));
  collection.clone()
}

/// A Wick Host wraps a Wick runtime with server functionality like persistence,.
#[must_use]
#[derive(Debug, derive_builder::Builder)]
#[builder(derive(Debug), setter(into))]
pub struct ComponentHost {
  #[builder(default = "Uuid::new_v4().to_string()")]
  id: String,
  #[builder(default)]
  runtime: Option<Runtime>,
  #[builder(default)]
  manifest: ComponentConfiguration,
  #[builder(default)]
  config: Option<GenericConfig>,
  #[builder(default, setter(strip_option))]
  server_metadata: Option<ServerState>,
  #[builder(default = "tracing::Span::current()")]
  span: Span,
}

impl ComponentHost {
  /// Starts the host. This call is non-blocking, so it is up to the consumer
  /// to wait with a method like `host.wait_for_sigint()`.
  pub async fn start(&mut self, seed: Option<u64>) -> Result<()> {
    self.span.in_scope(|| debug!("host starting"));

    // self.mesh = self.get_mesh().await?;
    self.start_engine(seed.map(Seed::unsafe_new)).await?;
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
    match &self.runtime {
      Some(engine) => Ok(engine.get_signature()?),
      None => Err(Error::NoEngine),
    }
  }

  #[must_use]
  pub fn get_server_info(&self) -> &Option<ServerState> {
    &self.server_metadata
  }

  /// Stops a running host.
  pub async fn stop(self) {
    self.span.in_scope(|| debug!("host stopping"));
    if let Some(engine) = self.runtime {
      let _ = engine.shutdown().await;
    }
  }

  pub fn get_engine(&self) -> Result<&Runtime> {
    self.runtime.as_ref().ok_or(Error::NoEngine)
  }

  pub fn get_engine_uid(&self) -> Result<Uuid> {
    self.runtime.as_ref().ok_or(Error::NoEngine).map(|engine| engine.uid)
  }

  pub async fn start_engine(&mut self, seed: Option<Seed>) -> Result<()> {
    ensure!(
      self.runtime.is_none(),
      crate::Error::InvalidHostState("Host already has a engine running".into())
    );

    let mut engine_builder = RuntimeBuilder::from_definition(self.manifest.clone());
    let span = debug_span!("host");
    span.follows_from(&self.span);
    engine_builder = engine_builder.span(span);
    engine_builder = engine_builder.namespace(self.get_host_id());
    engine_builder = engine_builder.config(self.config.clone());
    engine_builder = engine_builder.allow_latest(self.manifest.allow_latest());
    if let Some(insecure) = self.manifest.insecure_registries() {
      engine_builder = engine_builder.allowed_insecure(insecure.to_vec());
    }

    let engine = engine_builder.build(seed).await?;

    self.runtime = Some(engine);
    Ok(())
  }

  async fn start_servers(&mut self) -> Result<ServerState> {
    let nuid = self.get_engine_uid()?;

    let host_config = self.manifest.host().cloned().unwrap_or_default();

    #[allow(clippy::manual_map)]
    let options = HostOptions {
      rpc: host_config.rpc().map(|config| ServerOptions {
        port: config.port(),
        address: config.address().copied(),
        pem: config.pem().cloned(),
        key: config.key().cloned(),
        ca: config.ca().cloned(),
        enabled: config.enabled(),
      }),
      id: self.get_host_id().to_owned(),
      timeout: *host_config.timeout(),
    };

    let collection = from_registry(nuid);

    let metadata = tokio::spawn(wick_component_cli::start_server(collection, Some(options)))
      .await
      .map_err(|e| Error::Other(format!("Join error: {}", e)))?
      .map_err(|e| Error::Other(format!("Socket error: {}", e)))?;

    Ok(metadata)
  }

  pub async fn request(&self, operation: &str, stream: PacketStream, data: InherentData) -> Result<PacketStream> {
    match &self.runtime {
      Some(runtime) => {
        let invocation = Invocation::new(
          Entity::server(&self.id),
          Entity::operation(&self.id, operation),
          stream,
          data,
          &self.span,
        );
        Ok(runtime.invoke(invocation, None).await?)
      }
      None => Err(crate::Error::InvalidHostState("No engine available".into())),
    }
  }

  pub async fn invoke(&self, invocation: Invocation, data: Option<GenericConfig>) -> Result<PacketStream> {
    match &self.runtime {
      Some(runtime) => Ok(runtime.invoke(invocation, data).await?),
      None => Err(crate::Error::InvalidHostState("No engine available".into())),
    }
  }

  pub async fn wait_for_sigint(&self) -> Result<()> {
    tokio::signal::ctrl_c().await.unwrap();
    self.span.in_scope(|| debug!("SIGINT received"));
    Ok(())
  }

  #[must_use]
  pub fn get_host_id(&self) -> &str {
    &self.id
  }

  #[must_use]
  pub fn is_started(&self) -> bool {
    self.runtime.is_some()
  }
}

impl ComponentHostBuilder {
  /// Creates a new host builder.
  #[must_use]
  pub fn new() -> ComponentHostBuilder {
    ComponentHostBuilder::default()
  }
}

#[cfg(test)]
mod test {
  use std::net::Ipv4Addr;
  use std::path::PathBuf;
  use std::str::FromStr;

  use anyhow::Result;
  use futures::StreamExt;
  use http::Uri;
  use wick_config::config::HttpConfigBuilder;
  use wick_config::WickConfiguration;
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
    let mut host = ComponentHostBuilder::new().build()?;

    host.start(None).await?;
    assert!(host.is_started());
    host.stop().await;

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn request_direct() -> Result<()> {
    let file = PathBuf::from("manifests/logger.yaml");
    let manifest = WickConfiguration::load_from_file(&file).await?.try_component_config()?;
    let mut host = ComponentHostBuilder::default().manifest(manifest).build()?;
    host.start(None).await?;
    let passed_data = "logging output";
    let stream = packet_stream!(("input", passed_data));
    let stream = host.request("logger", stream, InherentData::unsafe_default()).await?;

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

    def.host_mut().rpc_mut().replace(
      HttpConfigBuilder::default()
        .enabled(true)
        .address(Ipv4Addr::from_str("127.0.0.1").unwrap())
        .build()?,
    );

    let mut host = ComponentHostBuilder::default().manifest(def).build()?;
    host.start(None).await?;
    let address = host.rpc_address().unwrap();
    println!("rpc server bound to : {}", address);

    let mut client = connect_rpc_client(Uri::from_str(&format!("http://{}", address)).unwrap()).await?;
    println!("connected to server");
    let passed_data = "logging output";
    let packets = packets![("input", passed_data)];
    let invocation: wick_rpc::rpc::Invocation = Invocation::test("test", Entity::local("logger"), Vec::new(), None)?
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
