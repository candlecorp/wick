use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use log::{
  debug,
  trace,
};
use nats::jetstream::{
  AckPolicy,
  Consumer,
  ConsumerConfig,
  StreamConfig,
  StreamInfo,
};
use nats::{
  Connection,
  Message,
  Subscription,
};
use parking_lot::Mutex;
use serde::Deserialize;
use tokio::runtime::{
  Builder,
  Runtime,
};
use tokio::time::timeout;
use vino_codec::messagepack::deserialize;

use crate::error::LatticeError;

type Result<T> = std::result::Result<T, LatticeError>;
static RPC_STREAM_NAME: &str = "rpc";

#[derive(Debug, Clone)]
pub struct NatsOptions {
  pub address: String,
  pub client_id: String,
  pub creds_path: Option<PathBuf>,
  pub token: Option<String>,
  pub timeout: Duration,
}

lazy_static::lazy_static! {
  static ref RT: Runtime = {
    Builder::new_multi_thread().thread_name("lattice")
    .build().unwrap()
  };
}

#[derive(Clone, Debug)]
pub(crate) struct Nats {
  nc: Connection,
  pub timeout: Duration,
}

impl Nats {
  pub(crate) async fn connect(nopts: NatsOptions) -> Result<Self> {
    let opts = if let Some(creds_path) = &nopts.creds_path {
      nats::Options::with_credentials(creds_path)
    } else if let Some(token) = &nopts.token {
      nats::Options::with_token(token)
    } else {
      nats::Options::new()
    };

    let timeout = nopts.timeout;

    let nc = RT
      .spawn_blocking(move || {
        trace!(
          "LATTICE:CONNECT[{}]:ID[{}]:TIMEOUT[{:?}]",
          nopts.address,
          nopts.client_id,
          nopts.timeout
        );
        opts
          .with_name(&nopts.client_id)
          .connect(&nopts.address)
          .map_err(LatticeError::ConnectionError)
      })
      .await??;

    Ok(Self { nc, timeout })
  }

  pub(crate) async fn create_stream(&self, stream_config: StreamConfig) -> Result<StreamInfo> {
    let nc = self.nc.clone();
    Ok(
      RT.spawn_blocking(move || nc.create_stream(stream_config))
        .await??,
    )
  }

  pub(crate) async fn create_consumer(&self, namespace: String) -> Result<NatsConsumer> {
    let topic = format!("{}.{}", RPC_STREAM_NAME, namespace);
    let consumer_config = ConsumerConfig {
      durable_name: Some(namespace.to_owned()),
      deliver_subject: None, //make pull based
      ack_policy: AckPolicy::Explicit,
      filter_subject: topic.clone(),
      ..Default::default()
    };

    let nc = self.nc.clone();
    let rpc_stream_topic = RPC_STREAM_NAME.to_owned();
    debug!("LATTICE:CONSUMER[{},{}]:CREATE", rpc_stream_topic, topic);
    let c = RT
      .spawn_blocking(move || Consumer::create_or_open(nc, &rpc_stream_topic, consumer_config))
      .await??;

    let consumer = NatsConsumer {
      topic,
      consumer: Arc::new(Mutex::new(c)),
    };
    Ok(consumer)
  }

  pub(crate) async fn publish(&self, topic: String, payload: Vec<u8>) -> Result<()> {
    let nc = self.nc.clone();
    RT.spawn_blocking(move || {
      debug!("LATTICE:PUBLISH[{}]:LEN[{}]", topic, payload.len());
      trace!("LATTICE:PUBLISH[{}]:DATA:{:?}", topic, payload);
      nc.publish(&topic, payload)
        .map_err(|e| LatticeError::PublishFail(e.to_string()))
    })
    .await?
  }

  pub(crate) async fn new_inbox(&self) -> String {
    self.nc.new_inbox()
  }

  pub(crate) async fn subscribe(&self, topic: String) -> Result<NatsSubscription> {
    let nc = self.nc.clone();
    let sub = RT
      .spawn_blocking(move || {
        debug!("LATTICE:SUBSCRIBE[{}]", topic);
        nc.subscribe(&topic)
          .map_err(|e| LatticeError::PublishFail(e.to_string()))
      })
      .await??;
    Ok(NatsSubscription {
      inner: sub,
      timeout: self.timeout,
    })
  }

  pub(crate) async fn list_consumers(&self, stream_name: String) -> Result<Vec<String>> {
    let nc = self.nc.clone();

    let result = RT
      .spawn_blocking(move || match nc.list_consumers(stream_name) {
        Ok(iter) => {
          let pages = iter.collect::<Vec<_>>();
          let mut all = vec![];
          for page in pages {
            match page {
              Ok(page) => all.push(page.name),
              Err(e) => return Err(LatticeError::ListFail(e.to_string())),
            }
          }
          Ok(all)
        }
        Err(e) => Err(LatticeError::ListFail(e.to_string())),
      })
      .await??;
    Ok(result)
  }

  pub(crate) async fn stream_info(&self, stream_name: String) -> Result<StreamInfo> {
    let nc = self.nc.clone();
    let result = RT
      .spawn_blocking(move || nc.stream_info(stream_name))
      .await??;
    Ok(result)
  }
}

pub(crate) struct NatsSubscription {
  inner: Subscription,
  timeout: Duration,
}

impl NatsSubscription {
  pub(crate) async fn next(&self) -> Result<Option<Message>> {
    let sub = self.inner.clone();
    let task = RT.spawn_blocking(move || {
      trace!("LATTICE:SUB:WAIT");
      let a = sub.next();
      trace!("LATTICE:SUB:CONTINUE");
      a
    });
    let result = timeout(self.timeout, task)
      .await
      .map_err(|_| LatticeError::WaitTimeout)??;

    Ok(result)
  }
}

pub(crate) struct NatsConsumer {
  topic: String,
  consumer: Arc<Mutex<Consumer>>,
}

impl NatsConsumer {
  pub(crate) async fn next(&mut self) -> Result<NatsMessage> {
    let consumer = self.consumer.clone();
    let topic = self.topic.clone();

    let result = RT
      .spawn_blocking(move || {
        let mut lock = consumer.lock();
        trace!("LATTICE:HANDLER[{}]:WAIT", topic);
        let result = match lock.pull() {
          Ok(msg) => Ok(msg.into()),
          Err(e) => Err(LatticeError::SubscribeFail(e.to_string())),
        };
        trace!("LATTICE:HANDLER[{}]:CONTINUE", topic);
        result
      })
      .await?;
    result
  }
}

pub(crate) struct NatsMessage {
  inner: Arc<Message>,
}

impl From<Message> for NatsMessage {
  fn from(msg: Message) -> Self {
    Self {
      inner: Arc::new(msg),
    }
  }
}

impl NatsMessage {
  pub async fn ack(&self) -> Result<()> {
    trace!(
      "LATTICE:ACK[{}]",
      self.inner.reply.clone().unwrap_or_default()
    );
    let msg = self.inner.clone();
    Ok(RT.spawn_blocking(move || msg.ack()).await??)
  }

  pub fn subject(&self) -> &str {
    &self.inner.subject
  }

  pub fn deserialize<'de, T: Deserialize<'de>>(&'de self) -> Result<T> {
    deserialize(&self.inner.data).map_err(|e| LatticeError::MessageDeserialization(e.to_string()))
  }
}
