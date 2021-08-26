use std::sync::{
  Arc,
  RwLock,
};
use std::thread;
use std::time::Duration;

use log::{
  debug,
  error,
  trace,
};
use nats::jetstream::{
  RetentionPolicy,
  StreamConfig,
};
use tokio::runtime::Builder;
use tokio::spawn;
use tokio::sync::mpsc::unbounded_channel;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use vino_codec::messagepack::{
  deserialize,
  serialize,
};
use vino_entity::Entity;
use vino_rpc::{
  BoxedRpcHandler,
  RpcHandler,
};
use vino_transport::{
  MessageTransport,
  TransportMap,
  TransportStream,
  TransportWrapper,
};
use vino_types::signatures::HostedType;

use crate::error::LatticeError;
use crate::nats::{
  Nats,
  NatsOptions,
};

type Result<T> = std::result::Result<T, LatticeError>;

/// The LatticeBuilder builds the configuration for a Lattice.
#[derive(Debug, Clone)]
pub struct LatticeBuilder {
  address: String,
  client_id: String,
  credential_path: Option<String>,
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
      timeout: Duration::from_secs(5),
    }
  }

  /// Creates a new [LatticeBuilder] using the environment variable NATS_URL for the address.
  pub fn new_from_env<T: AsRef<str>>(namespace: T) -> Result<Self> {
    let address =
      std::env::var("NATS_URL").map_err(|_| LatticeError::NatsEnvVar("NATS_URL".to_owned()))?;

    Ok(Self {
      address,
      client_id: namespace.as_ref().to_owned(),
      credential_path: None,
      token: None,
      timeout: Duration::from_secs(5),
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
      credential_path: Some(credential_path.as_ref().to_owned()),
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

// static PREFIX: &str = "ofp";
static RPC_STREAM_NAME: &str = "rpc";

#[derive(Debug, Clone)]
pub struct Lattice {
  nats: Nats,
  rpc_stream_topic: String,
  handlers: Arc<RwLock<Vec<JoinHandle<()>>>>,
}

impl Lattice {
  pub async fn connect(opts: NatsOptions) -> Result<Self> {
    let nats = Nats::connect(opts).await?;

    let rpc_stream_topic = RPC_STREAM_NAME.to_owned();

    let subjects = vec![format!("{}.*", rpc_stream_topic)];
    let stream_config = StreamConfig {
      subjects: Some(subjects),
      name: rpc_stream_topic.clone(),
      retention: RetentionPolicy::WorkQueue,

      ..Default::default()
    };

    let stream_info = nats.create_stream(stream_config).await?;
    debug!("LATTICE:RPC_STREAM[{}]:CREATED", stream_info.config.name);

    Ok(Self {
      nats,
      rpc_stream_topic,
      handlers: Arc::new(RwLock::new(vec![])),
    })
  }

  pub async fn handle_namespace<F>(&self, namespace: String, handler: F) -> Result<()>
  where
    F: FnOnce() -> BoxedRpcHandler,
  {
    debug!("LATTICE:HANDLER[{}]:REGISTER", namespace);
    let mut consumer = self.nats.create_consumer(namespace.to_owned()).await?;

    let provider = handler();
    let nc = self.nats.clone();

    thread::spawn(|| {
      let rt = Builder::new_multi_thread()
        .thread_name(format!("lattice_handler_{}", namespace))
        .build()
        .unwrap();
      rt.block_on(async move {
        debug!("LATTICE:HANDLER[{}]:OPEN", namespace);
        while let Ok(nats_msg) = consumer.next().await {
          let result: Result<LatticeRpcMessage> = nats_msg.deserialize();
          trace!("LATTICE:HANDLER[{}]:RPC_MESSAGE:{:?}", namespace, result);
          let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
              error!(
                "Error deserializing RPC message, can not continue. Error was: {}",
                e
              );
              return;
            }
          };
          match msg {
            LatticeRpcMessage::List { reply_to, .. } => {
              println!("pre get_list");
              let result = provider.get_list().await;
              println!("post get_list");
              match result {
                Ok(components) => {
                  let response = LatticeRpcResponse::List(components);
                  if let Err(e) = nc.publish(reply_to.clone(), response.serialize()).await {
                    error!("Error sending response to lattice: {}", e);
                    break;
                  }
                }
                Err(e) => {
                  let response = LatticeRpcResponse::Error(e.to_string());
                  if let Err(e) = nc.publish(reply_to, response.serialize()).await {
                    error!("Error sending response to lattice: {}", e);
                  }
                }
              };
            }
            LatticeRpcMessage::Invocation {
              reply_to,
              entity,
              payload,
            } => {
              let result = provider.invoke(entity, payload).await;
              match result {
                Ok(mut stream) => {
                  while let Some(msg) = stream.next().await {
                    let response = LatticeRpcResponse::Output(msg);
                    if let Err(e) = nc.publish(reply_to.clone(), response.serialize()).await {
                      error!("Error sending response to lattice: {}", e);
                      break;
                    }
                  }
                }
                Err(e) => {
                  let response = LatticeRpcResponse::Error(e.to_string());
                  if let Err(e) = nc.publish(reply_to, response.serialize()).await {
                    error!("Error sending response to lattice: {}", e);
                  }
                }
              };
            }
          }
          if let Err(e) = nats_msg.ack().await {
            error!(
              "Error sending ACK for message {}. Error was {}",
              nats_msg.subject(),
              e
            )
          }
          trace!("LATTICE:HANDLER[{}]:RPC_MESSAGE:ACKED", namespace);
        }
        error!("LATTICE:HANDLER[{}]:CLOSE", namespace);
      })
    });

    Ok(())
  }

  pub async fn invoke(&self, entity: Entity, payload: TransportMap) -> Result<TransportStream> {
    let entity_string = entity.to_string();
    debug!("LATTICE:INVOKE[{}]:PAYLOAD[{:?}]", entity_string, payload);
    let ns = match &entity {
      Entity::Component(ns, _) => ns,
      _ => {
        return Err(LatticeError::InvalidEntity);
      }
    };

    // Create unique inbox subject to listen on for the reply
    let reply = self.nats.new_inbox().await;
    let sub = self.nats.subscribe(reply.clone()).await?;

    let topic = rpc_message_topic(&self.rpc_stream_topic, ns);
    let msg = LatticeRpcMessage::Invocation {
      reply_to: reply,
      entity,
      payload,
    };
    let payload = serialize(&msg).map_err(|e| LatticeError::MessageSerialization(e.to_string()))?;
    self.nats.publish(topic, payload).await?;

    let (tx, rx) = unbounded_channel();
    let stream = TransportStream::new(UnboundedReceiverStream::new(rx));
    debug!("LATTICE:INVOKE[{}]:REPLY_LISTENER:OPEN", entity_string);
    spawn(async move {
      while let Ok(Some(lattice_msg)) = sub.next().await {
        debug!(
          "LATTICE:INVOKE[{}]:NEXT:DATA:{:?}",
          entity_string, lattice_msg.data
        );
        let result = match deserialize::<LatticeRpcResponse>(&lattice_msg.data) {
          Ok(LatticeRpcResponse::Output(wrapper)) => tx.send(wrapper),
          Ok(LatticeRpcResponse::Error(e)) => {
            tx.send(TransportWrapper::internal_error(MessageTransport::Error(e)))
          }
          Err(e) => tx.send(TransportWrapper::new(
            vino_transport::SYSTEM_ID,
            MessageTransport::Error(e.to_string()),
          )),
          _ => unreachable!(),
        };
        if let Err(e) = result {
          error!("Error sending RPC output to TransportStream: {}", e);
          break;
        }
      }
      debug!("LATTICE:INVOKE[{}]:REPLY_LISTENER:CLOSE", entity_string);
    });

    Ok(stream)
  }

  pub async fn list_namespaces(&self) -> Result<Vec<String>> {
    self.nats.list_consumers(RPC_STREAM_NAME.to_owned()).await
  }

  pub async fn list_components(&self, entity: Entity) -> Result<Vec<HostedType>> {
    debug!("LATTICE:LIST[{}]", entity);

    // Create unique inbox subject to listen on for the reply
    let reply = self.nats.new_inbox().await;
    let sub = self.nats.subscribe(reply.clone()).await?;

    let topic = rpc_message_topic(&self.rpc_stream_topic, &entity.name());
    let msg = LatticeRpcMessage::List {
      reply_to: reply,
      namespace: entity.name(),
    };
    let payload = serialize(&msg).map_err(|e| LatticeError::MessageSerialization(e.to_string()))?;
    self.nats.publish(topic, payload).await?;

    let components = match sub.next().await? {
      Some(lattice_msg) => match deserialize::<LatticeRpcResponse>(&lattice_msg.data) {
        Ok(LatticeRpcResponse::List(list)) => Ok(list),
        Ok(LatticeRpcResponse::Error(e)) => Err(LatticeError::ListFail(e)),
        Err(e) => Err(LatticeError::ListFail(e.to_string())),
        _ => unreachable!(),
      },
      None => Ok(vec![]),
    };

    components
  }

  pub async fn get_total_pending(&self) -> Result<u64> {
    let info = self.nats.stream_info(RPC_STREAM_NAME.to_owned()).await?;
    let state = info.state;
    Ok(state.messages)
  }
}

fn rpc_message_topic(prefix: &str, ns: &str) -> String {
  format!("{}.{}", prefix, ns)
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum LatticeRpcMessage {
  Invocation {
    reply_to: String,
    entity: vino_entity::Entity,
    payload: TransportMap,
  },
  List {
    reply_to: String,
    namespace: String,
  },
}

impl LatticeRpcMessage {}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum LatticeRpcResponse {
  Output(TransportWrapper),
  List(Vec<HostedType>),
  Error(String),
}

impl LatticeRpcResponse {
  pub fn serialize(&self) -> Vec<u8> {
    serialize(self)
      .unwrap_or_else(|e| serialize(&LatticeRpcResponse::Error(e.to_string())).unwrap())
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use async_once::AsyncOnce;
  use log::*;
  use test_vino_provider::Provider;
  use tokio_stream::StreamExt;
  use vino_entity::Entity;
  use vino_transport::{
    MessageTransport,
    TransportMap,
  };
  use vino_types::signatures::{
    ComponentSignature,
    HostedType,
    PortSignature,
  };

  use super::{
    Lattice,
    LatticeBuilder,
  };

  async fn get_lattice() -> Result<(Lattice, String)> {
    let lattice_builder = LatticeBuilder::new_from_env("test").unwrap();
    let lattice = lattice_builder.build().await.unwrap();
    let namespace = "some_namespace_id".to_owned();
    lattice
      .handle_namespace(namespace.clone(), || Box::new(Provider::default()))
      .await
      .unwrap();

    Ok((lattice, namespace))
  }

  #[test_logger::test(tokio::test)]
  async fn test_invoke() -> Result<()> {
    let (lattice, namespace) = get_lattice().await?;
    let component_name = "test-component";
    let user_input = String::from("Hello world");
    let entity = vino_entity::Entity::component(namespace, component_name);
    let mut payload = TransportMap::new();
    payload.insert("input", MessageTransport::success(&user_input));
    println!("Sending payload: {:?}", payload);
    let mut stream = lattice.invoke(entity, payload).await?;
    println!("Sent payload, received stream");

    let msg = stream.next().await.unwrap();
    debug!("msg: {:?}", msg);
    let result: String = msg.try_into()?;
    assert_eq!(result, format!("TEST: {}", user_input));

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_list() -> Result<()> {
    let (lattice, namespace) = get_lattice().await?;
    let namespaces = lattice.list_namespaces().await?;
    println!("Lattice namespaces: {:?}", namespaces);

    assert!(namespaces.contains(&namespace));

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_list_namespace_components() -> Result<()> {
    let (lattice, namespace) = get_lattice().await?;
    let entity = Entity::provider(namespace);
    let components = lattice.list_components(entity).await?;
    println!("Components on namespace: {:?}", components);
    assert_eq!(
      components,
      vec![HostedType::Component(ComponentSignature {
        name: "test-component".to_owned(),
        inputs: vec![PortSignature {
          name: "input".to_owned(),
          type_string: "string".to_owned()
        }],
        outputs: vec![PortSignature {
          name: "output".to_owned(),
          type_string: "string".to_owned()
        }],
      })]
    );
    Ok(())
  }
}
