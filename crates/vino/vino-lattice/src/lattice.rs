use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use vino_codec::messagepack::serialize;
use vino_rpc::SharedRpcHandler;
use vino_transport::{Invocation, MessageTransport, TransportStream, TransportWrapper};
use vino_types::HostedType;

use crate::error::LatticeError;
use crate::nats::{Nats, NatsMessage, NatsOptions};

type Result<T> = std::result::Result<T, LatticeError>;

static DEFAULT_TIMEOUT_SECS: u64 = 10;

/// The LatticeBuilder builds the configuration for a Lattice.
#[derive(Debug, Clone)]
pub struct LatticeBuilder {
  address: String,
  client_id: String,
  credential_path: Option<PathBuf>,
  token: Option<String>,
  timeout: Duration,
}

impl LatticeBuilder {
  /// Creates a new [LatticeBuilder].
  #[must_use]
  pub fn new<T: AsRef<str>, U: AsRef<str>>(address: T, namespace: U) -> Self {
    Self {
      address: address.as_ref().to_owned(),
      client_id: namespace.as_ref().to_owned(),
      credential_path: None,
      token: None,
      timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
    }
  }

  /// Creates a new [LatticeBuilder] using the environment variable NATS_URL for the address.
  pub fn new_from_env<T: AsRef<str>>(namespace: T) -> Result<Self> {
    let address = std::env::var("NATS_URL").map_err(|_| LatticeError::NatsEnvVar("NATS_URL".to_owned()))?;

    Ok(Self {
      address,
      client_id: namespace.as_ref().to_owned(),
      credential_path: None,
      token: None,
      timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
    })
  }

  /// Set the address.
  pub fn address(self, address: impl AsRef<str>) -> Result<Self> {
    Ok(Self {
      address: address.as_ref().to_owned(),
      ..self
    })
  }

  /// Set the client ID (ususally the public key of the connecting host).
  pub fn client_id(self, client_id: impl AsRef<str>) -> Result<Self> {
    Ok(Self {
      client_id: client_id.as_ref().to_owned(),
      ..self
    })
  }

  /// Set the NATS auth token.
  pub fn token(self, token: impl AsRef<str>) -> Result<Self> {
    Ok(Self {
      token: Some(token.as_ref().to_owned()),
      ..self
    })
  }

  /// Set the path to the NATS creds file.
  pub fn credential_path(self, credential_path: impl AsRef<str>) -> Result<Self> {
    Ok(Self {
      credential_path: Some(
        PathBuf::from_str(credential_path.as_ref()).map_err(|e| LatticeError::BadPath(e.to_string()))?,
      ),
      ..self
    })
  }

  /// Populate lattice configuration with a premade [NatsOptions] object
  pub fn with_opts(self, opts: NatsOptions) -> Result<Self> {
    #[allow(clippy::needless_update)]
    Ok(Self {
      address: opts.address,
      client_id: opts.client_id,
      token: opts.token,
      credential_path: opts.creds_path,
      ..self
    })
  }

  /// Constructs a Vino lattice and connects to NATS.
  pub async fn build(self) -> Result<Lattice> {
    Lattice::connect(NatsOptions {
      address: self.address,
      client_id: self.client_id,
      creds_path: self.credential_path,
      token: self.token,
      timeout: self.timeout,
    })
    .await
  }
}

#[derive(Debug)]
struct NsHandler {
  task: JoinHandle<()>,
}
impl NsHandler {
  fn new(task: JoinHandle<()>) -> Self {
    Self { task }
  }
}

impl Drop for Lattice {
  fn drop(&mut self) {
    for (_, handler) in self.handlers.write().iter_mut() {
      handler.task.abort();
    }
  }
}

#[derive(Debug, Clone)]
#[must_use]
pub struct Lattice {
  nats: Nats,
  timeout: Duration,
  handlers: Arc<RwLock<HashMap<String, NsHandler>>>,
}

impl Lattice {
  pub async fn connect(opts: NatsOptions) -> Result<Self> {
    let timeout = opts.timeout;
    let nats = Nats::connect(opts).await?;

    Ok(Self {
      nats,
      timeout,
      handlers: Default::default(),
    })
  }

  pub async fn shutdown(&self) -> Result<()> {
    self.nats.disconnect().await
  }

  pub async fn handle_namespace(&self, namespace: String, provider: SharedRpcHandler) -> Result<()> {
    trace!(namespace = namespace.as_str(), "register");

    let sub = self
      .nats
      .queue_subscribe(rpc_message_topic(&namespace), rpc_message_topic(&namespace))
      .await?;

    let deadline = self.timeout;
    let nats = self.nats.clone();

    let ns_inner = namespace.clone();
    let (ready_tx, ready_rx) = oneshot::channel::<()>();
    let handle = tokio::spawn(async move {
      trace!(namespace = ns_inner.as_str(), "handler open");
      let _ = ready_tx.send(());
      loop {
        trace!(namespace = ns_inner.as_str(), "handler wait");
        let next = sub.next_wait().await;
        match next {
          Some(nats_msg) => {
            debug!(
              namespace = ns_inner.as_str(),
              message = ?nats_msg.data(),
              "received message"
            );
            if let Err(e) = handle_message(&provider, nats_msg, deadline).await {
              error!(
                namespace = ns_inner.as_str(),
                error = e.to_string().as_str(),
                "Error processing lattice message",
              );
            }
          }
          None => {
            trace!(namespace = ns_inner.as_str(), "handler done");
            break;
          }
        }
        let _ = nats.flush().await;
      }
      trace!(namespace = ns_inner.as_str(), "handler close");
    });

    self.handlers.write().insert(namespace.clone(), NsHandler::new(handle));

    let _ = ready_rx.await;
    Ok(())
  }

  pub async fn invoke(&self, lattice_id: &str, invocation: Invocation) -> Result<TransportStream> {
    let target_url = invocation.target_url();

    debug!(target=target_url.as_str(),payload=?invocation.payload,"invoke");

    let topic = rpc_message_topic(lattice_id);
    let msg = LatticeRpcMessage::Invocation(invocation);
    let payload = serialize(&msg).map_err(|e| LatticeError::MessageSerialization(e.to_string()))?;
    let sub = self.nats.request(&topic, &payload).await?;

    let (tx, rx) = unbounded_channel();
    let stream = TransportStream::new(UnboundedReceiverStream::new(rx));

    tokio::spawn(async move {
      trace!(target = target_url.as_str(), "invoke task open");
      loop {
        trace!(target = target_url.as_str(), "invoke task wait");
        match sub.next().await {
          Ok(Some(nats_msg)) => {
            debug!(target = target_url.as_str(), message=?nats_msg.data(), "invoke task received message");
            if let Err(e) = handle_response(&tx, &nats_msg) {
              error!("Error processing response: {}", e);
            }
          }
          Ok(None) => {
            trace!(target = target_url.as_str(), "invoke task done");
            break;
          }
          Err(e) => {
            error!("Error retrieving lattice message for {}: {}", target_url, e);
            break;
          }
        }
      }
      trace!(target = target_url.as_str(), "invoke task close");
    });

    Ok(stream)
  }

  pub async fn list_components(&self, namespace: String) -> Result<Vec<HostedType>> {
    debug!(namespace = namespace.as_str(), "get signature");

    let topic = rpc_message_topic(&namespace);
    let msg = LatticeRpcMessage::List { namespace };
    let payload = serialize(&msg).map_err(|e| LatticeError::MessageSerialization(e.to_string()))?;
    let sub = self.nats.request(&topic, &payload).await?;

    let components = match sub.next().await? {
      Some(lattice_msg) => match lattice_msg.deserialize::<LatticeRpcResponse>() {
        Ok(LatticeRpcResponse::List(list)) => Ok(list),
        Ok(LatticeRpcResponse::Error(e)) => Err(LatticeError::ListFail(e)),
        Err(e) => Err(LatticeError::ListFail(e.to_string())),
        _ => unreachable!(),
      },
      None => Ok(vec![]),
    };

    components
  }
}

fn handle_response(tx: &UnboundedSender<TransportWrapper>, lattice_msg: &NatsMessage) -> Result<()> {
  let msg: Result<LatticeRpcResponse> = lattice_msg.deserialize();
  trace!(message=?msg,"lattice response");
  let result = match msg {
    Ok(response) => match response {
      LatticeRpcResponse::Output(wrapper) => tx.send(wrapper),
      LatticeRpcResponse::List(_) => unreachable!(),
      LatticeRpcResponse::Error(e) => tx.send(TransportWrapper::component_error(MessageTransport::error(e))),
      LatticeRpcResponse::Close => tx.send(TransportWrapper::new_system_close()),
    },
    Err(e) => tx.send(TransportWrapper::component_error(MessageTransport::error(
      e.to_string(),
    ))),
  };
  result.map_err(|_| LatticeError::ResponseUpstreamClosed)
}

async fn handle_message(provider: &SharedRpcHandler, nats_msg: NatsMessage, deadline: Duration) -> Result<()> {
  let msg: LatticeRpcMessage = nats_msg.deserialize()?;
  trace!(message=?msg,"lattice request");
  match msg {
    LatticeRpcMessage::List { .. } => {
      let result = provider.get_list();
      match result {
        Ok(components) => {
          let response = LatticeRpcResponse::List(components);
          nats_msg.respond(&response).await?;
        }
        Err(e) => {
          error!("Provider component list resulted in error: {}", e);
          let response = LatticeRpcResponse::Error(e.to_string());
          nats_msg.respond(&response).await?;
        }
      };
    }
    LatticeRpcMessage::Invocation(invocation) => {
      let target_url = invocation.target_url();
      let result = provider.invoke(invocation).await;

      match result {
        Ok(mut stream) => {
          loop {
            let result = timeout(deadline, stream.next()).await;
            match result {
              Ok(Some(msg)) => {
                let response = LatticeRpcResponse::Output(msg);
                nats_msg.respond(&response).await?;
              }
              Ok(None) => {
                break;
              }
              Err(_) => {
                error!("Timeout receiving next packet from invocation stream.");
                break;
              }
            }
          }

          let response = LatticeRpcResponse::Close;
          nats_msg.respond(&response).await?;
        }
        Err(e) => {
          error!("Provider invocation for {} resulted in error: {}", target_url, e);
          let response = LatticeRpcResponse::Error(e.to_string());
          nats_msg.respond(&response).await?;
        }
      };
    }
  }
  Ok(())
}

fn rpc_message_topic(ns: &str) -> String {
  format!("lattice.rpc.{}.{}", ns, "default")
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum LatticeRpcMessage {
  #[serde(rename = "0")]
  Invocation(Invocation),

  #[serde(rename = "1")]
  List {
    #[serde(rename = "1")]
    namespace: String,
  },
}

impl LatticeRpcMessage {}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
pub enum LatticeRpcResponse {
  #[serde(rename = "0")]
  Output(TransportWrapper),

  #[serde(rename = "1")]
  List(Vec<HostedType>),

  #[serde(rename = "2")]
  Error(String),

  #[serde(rename = "3")]
  Close,
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use tracing::*;
  use vino_codec::messagepack::{deserialize, serialize};
  use vino_transport::{MessageTransport, TransportWrapper};

  use crate::lattice::LatticeRpcResponse;
  #[test_logger::test]
  fn test_serde() -> Result<()> {
    let data = "Yay".to_owned();
    let expected = LatticeRpcResponse::Output(TransportWrapper {
      port: "port-name".to_owned(),
      payload: MessageTransport::success(&data),
    });
    let bytes = serialize(&expected).unwrap();
    debug!("{:?}", bytes);
    let actual: LatticeRpcResponse = deserialize(&bytes)?;
    assert_eq!(expected, actual);
    Ok(())
  }
}

#[cfg(test)]
mod test_integration {
  use std::convert::TryInto;
  use std::sync::Arc;

  use anyhow::Result;
  use test_vino_provider::Provider;
  use tokio_stream::StreamExt;
  use tracing::*;
  use vino_rpc::MapWrapper;
  use vino_transport::{Invocation, MessageTransport, TransportMap};
  use vino_types::{ComponentMap, ComponentSignature, HostedType, ProviderSignature, StructMap};

  use super::{Lattice, LatticeBuilder};

  async fn get_lattice() -> Result<(Lattice, String)> {
    let lattice_builder = LatticeBuilder::new_from_env("test").unwrap();
    let lattice = lattice_builder.build().await.unwrap();
    let namespace = "some_namespace_id".to_owned();
    lattice
      .handle_namespace(namespace.clone(), Arc::new(Provider::default()))
      .await
      .unwrap();

    Ok((lattice, namespace))
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_rpc_invoke() -> Result<()> {
    let (lattice, lattice_id) = get_lattice().await?;
    let component_name = "test-component";
    let user_input = String::from("Hello world");
    let entity = vino_entity::Entity::component("arbitrary_ns", component_name);
    let mut payload = TransportMap::new();
    payload.insert("input", MessageTransport::success(&user_input));
    println!("Sending payload: {:?}", payload);
    let invocation = Invocation::new_test(file!(), entity, payload, None);
    let mut stream = lattice.invoke(&lattice_id, invocation).await?;
    println!("Sent payload, received stream");

    let msg = stream.next().await.unwrap();
    debug!("msg: {:?}", msg);
    let result: String = msg.deserialize()?;
    assert_eq!(result, format!("TEST: {}", user_input));

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_clean_shutdown() -> Result<()> {
    let lattice_builder = LatticeBuilder::new_from_env("test").unwrap();
    let lattice = lattice_builder.build().await.unwrap();
    let namespace = "some_namespace_id".to_owned();
    lattice
      .handle_namespace(namespace.clone(), Arc::new(Provider::default()))
      .await
      .unwrap();
    let _ = lattice.shutdown().await;

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_rpc_invoke_error() -> Result<()> {
    let (lattice, lattice_id) = get_lattice().await?;
    let component_name = "error";
    let user_input = String::from("Hello world");
    let entity = vino_entity::Entity::component("arbitrary_ns", component_name);
    let mut payload = TransportMap::new();
    payload.insert("input", MessageTransport::success(&user_input));
    debug!("Sending payload: {:?}", payload);
    let invocation = Invocation::new_test(file!(), entity, payload, None);
    let mut stream = lattice.invoke(&lattice_id, invocation).await?;
    debug!("Sent payload, received stream");

    let msg = stream.next().await;
    debug!("msg: {:?}", msg);
    let msg = msg.unwrap();
    assert_eq!(msg.payload, MessageTransport::error("This always errors"));

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_rpc_list_namespace_components() -> Result<()> {
    let (lattice, namespace) = get_lattice().await?;
    let schemas = lattice.list_components(namespace).await?;
    debug!("Hosted schemas on namespace: {:#?}", schemas);
    let mut components = ComponentMap::default();
    components.insert(
      "error",
      ComponentSignature {
        name: "error".to_owned(),
        inputs: vec![("input", "string")].try_into()?,
        outputs: vec![("output", "string")].try_into()?,
      },
    );
    components.insert(
      "test-component",
      ComponentSignature {
        name: "test-component".to_owned(),
        inputs: vec![("input", "string")].try_into()?,
        outputs: vec![("output", "string")].try_into()?,
      },
    );

    assert!(schemas.contains(&HostedType::Provider(ProviderSignature {
      name: Some("test-vino-provider".to_owned()),
      types: StructMap::new(),
      components
    })));
    Ok(())
  }
}
