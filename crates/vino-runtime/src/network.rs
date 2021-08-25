use std::collections::HashMap;
use std::time::Duration;

use serde::Serialize;
use vino_lattice::nats::NatsOptions;
use vino_transport::message_transport::TransportMap;
use vino_wascap::KeyPair;

use crate::dev::prelude::*;
use crate::network_service::handlers::initialize::Initialize;
pub use crate::providers::network_provider::Provider as NetworkProvider;

type Result<T> = std::result::Result<T, RuntimeError>;
#[derive(Debug)]
#[must_use]
pub struct Network {
  pub id: String,
  definition: NetworkDefinition,
  addr: Addr<NetworkService>,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
  kp: KeyPair,
  timeout: Duration,
  lattice_config: Option<NatsOptions>,
}

impl Network {
  pub fn new(definition: NetworkDefinition, seed: &str) -> Result<Self> {
    Ok(NetworkBuilder::new(definition, seed)?.build())
  }

  pub async fn init(&self) -> Result<()> {
    let kp = KeyPair::new_service();
    let seed = log_ie!(kp.seed(), 5103)?;
    let init = Initialize {
      network_id: self.id.clone(),
      seed,
      lattice_config: self.lattice_config.clone(),
      network: self.definition.clone(),
      allowed_insecure: self.allowed_insecure.clone(),
      allow_latest: self.allow_latest,
      timeout: self.timeout,
    };
    log_ie!(self.addr.send(init).await, 5102)??;
    Ok(())
  }

  pub async fn request<T, U>(
    &self,
    schematic: T,
    origin: Entity,
    data: &HashMap<U, impl Serialize + Sync>,
  ) -> Result<TransportStream>
  where
    T: AsRef<str> + Send + Sync,
    U: AsRef<str> + Send + Sync,
  {
    let serialized_data: HashMap<String, MessageTransport> = data
      .iter()
      .map(|(k, v)| {
        Ok((
          k.as_ref().to_owned(),
          MessageTransport::MessagePack(mp_serialize(&v)?),
        ))
      })
      .filter_map(Result::ok)
      .collect();

    let time = std::time::Instant::now();
    let payload = TransportMap::with_map(serialized_data);

    let invocation = Invocation::new(
      origin,
      Entity::Schematic(schematic.as_ref().to_owned()),
      payload,
    );

    let response: InvocationResponse = log_ie!(
      self
        .addr
        .send(invocation)
        .timeout(Duration::from_secs(10))
        .await,
      5101
    )?;

    trace!(
      "NETWORK:Result for {} took {} μs",
      schematic.as_ref().to_owned(),
      time.elapsed().as_micros()
    );
    Ok(response.ok()?)
  }
}

/// The HostBuilder builds the configuration for a Vino Host.
#[derive(Debug)]
#[must_use]
pub struct NetworkBuilder {
  allow_latest: bool,
  allowed_insecure: Vec<String>,
  definition: NetworkDefinition,
  kp: KeyPair,
  id: String,
  nats_address: Option<String>,
  // namespace: String,
  nats_creds_path: Option<String>,
  nats_token: Option<String>,
  timeout: Duration,
}

impl NetworkBuilder {
  /// Creates a new host builder.
  pub fn new(definition: NetworkDefinition, seed: &str) -> Result<Self> {
    let kp = keypair_from_seed(seed)?;
    let network_id = kp.public_key();
    Ok(Self {
      definition,
      allow_latest: false,
      allowed_insecure: vec![],
      id: network_id,
      timeout: Duration::from_secs(5),
      nats_address: None,
      nats_creds_path: None,
      nats_token: None,
      kp,
    })
  }

  pub fn timeout(self, val: Duration) -> Self {
    Self {
      timeout: val,
      ..self
    }
  }

  pub fn allow_latest(self, val: bool) -> Self {
    Self {
      allow_latest: val,
      ..self
    }
  }

  pub fn allow_insecure(self, registries: Vec<String>) -> Self {
    Self {
      allowed_insecure: registries,
      ..self
    }
  }

  pub fn nats_address(self, nats_address: String) -> Self {
    Self {
      nats_address: Some(nats_address),
      ..self
    }
  }

  pub fn from_env(self) -> Self {
    let mut me = self;
    if let Ok(nats_address) = std::env::var("NATS_URL") {
      me = Self {
        nats_address: Some(nats_address),
        ..me
      }
    };
    me
  }

  pub fn nats_creds_path(self, nats_creds_path: String) -> Self {
    Self {
      nats_creds_path: Some(nats_creds_path),
      ..self
    }
  }

  pub fn nats_token(self, nats_token: String) -> Self {
    Self {
      nats_token: Some(nats_token),
      ..self
    }
  }

  /// Constructs an instance of a Vino host.
  pub fn build(self) -> Network {
    let addr = crate::network_service::NetworkService::for_id(&self.id);
    let nats_options = match self.nats_address {
      Some(addr) => Some(NatsOptions {
        address: addr,
        creds_path: self.nats_creds_path,
        token: self.nats_token,
        client_id: self.kp.public_key(),
      }),
      None => None,
    };

    Network {
      addr,
      definition: self.definition,
      id: self.id,
      allow_latest: self.allow_latest,
      allowed_insecure: self.allowed_insecure,
      kp: self.kp,
      timeout: self.timeout,
      lattice_config: nats_options,
    }
  }
}
