use std::collections::HashMap;
use std::fmt::Display;
// use std::net::{
//   Ipv4Addr,
//   SocketAddr,
// };
use std::sync::RwLock;

use actix::prelude::*;
use serde::Serialize;
// use tokio::sync::Mutex;
// use vino_provider_cli::cli::Options as CliOpts;
use vino_runtime::{
  Network,
  NetworkDefinition,
};
use vino_transport::MessageTransport;

use crate::Result;

/// A Vino Host wraps a Vino runtime with server functionality like persistence,
#[derive(Debug)]
pub struct Host {
  pub(crate) host_id: String,
  pub(crate) seed: String,
  pub(crate) started: RwLock<bool>,
  pub(crate) network: Option<Addr<Network>>,
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
    if let Some(system) = System::try_current() {
      system.stop();
    }
  }

  pub async fn start_network(&mut self, def: NetworkDefinition) -> Result<()> {
    ensure!(
      self.network.is_none(),
      crate::Error::InvalidHostState("Host already has a network running".into())
    );
    let network = Network::from_registry();
    self.network = Some(network.clone());
    network
      .send(vino_runtime::network::Initialize {
        network_id: self.host_id.to_string(),
        seed: self.seed.to_string(),
        network: def,
      })
      .await??;
    Ok(())
  }

  // pub async fn start_rpc_server(
  //   &mut self,
  //   address: Ipv4Addr,
  //   port: Option<u16>,
  // ) -> Result<SocketAddr> {
  //   let addr = tokio::spawn(vino_provider_cli::init(
  //     Arc::new(Mutex::new(crate::host_provider::Provider::default())),
  //     Some(CliOpts { port, address }),
  //   ))
  //   .await
  //   .map_err(|e| Error::Other(format!("Join error: {}", e)))?
  //   .map_err(|e| Error::Other(format!("Socket error: {}", e)))?;
  //   Ok(addr)
  // }

  pub async fn request<T: AsRef<str> + Display>(
    &self,
    schematic: &str,
    payload: HashMap<T, impl Serialize>,
  ) -> Result<HashMap<String, MessageTransport>> {
    match &self.network {
      Some(network) => vino_runtime::request(network, schematic, payload)
        .await
        .map_err(crate::Error::VinoError),
      None => Err(crate::Error::InvalidHostState(
        "No network available".into(),
      )),
    }
  }

  pub async fn wait_for_sigint(&self) -> Result<()> {
    actix_rt::signal::ctrl_c().await.unwrap();
    debug!("SIGINT received");
    Ok(())
  }

  fn _ensure_started(&self) -> Result<()> {
    ensure!(
      *self.started.read().unwrap(),
      crate::Error::InvalidHostState("Host not started".into())
    );
    ensure!(System::try_current().is_some(), "No actix rt system found.");
    Ok(())
  }

  pub fn is_started(&self) -> bool {
    *self.started.read().unwrap()
  }
}

#[cfg(test)]
mod test {
  use std::collections::HashMap;
  // use std::net::Ipv4Addr;
  use std::path::PathBuf;

  // use std::str::FromStr;
  use maplit::hashmap;
  use vino_codec::messagepack::deserialize;
  use vino_transport::MessageTransport;

  use crate::host_definition::HostDefinition;
  use crate::{
    HostBuilder,
    Result,
  };

  #[test_env_log::test(actix_rt::test)]
  async fn should_start_and_stop() -> Result<()> {
    let host = HostBuilder::new().start().await?;
    host.stop().await;

    assert!(!host.is_started());
    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn ensure_started() -> Result<()> {
    let host = HostBuilder::new().start().await?;
    host._ensure_started()?;
    host.stop().await;

    assert!(!host.is_started());
    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn request_from_network() -> Result<()> {
    let mut host = HostBuilder::new().start().await?;
    let file = PathBuf::from("src/configurations/logger.yaml");
    let manifest = HostDefinition::load_from_file(&file)?;
    host.start_network(manifest.network).await?;
    let passed_data = "logging output";
    let data: HashMap<&str, &str> = hashmap! {
        "input" => passed_data,
    };
    let mut result = host.request("logger", data).await?;
    let output = result.remove("output").unwrap();
    if let MessageTransport::MessagePack(bytes) = output {
      let output = deserialize::<String>(&bytes)?;
      assert_eq!(output, passed_data.to_string());
    } else {
      panic!();
    }
    host.stop().await;

    assert!(!host.is_started());
    Ok(())
  }

  // #[test_env_log::test(actix_rt::test)]
  // async fn request_from_rpc_server() -> Result<()> {
  //   let mut host = HostBuilder::new().start().await?;
  //   let file = PathBuf::from("src/configurations/logger.yaml");
  //   let manifest = HostDefinition::load_from_file(&file)?;
  //   host.start_network(manifest.network).await?;
  //   host
  //     .start_rpc_server(Ipv4Addr::from_str("127.0.0.1").unwrap(), Some(54321))
  //     .await;
  //   let passed_data = "logging output";
  //   let data: HashMap<&str, &str> = hashmap! {
  //       "input" => passed_data,
  //   };
  //   let mut result = host.request("logger", data).await?;
  //   let output = result.remove("output").unwrap();
  //   if let MessageTransport::MessagePack(bytes) = output {
  //     let output = deserialize::<String>(&bytes)?;
  //     assert_eq!(output, passed_data.to_string());
  //   } else {
  //     panic!();
  //   }
  //   host.stop().await;

  //   assert!(!host.is_started());
  //   Ok(())
  // }
}
