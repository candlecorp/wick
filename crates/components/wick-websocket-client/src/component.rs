use std::collections::HashMap;
use std::sync::Arc;

use flow_component::{BoxFuture, Component, ComponentError, IntoComponentResult, RuntimeCallback};
use futures::stream::{SplitSink, SplitStream};
use futures::{Stream, StreamExt, TryStreamExt};
use serde_json::{Map, Value};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{connect_async, WebSocketStream};
use tracing::Span;
use url::Url;
use wick_config::config::components::{
  ComponentConfig,
  OperationConfig,
  WebSocketClientComponentConfig,
  WebSocketClientOperationDefinition,
};
use wick_config::config::{LiquidJsonConfig, Metadata, UrlResource};
use wick_config::{ConfigValidation, Resolver};
use wick_interface_types::{ComponentSignature, OperationSignatures};
use wick_packet::{Base64Bytes, FluxChannel, Invocation, Observer, Packet, PacketSender, PacketStream, RuntimeConfig};

use crate::error::Error;
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Debug, Clone)]
#[must_use]
pub struct WebSocketClientComponent {
  base: Url,
  signature: ComponentSignature,
  config: WebSocketClientComponentConfig,
  root_config: Option<RuntimeConfig>,
  path_templates: HashMap<String, Arc<(String, String)>>,
  sender: Arc<Mutex<Option<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
  receiver: Arc<Mutex<Option<SplitStream<WebSocketStream<TcpStream>>>>>,
}

impl WebSocketClientComponent {
  #[allow(clippy::needless_pass_by_value)]
  pub async fn new(
    config: WebSocketClientComponentConfig,
    root_config: Option<RuntimeConfig>,
    metadata: Option<Metadata>,
    resolver: &Resolver,
  ) -> Result<Self, ComponentError> {
    validate(&config, resolver)?;
    let addr: UrlResource = resolver(config.resource())
      .and_then(|r| r.try_resource())
      .and_then(|r| r.try_url())
      .into_component_error()?;

    let mut sig = ComponentSignature::new("wick/component/websocket");
    sig.metadata.version = metadata.map(|v| v.version().to_owned());
    sig.operations = config.operation_signatures();
    sig.config = config.config().to_vec();

    let url = addr
      .url()
      .value()
      .cloned()
      .ok_or_else(|| ComponentError::message("Internal Error - Invalid resource"))?;

    let mut path_templates = HashMap::new();
    for ops in config.operations() {
      path_templates.insert(
        ops.name().to_owned(),
        Arc::new((ops.path().to_owned(), ops.path().to_owned())),
      );
    }

    let sender = Arc::new(Mutex::new(None));
    let receiver = Arc::new(Mutex::new(None));

    Ok(Self {
      base: url,
      signature: sig,
      config,
      root_config,
      path_templates,
      sender,
      receiver,
    })
  }
}

impl Component for WebSocketClientComponent {
  fn handle(
    &self,
    invocation: Invocation,
    op_config: Option<RuntimeConfig>,
    _callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let config = self.config.clone();
    let baseurl = self.base.clone();
    let opdef = get_op_by_name(&config, invocation.target.operation_id());
    let path_template = opdef
      .as_ref()
      .and_then(|op| self.path_templates.get(op.name()).cloned());

    Box::pin(async move {
      let (tx, rx) = invocation.make_response();
      let span = invocation.span.clone();
      let fut = handle(
        opdef,
        tx.clone(),
        invocation,
        self.root_config.clone(),
        op_config,
        path_template,
        baseurl,
        self.sender.clone(),
        self.receiver.clone(),
      );
      tokio::spawn(async move {
        if let Err(e) = fut.await {
          span.in_scope(|| error!(error = %e, "websocket:client"));
          let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
        }
      });
      Ok(rx)
    })
  }

  fn signature(&self) -> &ComponentSignature {
    &self.signature
  }
}

fn get_op_by_name(config: &WebSocketClientComponentConfig, name: &str) -> Option<WebSocketClientOperationDefinition> {
  config.operations().iter().find(|op| op.name() == name).cloned()
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
async fn handle(
  opdef: Option<WebSocketClientOperationDefinition>,
  tx: FluxChannel<Packet, wick_packet::Error>,
  mut invocation: Invocation,
  root_config: Option<RuntimeConfig>,
  op_config: Option<RuntimeConfig>,
  path_template: Option<Arc<(String, String)>>,
  baseurl: Url,
  sender: Arc<Mutex<Option<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
  receiver: Arc<Mutex<Option<SplitStream<WebSocketStream<TcpStream>>>>>,
) -> anyhow::Result<()> {
  if baseurl.cannot_be_a_base() {
    return Err(Error::InvalidBaseUrl(baseurl).into());
  }
  let opdef = match opdef {
    Some(opdef) => opdef,
    None => {
      return Err(Error::OpNotFound(invocation.target.operation_id().to_owned()).into());
    }
  };
  let template = path_template.unwrap();

  let input_list: Vec<_> = opdef.inputs().iter().map(|i| i.name.clone()).collect();
  let mut inputs = wick_packet::StreamMap::from_stream(invocation.eject_stream(), input_list);
  let mut handles = Vec::new();

  'outer: loop {
    let inputs = match inputs.next_set().await {
      Ok(Some(inputs)) => inputs,
      Ok(None) => break 'outer,
      Err(e) => {
        let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
        break 'outer;
      }
    };

    if inputs.values().all(|v| v.is_done()) {
      break 'outer;
    }
    let inputs: Map<String, Value> = inputs
      .into_iter()
      .map(|(k, v)| {
        let v = v
          .decode_value()
          .map_err(|e| {
            invocation.trace(|| error!(port=%k,error=%e, "websocket:client"));
            e
          })
          .unwrap_or(Value::Null);
        (k, v)
      })
      .collect();
    let inputs = Value::Object(inputs);

    invocation.trace(|| trace!(inputs=?inputs, "websocket:client:inputs"));
    let ctx = LiquidJsonConfig::make_context(
      Some(inputs),
      root_config.as_ref(),
      op_config.as_ref(),
      None,
      Some(&invocation.inherent),
    )?;

    let message = match opdef.message() {
      Some(message) => match message.render(&ctx) {
        Ok(p) => Some(p),
        Err(e) => {
          let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
          break 'outer;
        }
      },
      None => None,
    };

    let append_path = match liquid_json::render_string(&template.0, &ctx)
      .map_err(|e| Error::PathTemplate(template.1.clone(), e.to_string()))
    {
      Ok(p) => p,
      Err(e) => {
        let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
        break 'outer;
      }
    };

    let mut request_url = baseurl.clone();
    let (path, query) = append_path.split_once('?').unwrap_or((&append_path, ""));
    request_url.set_path(&format!("{}{}", request_url.path(), path));
    request_url.set_query((!query.is_empty()).then_some(query));

    invocation.trace(|| trace!(url= %request_url, "websocket:client:connect"));

    // Establish WebSocket Connection
    let (ws_stream, _) = connect_async(baseurl.clone()).await?;
    let (sink, stream) = ws_stream.split();

    // Update sender and receiver
    *sender.lock().await = Some(sink);
    *receiver.lock().await = Some(stream);

    // Start a task for sending messages to the server
    let sender = sender.clone();
    tokio::spawn(async move {
      while let Some(message) = inputs.next().await {
        if let Some(ref mut sender) = *sender.lock().await {
          sender.send(Message::Text(message.to_string())).await?;
        }
      }
      Ok(())
    });

    // Start a task for receiving messages from the server
    let receiver = receiver.clone();
    tokio::spawn(async move {
      if let Some(ref mut receiver) = *receiver.lock().await {
        while let Some(message) = receiver.next().await {
          match message {
            Ok(Message::Text(text)) => {
              tx.send(Packet::encode("response", text)).await?;
            }
            // Handle other types of messages or errors
            _ => {}
          }
        }
      }
      Ok(())
    });
  }
  Ok(())
}

impl ConfigValidation for WebSocketClientComponent {
  type Config = WebSocketClientComponent;

  fn validate(config: &Self::Config, resolver: &Resolver) -> Result<(), ComponentError> {
    Ok(validate(config, resolver)?)
  }
}

fn validate(_config: &WebSocketClientComponent, _resolver: &Resolver) -> Result<(), Error> {
  Ok(())
}
