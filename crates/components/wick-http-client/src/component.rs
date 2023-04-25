use std::collections::HashMap;
use std::sync::Arc;

use bytes::Bytes;
use flow_component::{BoxFuture, Component, ComponentError, RuntimeCallback};
use futures::{Stream, StreamExt, TryStreamExt};
use parking_lot::Mutex;
use reqwest::{ClientBuilder, Method, Request};
use serde_json::Value;
use url::Url;
use wick_config::config::components::{Codec, HttpClientComponentConfig, HttpMethod};
use wick_config::config::{Metadata, UrlResource};
use wick_config::{ConfigValidation, Resolver};
use wick_interface_types::ComponentSignature;
use wick_packet::{FluxChannel, Invocation, Observer, Packet, PacketStream};

use crate::error::Error;
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive()]
pub(crate) struct Context {
  path_templates: HashMap<String, Arc<(liquid::Template, String)>>,
  client: reqwest::Client,
}

impl Context {}

impl std::fmt::Debug for Context {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Context").finish()
  }
}

#[derive(Debug, Clone)]
#[must_use]
pub struct HttpClientComponent {
  base: Url,
  signature: ComponentSignature,
  ctx: Arc<Mutex<Option<Context>>>,
  config: HttpClientComponentConfig,
}

impl HttpClientComponent {
  pub fn new(
    config: HttpClientComponentConfig,
    metadata: Metadata,
    resolver: &Resolver,
  ) -> Result<Self, ComponentError> {
    validate(&config, resolver)?;
    let addr: UrlResource = resolver(&config.resource)
      .ok_or_else(|| ComponentError::message(&format!("Could not resolve resource ID {}", config.resource)))
      .and_then(|r| r.try_resource().map_err(ComponentError::new))?
      .into();

    let mut sig = ComponentSignature::new("wick/component/http");
    sig.metadata.version = Some(metadata.version);
    sig.operations = config.operation_signatures();

    Ok(Self {
      signature: sig,
      base: (*addr).clone(),
      ctx: Default::default(),
      config,
    })
  }
}

impl Component for HttpClientComponent {
  fn handle(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    _data: Option<Value>,
    _callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let ctx = self.ctx.clone();
    let config = self.config.clone();
    let baseurl = self.base.clone();
    let codec = config.codec;

    Box::pin(async move {
      let (opdef, path_template, client) = match ctx.lock().as_ref() {
        Some(ctx) => {
          let opdef = config
            .operations
            .iter()
            .find(|op| op.name == invocation.target.operation_id())
            .cloned();
          let template = opdef.as_ref().and_then(|op| ctx.path_templates.get(&op.name).cloned());
          (opdef, template, ctx.client.clone())
        }
        None => return Err(ComponentError::message("Http client component not initialized")),
      };
      let (tx, rx) = PacketStream::new_channels();
      tokio::spawn(async move {
        let opdef = match opdef {
          Some(opdef) => opdef,
          None => {
            let _ = tx.send(Packet::component_error(format!(
              "Operation {} not found",
              invocation.target.operation_id()
            )));
            return;
          }
        };
        // Defer to operation codec, then to client codec, then to default.
        let codec = opdef.codec.unwrap_or(codec.unwrap_or_default());
        let template = path_template.unwrap();

        let input_list: Vec<_> = opdef.inputs.iter().map(|i| i.name.clone()).collect();
        let mut inputs = wick_packet::StreamMap::from_stream(stream, &input_list);

        'outer: loop {
          let inputs = match inputs.next_set().await {
            Ok(Some(inputs)) => inputs,
            Ok(None) => break 'outer,
            Err(e) => {
              let _ = tx.send(Packet::component_error(e.to_string()));
              break 'outer;
            }
          };
          trace!(inputs = ?inputs, "http:inputs");
          let inputs = inputs
            .into_iter()
            .map(|(k, v)| (k, v.deserialize::<liquid::model::Value>().unwrap()))
            .collect::<HashMap<_, _>>();

          let append_path = match template
            .0
            .render(&inputs)
            .map_err(|e| Error::PathTemplate(template.1.clone(), e.to_string()))
          {
            Ok(p) => p,
            Err(e) => {
              let _ = tx.send(Packet::component_error(e.to_string()));
              break 'outer;
            }
          };
          let request_url = baseurl.join(&append_path).unwrap();
          trace!(url= %request_url, "http:sending request");

          let method = match opdef.method {
            HttpMethod::Get => Method::GET,
            HttpMethod::Post => Method::POST,
            HttpMethod::Put => Method::PUT,
            HttpMethod::Delete => Method::DELETE,
          };
          let response = match client.execute(Request::new(method, request_url)).await {
            Ok(r) => r,
            Err(e) => {
              let _ = tx.send(Packet::component_error(e.to_string()));
              break 'outer;
            }
          };
          let (our_response, body_stream) = match crate::conversions::to_wick_response(response) {
            Ok(r) => r,
            Err(e) => {
              let _ = tx.send(Packet::component_error(e.to_string()));
              break 'outer;
            }
          };
          let _ = tx.send(Packet::encode("response", our_response));
          let _ = tx.send(Packet::done("response"));
          tokio::spawn(output_task(codec, Box::pin(body_stream), tx.clone()));
        }
      });

      Ok(rx)
    })
  }

  fn list(&self) -> &ComponentSignature {
    &self.signature
  }

  fn init(&self) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<(), ComponentError>> + Send + 'static>> {
    let container = self.ctx.clone();
    let config = self.config.clone();

    Box::pin(async move {
      let mut path_templates = HashMap::new();
      let parser = liquid::ParserBuilder::with_stdlib().build().unwrap();
      for ops in &config.operations {
        let template = parser.parse(&ops.path).map_err(ComponentError::new)?;
        path_templates.insert(ops.name.clone(), Arc::new((template, ops.path.clone())));
      }
      let ctx = Context {
        path_templates,
        client: ClientBuilder::new()
          .connect_timeout(std::time::Duration::from_secs(5))
          .user_agent(APP_USER_AGENT)
          .build()
          .unwrap(),
      };
      container.lock().replace(ctx);

      Ok(())
    })
  }
}

fn output_task(
  codec: Codec,
  mut body_stream: std::pin::Pin<Box<impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static>>,
  tx: FluxChannel<Packet, wick_packet::Error>,
) -> BoxFuture<'static, ()> {
  let task = async move {
    match codec {
      Codec::Json => {
        let bytes: Vec<Bytes> = match body_stream.try_collect().await {
          Ok(r) => r,
          Err(e) => {
            let _ = tx.send(Packet::component_error(e.to_string()));
            return;
          }
        };
        let bytes = bytes.concat();

        let json: Value = match serde_json::from_slice(&bytes) {
          Ok(r) => r,
          Err(e) => {
            let _ = tx.send(Packet::component_error(e.to_string()));
            return;
          }
        };
        let _ = tx.send(Packet::encode("body", json));
        let _ = tx.send(Packet::done("body"));
      }
      Codec::Raw => {
        let _ = tx.send(Packet::open_bracket("body"));
        while let Some(Ok(bytes)) = body_stream.next().await {
          let _ = tx.send(Packet::encode("body", bytes));
        }
        let _ = tx.send(Packet::close_bracket("body"));
      }
    }
  };
  Box::pin(task)
}

impl ConfigValidation for HttpClientComponent {
  type Config = HttpClientComponentConfig;

  fn validate(config: &Self::Config, resolver: &Resolver) -> Result<(), ComponentError> {
    Ok(validate(config, resolver)?)
  }
}

fn validate(_config: &HttpClientComponentConfig, _resolver: &Resolver) -> Result<(), Error> {
  Ok(())
}

#[cfg(test)]
mod test {
  use std::str::FromStr;

  use anyhow::Result;
  use flow_component::panic_callback;
  use futures::StreamExt;
  use wick_config::config::components::{HttpClientComponentConfig, HttpClientOperationDefinition};
  use wick_config::config::{AppConfiguration, ResourceDefinition};
  use wick_interface_types::{Field, TypeSignature};
  use wick_packet::{packet_stream, Entity};

  use super::*;

  #[test]
  fn test_component() {
    fn is_send_sync<T: Send + Sync>() {}
    is_send_sync::<HttpClientComponent>();
  }

  static GET_OP: &str = "get";

  fn get_config() -> (AppConfiguration, HttpClientComponentConfig) {
    let mut config = HttpClientComponentConfig {
      resource: "base".to_owned(),
      codec: Default::default(),
      operations: vec![],
    };
    let op = HttpClientOperationDefinition {
      name: GET_OP.to_owned(),
      path: "/get?query1={{input}}".to_owned(),
      inputs: vec![Field::new("input", TypeSignature::String)],
      codec: Default::default(),
      method: HttpMethod::Get,
    };
    config.operations.push(op);
    let mut app_config = wick_config::config::AppConfiguration::default();
    app_config.add_resource(
      "base",
      ResourceDefinition::Url(UrlResource::new(
        Url::from_str(&format!("http://{}", std::env::var("HTTPBIN").unwrap())).unwrap(),
      )),
    );

    (app_config, config)
  }

  async fn get_component(
    app_config: AppConfiguration,
    component_config: HttpClientComponentConfig,
  ) -> HttpClientComponent {
    let resolver = app_config.resolver();
    let component = HttpClientComponent::new(component_config, Metadata::default(), &resolver).unwrap();
    component.init().await.unwrap();
    component
  }

  #[test_logger::test(test)]
  fn test_validate() -> Result<()> {
    let (app_config, component_config) = get_config();

    let result = validate(&component_config, &app_config.resolver());
    assert_eq!(result, Ok(()));
    Ok(())
  }

  mod integration_test {
    use serde_json::json;
    use wick_interface_http::types::{HttpResponse, HttpVersion};

    use super::*;

    #[test_logger::test(tokio::test)]
    async fn test_get_request() -> Result<()> {
      let (app_config, component_config) = get_config();
      let comp = get_component(app_config, component_config).await;
      let invocation = Invocation::test("test_get_request", Entity::local(GET_OP), Default::default())?;
      let packets = packet_stream!(("input", "SENTINEL"));
      let mut stream = comp
        .handle(invocation, packets, None, panic_callback())
        .await?
        .collect::<Vec<_>>()
        .await;

      assert_eq!(stream.pop().unwrap(), Ok(Packet::done("body")));
      let response = stream.pop().unwrap().unwrap().deserialize_generic().unwrap();
      let response = response.get("args").unwrap();
      assert_eq!(response, &json!( {"query1": "SENTINEL"}));
      assert_eq!(stream.pop().unwrap(), Ok(Packet::done("response")));
      let response: HttpResponse = stream.pop().unwrap().unwrap().deserialize().unwrap();
      assert_eq!(response.version, HttpVersion::Http11);

      Ok(())
    }
  }
}
