use std::collections::HashMap;
use std::time::Duration;

use serde::Serialize;
use vino_wascap::KeyPair;

use crate::dev::prelude::*;
use crate::network_service::handlers::initialize::Initialize;
pub use crate::providers::network_provider::Provider as NetworkProvider;

type Result<T> = std::result::Result<T, NetworkError>;
#[derive(Debug)]
pub struct Network {
  pub id: String,
  definition: NetworkDefinition,
  addr: Addr<NetworkService>,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
  kp: KeyPair,
}

impl Network {
  pub fn new(definition: NetworkDefinition, seed: &str) -> Result<Self> {
    Ok(NetworkBuilder::new(definition, seed)?.build())
  }
  pub async fn init(&self) -> Result<()> {
    let kp = KeyPair::new_service();
    let seed = kp.seed().map_err(|_| InternalError(5103))?;
    self
      .addr
      .send(Initialize {
        network_id: self.id.clone(),
        seed,
        network: self.definition.clone(),
        allowed_insecure: self.allowed_insecure.clone(),
        allow_latest: self.allow_latest,
      })
      .await
      .map_err(|_| InternalError(5102))??;
    Ok(())
  }
  pub async fn request<T, U>(
    &self,
    schematic: T,
    origin: Entity,
    data: &HashMap<U, impl Serialize + Sync>,
  ) -> Result<HashMap<String, MessageTransport>>
  where
    T: AsRef<str> + Send + Sync,
    U: AsRef<str> + Send + Sync,
  {
    let serialized_data: HashMap<String, Vec<u8>> = data
      .iter()
      .map(|(k, v)| Ok((k.as_ref().to_owned(), mp_serialize(&v)?)))
      .filter_map(Result::ok)
      .collect();

    let time = std::time::Instant::now();
    let payload = MessageTransport::MultiBytes(serialized_data);

    let invocation = Invocation::new(
      &self.kp,
      origin,
      Entity::Schematic(schematic.as_ref().to_owned()),
      payload,
    );
    let response = self
      .addr
      .send(invocation)
      .timeout(Duration::from_secs(10))
      .await
      .map_err(|_| InternalError(5101))?;
    trace!(
      "result for {} took {} μs",
      schematic.as_ref().to_owned(),
      time.elapsed().as_micros()
    );

    match response {
      InvocationResponse::Stream { mut rx, .. } => {
        debug!("Got stream");
        let mut map = HashMap::new();
        while let Some(next) = rx.next().await {
          debug!("Received packet on port {}: {:?}", next.port, next.payload);
          map.insert(next.port, next.payload.into());
        }
        trace!("Result: {:?}", map);
        Ok(map)
      }
      InvocationResponse::Error { msg, .. } => Err(NetworkError::ExecutionError(msg)),
    }
  }
}

/// The HostBuilder builds the configuration for a Vino Host
#[derive(Debug)]
pub struct NetworkBuilder {
  allow_latest: bool,
  allowed_insecure: Vec<String>,
  definition: NetworkDefinition,
  kp: KeyPair,
  id: String,
}

impl NetworkBuilder {
  /// Creates a new host builder
  pub fn new(definition: NetworkDefinition, seed: &str) -> Result<Self> {
    let kp = keypair_from_seed(seed)?;
    let network_id = kp.public_key();
    Ok(Self {
      definition,
      allow_latest: false,
      allowed_insecure: vec![],
      id: network_id,
      kp,
    })
  }

  #[must_use]
  pub fn allow_latest(self, val: bool) -> Self {
    Self {
      allow_latest: val,
      ..self
    }
  }

  #[must_use]
  pub fn allow_insecure(self, registries: Vec<String>) -> Self {
    Self {
      allowed_insecure: registries,
      ..self
    }
  }

  /// Constructs an instance of a Vino host.
  #[must_use]
  pub fn build(self) -> Network {
    let addr = crate::network_service::NetworkService::for_id(&self.id);
    Network {
      addr,
      definition: self.definition,
      id: self.id,
      allow_latest: self.allow_latest,
      allowed_insecure: self.allowed_insecure,
      kp: self.kp,
    }
  }
}
