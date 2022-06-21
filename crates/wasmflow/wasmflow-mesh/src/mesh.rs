use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use sdk::codec::messagepack::serialize;
use sdk::transport::{MessageTransport, TransportStream, TransportWrapper};
use sdk::types::HostedType;
use sdk::{Entity, Invocation};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use wasmflow_rpc::SharedRpcHandler;
use wasmflow_sdk::v1 as sdk;

use crate::error::MeshError;
use crate::nats::{Nats, NatsMessage, NatsOptions};

type Result<T> = std::result::Result<T, MeshError>;

static DEFAULT_TIMEOUT_SECS: u64 = 10;

/// The MeshBuilder builds the configuration for a Mesh.
#[derive(Debug, Clone)]
pub struct MeshBuilder {
  address: String,
  client_id: String,
  credential_path: Option<PathBuf>,
  token: Option<String>,
  timeout: Duration,
}

impl MeshBuilder {
  /// Creates a new [MeshBuilder].
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

  /// Creates a new [MeshBuilder] using the environment variable NATS_URL for the address.
  pub fn new_from_env<T: AsRef<str>>(namespace: T) -> Result<Self> {
    let address = std::env::var("NATS_URL").map_err(|_| MeshError::NatsEnvVar("NATS_URL".to_owned()))?;

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
        PathBuf::from_str(credential_path.as_ref()).map_err(|e| MeshError::BadPath(e.to_string()))?,
      ),
      ..self
    })
  }

  /// Populate mesh configuration with a premade [NatsOptions] object
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

  /// Constructs a Wasmflow mesh and connects to NATS.
  pub async fn build(self) -> Result<Mesh> {
    Mesh::connect(NatsOptions {
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

impl Drop for Mesh {
  fn drop(&mut self) {
    for (_, handler) in self.handlers.write().iter_mut() {
      handler.task.abort();
    }
  }
}

#[derive(Debug, Clone)]
#[must_use]
pub struct Mesh {
  nats: Nats,
  timeout: Duration,
  handlers: Arc<RwLock<HashMap<String, NsHandler>>>,
}

impl Mesh {
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

  pub async fn handle_namespace(&self, namespace: String, collection: SharedRpcHandler) -> Result<()> {
    trace!(%namespace, "register");

    let sub = self
      .nats
      .queue_subscribe(rpc_message_topic(&namespace), rpc_message_topic(&namespace))
      .await?;

    let deadline = self.timeout;
    let nats = self.nats.clone();

    let ns_inner = namespace.clone();
    let (ready_tx, ready_rx) = oneshot::channel::<()>();
    let handle = tokio::spawn(async move {
      let span = trace_span!("mesh ns handler", namespace=%ns_inner);
      let _guard = span.enter();

      trace!("open");
      let _ = ready_tx.send(());
      loop {
        trace!("handler wait");
        let next = sub.next_wait().await;
        match next {
          Some(nats_msg) => {
            debug!(message = ?nats_msg.data(),"received message");
            if let Err(error) = handle_message(&collection, nats_msg, deadline).await {
              error!(%error,"Error processing mesh message",);
            }
          }
          None => {
            trace!("handler done");
            break;
          }
        }
        let _ = nats.flush().await;
      }
      trace!("handler close");
    });

    self.handlers.write().insert(namespace.clone(), NsHandler::new(handle));

    let _ = ready_rx.await;
    Ok(())
  }

  pub async fn invoke(&self, mesh_id: &str, mut invocation: Invocation) -> Result<TransportStream> {
    let target_url = invocation.target_url();

    debug!(target=%target_url,payload=?invocation.payload,"invoke");

    let topic = rpc_message_topic(mesh_id);
    debug!(from = invocation.target.namespace(), to = mesh_id, "resolved namespace");
    invocation.target = Entity::component(mesh_id, invocation.target.name());

    let msg = MeshRpcMessage::Invocation(Box::new(invocation));
    let payload = serialize(&msg).map_err(|e| MeshError::MessageSerialization(e.to_string()))?;
    let sub = self.nats.request(&topic, &payload).await?;

    let (tx, rx) = unbounded_channel();
    let stream = TransportStream::new(UnboundedReceiverStream::new(rx));

    tokio::spawn(async move {
      let span = trace_span!("invocation task",target = %target_url);
      let _guard = span.enter();
      trace!("task open");
      loop {
        trace!("task wait");
        match sub.next().await {
          Ok(Some(nats_msg)) => {
            debug!(message=?nats_msg.data(), "received message");
            if let Err(e) = handle_response(&tx, &nats_msg) {
              error!("Error processing response: {}", e);
            }
          }
          Ok(None) => {
            trace!("task done");
            break;
          }
          Err(e) => {
            error!("Error retrieving mesh message for {}: {}", target_url, e);
            break;
          }
        }
      }
      trace!("task close");
    });

    Ok(stream)
  }

  pub async fn list_components(&self, namespace: String) -> Result<Vec<HostedType>> {
    debug!(%namespace, "get signature");

    let topic = rpc_message_topic(&namespace);
    let msg = MeshRpcMessage::List { namespace };
    let payload = serialize(&msg).map_err(|e| MeshError::MessageSerialization(e.to_string()))?;
    let sub = self.nats.request(&topic, &payload).await?;

    let components = match sub.next().await? {
      Some(mesh_msg) => match mesh_msg.deserialize::<MeshRpcResponse>() {
        Ok(MeshRpcResponse::List(list)) => Ok(list),
        Ok(MeshRpcResponse::Error(e)) => Err(MeshError::ListFail(e)),
        Err(e) => Err(MeshError::ListFail(e.to_string())),
        _ => unreachable!(),
      },
      None => Ok(vec![]),
    };

    components
  }
}

fn handle_response(tx: &UnboundedSender<TransportWrapper>, mesh_msg: &NatsMessage) -> Result<()> {
  let msg: Result<MeshRpcResponse> = mesh_msg.deserialize();
  trace!(message=?msg,"mesh response");
  let result = match msg {
    Ok(response) => match response {
      MeshRpcResponse::Output(wrapper) => tx.send(wrapper),
      MeshRpcResponse::List(_) => unreachable!(),
      MeshRpcResponse::Error(e) => tx.send(TransportWrapper::component_error(MessageTransport::error(e))),
      MeshRpcResponse::Close => tx.send(TransportWrapper::new_system_close()),
    },
    Err(e) => tx.send(TransportWrapper::component_error(MessageTransport::error(
      e.to_string(),
    ))),
  };
  result.map_err(|_| MeshError::ResponseUpstreamClosed)
}

async fn handle_message(collection: &SharedRpcHandler, nats_msg: NatsMessage, deadline: Duration) -> Result<()> {
  let msg: MeshRpcMessage = nats_msg.deserialize()?;
  trace!(message=?msg,"mesh request");
  match msg {
    MeshRpcMessage::List { .. } => {
      let result = collection.get_list();
      match result {
        Ok(components) => {
          let response = MeshRpcResponse::List(components);
          nats_msg.respond(&response).await?;
        }
        Err(e) => {
          error!("Collection component list resulted in error: {}", e);
          let response = MeshRpcResponse::Error(e.to_string());
          nats_msg.respond(&response).await?;
        }
      };
    }
    MeshRpcMessage::Invocation(invocation) => {
      let target_url = invocation.target_url();
      let result = collection.invoke(*invocation).await;

      match result {
        Ok(mut stream) => {
          loop {
            let result = timeout(deadline, stream.next()).await;
            match result {
              Ok(Some(msg)) => {
                let response = MeshRpcResponse::Output(msg);
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

          let response = MeshRpcResponse::Close;
          nats_msg.respond(&response).await?;
        }
        Err(e) => {
          error!("Collection invocation for {} resulted in error: {}", target_url, e);
          let response = MeshRpcResponse::Error(e.to_string());
          nats_msg.respond(&response).await?;
        }
      };
    }
  }
  Ok(())
}

fn rpc_message_topic(ns: &str) -> String {
  format!("mesh.rpc.{}.{}", ns, "default")
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum MeshRpcMessage {
  #[serde(rename = "0")]
  Invocation(Box<Invocation>),

  #[serde(rename = "1")]
  List {
    #[serde(rename = "1")]
    namespace: String,
  },
}

impl MeshRpcMessage {}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
pub enum MeshRpcResponse {
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
  use pretty_assertions::assert_eq;
  use tracing::*;
  use wasmflow_sdk::v1::codec::messagepack::{deserialize, serialize};
  use wasmflow_sdk::v1::transport::{MessageTransport, TransportWrapper};

  use crate::mesh::MeshRpcResponse;
  #[test_logger::test]
  fn test_serde() -> Result<()> {
    let data = "Yay".to_owned();
    let expected = MeshRpcResponse::Output(TransportWrapper {
      port: "port-name".to_owned(),
      payload: MessageTransport::success(&data),
    });
    let bytes = serialize(&expected).unwrap();
    debug!("{:?}", bytes);
    let actual: MeshRpcResponse = deserialize(&bytes)?;
    assert_eq!(expected, actual);
    Ok(())
  }
}

#[cfg(test)]
mod test_integration {
  use std::convert::TryInto;
  use std::sync::Arc;

  use anyhow::Result;
  use pretty_assertions::assert_eq;
  use test_native_collection::Collection;
  use tracing::*;
  use wasmflow_sdk::v1::packet::PacketMap;
  use wasmflow_sdk::v1::transport::MessageTransport;
  use wasmflow_sdk::v1::types::{CollectionSignature, ComponentMap, ComponentSignature, HostedType};
  use wasmflow_sdk::v1::Invocation;

  use super::{Mesh, MeshBuilder};

  async fn get_mesh() -> Result<(Mesh, String)> {
    let mesh_builder = MeshBuilder::new_from_env("test").unwrap();
    let mesh = mesh_builder.build().await.unwrap();
    let namespace = "some_namespace_id".to_owned();
    mesh
      .handle_namespace(namespace.clone(), Arc::new(Collection::default()))
      .await
      .unwrap();

    Ok((mesh, namespace))
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_rpc_invoke() -> Result<()> {
    let (mesh, mesh_id) = get_mesh().await?;
    let component_name = "test-component";
    let user_input = String::from("Hello world");
    let entity = wasmflow_sdk::v1::Entity::component("arbitrary_ns", component_name);
    let mut payload = PacketMap::default();
    payload.insert("input", &user_input);
    println!("Sending payload: {:?}", payload);
    let invocation = Invocation::new_test(file!(), entity, payload, None);
    let mut stream = mesh.invoke(&mesh_id, invocation).await?;
    println!("Sent payload, received stream");
    let output = stream.drain_port("output").await?;

    let msg = output[0].clone();
    debug!("msg: {:?}", msg);
    let result: String = msg.deserialize()?;
    assert_eq!(result, format!("TEST: {}", user_input));

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_clean_shutdown() -> Result<()> {
    let mesh_builder = MeshBuilder::new_from_env("test").unwrap();
    let mesh = mesh_builder.build().await.unwrap();
    let namespace = "some_namespace_id".to_owned();
    mesh
      .handle_namespace(namespace.clone(), Arc::new(Collection::default()))
      .await
      .unwrap();
    let _ = mesh.shutdown().await;

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_rpc_invoke_error() -> Result<()> {
    let (mesh, mesh_id) = get_mesh().await?;
    let component_name = "error";
    let user_input = String::from("Hello world");
    let entity = wasmflow_sdk::v1::Entity::component("arbitrary_ns", component_name);
    let mut payload = PacketMap::default();
    payload.insert("input", &user_input);
    debug!("Sending payload: {:?}", payload);
    let invocation = Invocation::new_test(file!(), entity, payload, None);
    let mut stream = mesh.invoke(&mesh_id, invocation).await?;
    debug!("Sent payload, received stream");
    let outputs = stream.drain().await;

    let msg = outputs[0].clone();
    debug!("msg: {:?}", msg);
    assert_eq!(msg.payload, MessageTransport::error("This always errors"));

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn integration_test_rpc_list_namespace_components() -> Result<()> {
    let (mesh, namespace) = get_mesh().await?;
    let schemas = mesh.list_components(namespace).await?;
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

    let expected = HostedType::Collection(CollectionSignature {
      name: Some("test-native-collection".to_owned()),
      format: 1,
      version: "0.1.0".to_owned(),
      components,
      ..Default::default()
    });

    println!("Returned schemas: {:?}", schemas);

    assert_eq!(schemas[0], expected);
    Ok(())
  }
}
