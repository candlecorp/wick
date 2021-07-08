use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use tonic::transport::{
  Certificate,
  Channel,
  ClientTlsConfig,
  Identity,
  Uri,
};
use vino_rpc::rpc::invocation_service_client::InvocationServiceClient;

use crate::Result;

pub(crate) async fn rpc_client(
  address: Ipv4Addr,
  port: u16,
  pem: Option<PathBuf>,
  key: Option<PathBuf>,
  ca: Option<PathBuf>,
  domain: Option<String>,
) -> Result<InvocationServiceClient<Channel>> {
  let url = format!("https://{}:{}", address, port);
  let uri = Uri::from_str(&url)
    .map_err(|_| crate::Error::Other(format!("Could not create URI from: {}", url)))?;
  let channel = if let (Some(pem), Some(key), Some(ca), Some(domain)) = (pem, key, ca, domain) {
    let ca_pem = std::fs::read_to_string(ca)?;
    let client_pem = std::fs::read_to_string(pem)?;
    let client_key = std::fs::read_to_string(key)?;
    let id = Identity::from_pem(client_pem.as_bytes(), client_key.as_bytes());
    let ca = Certificate::from_pem(ca_pem.as_bytes());

    let tls = ClientTlsConfig::new()
      .domain_name(domain)
      .identity(id)
      .ca_certificate(ca);

    Channel::builder(uri)
      .tls_config(tls)?
      .timeout(Duration::from_secs(5))
      .rate_limit(5, Duration::from_secs(1))
      .concurrency_limit(256)
      .connect()
      .await?
  } else {
    Channel::builder(uri)
      .timeout(Duration::from_secs(5))
      .rate_limit(5, Duration::from_secs(1))
      .concurrency_limit(256)
      .connect()
      .await?
  };
  Ok(InvocationServiceClient::new(channel))
}
