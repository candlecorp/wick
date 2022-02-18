use std::collections::HashMap;
use std::io::Read;
use std::net::Ipv4Addr;
use std::path::Path;
use std::process::Stdio;
use std::str::FromStr;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use rand::{thread_rng, Rng};
use tokio::process;
use vino_loader::cache_location;
use vino_provider::native::prelude::*;
use vino_provider_cli::options::env;
use vino_rpc::error::RpcError;
use vino_rpc::{RpcClient, RpcHandler, RpcResult};

use crate::Error;

#[derive(Debug, Default)]
pub struct Context {
  pub documents: HashMap<String, String>,
  pub collections: HashMap<String, Vec<String>>,
}

#[derive(Debug)]
pub struct Provider {
  namespace: String,
  client: RpcClient,
  interface: ProviderSignature,
  #[allow(unused)]
  child: process::Child,
}

impl Provider {
  pub async fn from_tarbytes<T, NAME, REF>(namespace: NAME, reference: REF, bytes: T) -> Result<Self, Error>
  where
    T: Read + Send,
    NAME: AsRef<str> + Send,
    REF: AsRef<str> + Send,
  {
    let cachedir = cache_location("par", reference.as_ref());
    unpack(bytes, &cachedir)?;
    let interface_path = cachedir.join("interface.json");
    let binpath = cachedir.join("main.bin");
    let interface = get_interface(&interface_path).await?;
    let (cmd, connection) = start_bin(&binpath).await?;
    Ok(Self {
      namespace: namespace.as_ref().to_owned(),
      child: cmd,
      interface,
      client: connection,
    })
  }

  pub fn get_interface(&self) -> &ProviderSignature {
    &self.interface
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    let entity_url = entity.url();
    trace!("PROV:PAR:INVOKE:[{}]", entity_url);

    let start = Instant::now();

    let stream = self
      .client
      .clone()
      .invoke(self.namespace.clone(), entity.name(), payload)
      .await
      .map_err(|e| RpcError::ComponentError(e.to_string()))?;

    trace!(
      "PROV:PAR:INVOKE:[{}]:DURATION[{} ms]",
      entity_url,
      start.elapsed().as_millis()
    );
    Ok(Box::pin(stream))
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    Ok(vec![HostedType::Provider(self.interface.clone())])
  }
}

async fn get_interface(path: &Path) -> Result<ProviderSignature, Error> {
  let json = tokio::fs::read_to_string(path).await?;
  serde_json::from_str(&json).map_err(|e| Error::JsonError(e.to_string()))
}

async fn start_bin(path: &Path) -> Result<(process::Child, RpcClient), Error> {
  let local_addr = Ipv4Addr::from_str("127.0.0.1").unwrap();

  let mut iterations = 0;
  let (child, connection) = loop {
    let port: u16 = thread_rng().gen_range(40000..45000);
    let mut child = tokio::process::Command::new(path)
      .kill_on_drop(true)
      .env_clear()
      .envs([
        (env::VINO_RPC_PORT, port.to_string()),
        (env::VINO_RPC_ENABLED, "true".to_owned()),
      ])
      .stdin(Stdio::null())
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .spawn()?;

    tokio::time::sleep(Duration::from_millis(200)).await;
    let uri = format!("http://{}:{}", local_addr, port);
    if let Ok(connection) = vino_rpc::make_rpc_client(uri, None, None, None, None).await {
      trace!("PROV:PAR:CONNECTED");
      break Ok((child, connection));
    } else if child.try_wait().is_ok() {
      trace!("PROV:PAR:PROCESS_EXITED");
      // try again with a different port
    } else {
      trace!("PROV:PAR:WAIT");
      // still running, wait a little longer
      tokio::time::sleep(Duration::from_millis(1000)).await;
    };
    iterations += 1;
    if iterations > 10 {
      break Err(crate::Error::Other("Provider archive failed to load".to_owned()));
    }
  }?;

  Ok((child, connection))
}

fn unpack<T: Read + Send>(archive: T, dest: &Path) -> Result<(), Error> {
  trace!("PROV:PAR:UNPACK[dir={}]", dest.to_string_lossy());
  let mut archive = tar::Archive::new(archive);
  archive.unpack(dest)?;
  vino_par::validate_provider_dir(dest)?;

  Ok(())
}

#[cfg(test)]
mod tests {

  use anyhow::Result;
  use tokio_stream::StreamExt;
  use vino_par::make_archive;
  use vino_wascap::KeyPair;

  use super::*;

  #[test_logger::test(tokio::test)]
  async fn test_local_tar() -> Result<()> {
    let provider_bin = workspace_root::workspace_root()?
      .join("crates")
      .join("vino")
      .join("vino-provider-par")
      .join("test")
      .join("vino-standalone");
    debug!(
      "Creating provider archive with binary from: {}",
      provider_bin.to_string_lossy()
    );

    let bin_bytes = std::fs::File::open(provider_bin)?;
    let subject_kp = KeyPair::new_module();
    let issuer_kp = KeyPair::new_user();
    let archive_bytes = make_archive(
      bin_bytes,
      &Default::default(),
      Default::default(),
      &subject_kp,
      &issuer_kp,
    )?;

    let provider = Provider::from_tarbytes("test", "vino-test-par", &*archive_bytes).await?;
    let inputs: HashMap<&str, i32> = HashMap::from([("left", 2), ("right", 5)]);
    let stream = provider.invoke(Entity::component_direct("add"), inputs.into()).await?;

    let packets: Vec<_> = stream.collect().await;
    println!("packets: {:?}", packets);
    assert_eq!(packets.len(), 2);

    let signature = provider.get_list()?;
    assert_eq!(signature.len(), 1);

    Ok(())
  }
}
