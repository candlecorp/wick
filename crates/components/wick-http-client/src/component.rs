use std::collections::HashMap;
use std::sync::Arc;

use flow_component::{BoxFuture, Component, ComponentError, RuntimeCallback};
use futures::{Stream, StreamExt, TryStreamExt};
use parking_lot::Mutex;
use reqwest::{ClientBuilder, Method, Request, RequestBuilder};
use serde_json::{Map, Value};
use url::Url;
use wick_config::config::components::{Codec, HttpClientComponentConfig, HttpClientOperationDefinition, HttpMethod};
use wick_config::config::{Metadata, UrlResource};
use wick_config::{ConfigValidation, Resolver};
use wick_interface_types::ComponentSignature;
use wick_packet::{Base64Bytes, FluxChannel, Invocation, Observer, OperationConfig, Packet, PacketStream};

use crate::error::Error;
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive()]
pub(crate) struct Context {
  path_templates: HashMap<String, Arc<(String, String)>>,
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

fn get_op_by_name(config: &HttpClientComponentConfig, name: &str) -> Option<HttpClientOperationDefinition> {
  config.operations.iter().find(|op| op.name == name).cloned()
}

#[allow(clippy::too_many_arguments)]
async fn handle(
  opdef: Option<HttpClientOperationDefinition>,
  tx: FluxChannel<Packet, wick_packet::Error>,
  invocation: Invocation,
  config: Option<OperationConfig>,
  codec: Option<Codec>,
  path_template: Option<Arc<(String, String)>>,
  stream: PacketStream,
  baseurl: Url,
  client: reqwest::Client,
) -> anyhow::Result<()> {
  let opdef = match opdef {
    Some(opdef) => opdef,
    None => {
      return Err(Error::OpNotFound(invocation.target.operation_id().to_owned()).into());
    }
  };
  // Defer to operation codec, then to client codec, then to default.
  let codec = opdef.codec.unwrap_or(codec.unwrap_or_default());
  let template = path_template.unwrap();

  let input_list: Vec<_> = opdef.inputs.iter().map(|i| i.name.clone()).collect();
  let mut inputs = wick_packet::StreamMap::from_stream(stream, input_list);

  'outer: loop {
    let inputs = match inputs.next_set().await {
      Ok(Some(inputs)) => inputs,
      Ok(None) => break 'outer,
      Err(e) => {
        let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
        break 'outer;
      }
    };
    trace!(inputs = ?inputs, "http:inputs");
    let mut inputs: Map<String, Value> = inputs
      .into_iter()
      .map(|(k, v)| (k, v.deserialize_generic().unwrap()))
      .collect();
    if let Some(config) = &config {
      for (k, v) in config.iter() {
        inputs.insert(k.clone(), v.clone());
      }
    }
    let inputs = Value::Object(inputs);
    trace!(inputs = ?inputs, "http:inputs");

    let body = match &opdef.body {
      Some(body) => match body.render(&inputs) {
        Ok(p) => Some(p),
        Err(e) => {
          let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
          break 'outer;
        }
      },
      None => None,
    };

    let append_path = match liquid_json::render_string(&template.0, &inputs)
      .map_err(|e| Error::PathTemplate(template.1.clone(), e.to_string()))
    {
      Ok(p) => p,
      Err(e) => {
        let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
        break 'outer;
      }
    };
    let request_url = baseurl.join(&append_path).unwrap();
    trace!(url= %request_url, "http:sending request");

    let request = match opdef.method {
      HttpMethod::Get => Request::new(Method::GET, request_url),
      HttpMethod::Post => Request::new(Method::POST, request_url),
      HttpMethod::Put => Request::new(Method::PUT, request_url),
      HttpMethod::Delete => Request::new(Method::DELETE, request_url),
    };
    let request_builder = RequestBuilder::from_parts(client.clone(), request);
    let mut request_builder = if let Some(body) = body {
      if codec == Codec::Json {
        request_builder.json(&body)
      } else {
        unimplemented!("http: non-json body not implemented yet");
      }
    } else {
      request_builder
    };
    for (header, values) in &opdef.headers {
      for value in values {
        let Ok(value) =  liquid_json::render_string(value, &inputs) else {
          let _ = tx.error(wick_packet::Error::component_error(format!("Can't render template {}", value)));
          break 'outer;
        };
        request_builder = request_builder.header(header, value);
      }
    }
    let response = match request_builder.send().await {
      Ok(r) => r,
      Err(e) => {
        let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
        break 'outer;
      }
    };
    let (our_response, body_stream) = match crate::conversions::to_wick_response(response) {
      Ok(r) => r,
      Err(e) => {
        let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
        break 'outer;
      }
    };
    let _ = tx.send(Packet::encode("response", our_response));
    let _ = tx.send(Packet::done("response"));
    tokio::spawn(output_task(codec, Box::pin(body_stream), tx.clone()));
  }
  Ok(())
}

impl Component for HttpClientComponent {
  fn handle(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    data: Option<OperationConfig>,
    _callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let ctx = self.ctx.clone();
    let config = self.config.clone();
    let baseurl = self.base.clone();
    let codec = config.codec;

    Box::pin(async move {
      let (opdef, path_template, client) = match ctx.lock().as_ref() {
        Some(ctx) => {
          let opdef = get_op_by_name(&config, invocation.target.operation_id());
          let template = opdef.as_ref().and_then(|op| ctx.path_templates.get(&op.name).cloned());
          (opdef, template, ctx.client.clone())
        }
        None => return Err(ComponentError::message("Http client component not initialized")),
      };
      let (tx, rx) = PacketStream::new_channels();
      let fut = handle(
        opdef,
        tx.clone(),
        invocation,
        data,
        codec,
        path_template,
        stream,
        baseurl,
        client,
      );
      tokio::spawn(async move {
        if let Err(e) = fut.await {
          let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
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
      for ops in &config.operations {
        path_templates.insert(ops.name.clone(), Arc::new((ops.path.clone(), ops.path.clone())));
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
  mut body_stream: std::pin::Pin<Box<impl Stream<Item = Result<Base64Bytes, reqwest::Error>> + Send + 'static>>,
  tx: FluxChannel<Packet, wick_packet::Error>,
) -> BoxFuture<'static, ()> {
  let task = async move {
    match codec {
      Codec::Json => {
        let bytes: Vec<Base64Bytes> = match body_stream.try_collect().await {
          Ok(r) => r,
          Err(e) => {
            let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
            return;
          }
        };
        let bytes = bytes.concat();

        let json: Value = match serde_json::from_slice(&bytes) {
          Ok(r) => r,
          Err(e) => {
            let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
            return;
          }
        };
        trace!(json = %json, "http:response");
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
  use serde_json::json;
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
  static POST_OP: &str = "post";

  fn get_config() -> (AppConfiguration, HttpClientComponentConfig) {
    let mut config = HttpClientComponentConfig {
      resource: "base".to_owned(),
      codec: Default::default(),
      operations: vec![],
    };
    config.operations.push(
      HttpClientOperationDefinition::new_get(
        GET_OP,
        "/get?query1={{input}}&query2={{secret}}",
        vec![Field::new("input", TypeSignature::String)],
      )
      .config([Field::new("secret", TypeSignature::String)])
      .build()
      .unwrap(),
    );
    config.operations.push(
      HttpClientOperationDefinition::new_post(
        POST_OP,
        "/post?query1={{input}}",
        vec![
          Field::new("input", TypeSignature::String),
          Field::new("number", TypeSignature::I64),
        ],
        Some(json!({"key": "{{input}}","other":"{{number}}"}).into()),
      )
      .build()
      .unwrap(),
    );

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
      let config = json!({"secret":"0xDEADBEEF"});
      let mut stream = comp
        .handle(invocation, packets, Some(config.try_into()?), panic_callback())
        .await?
        .collect::<Vec<_>>()
        .await;

      assert_eq!(stream.pop().unwrap(), Ok(Packet::done("body")));
      let response = stream.pop().unwrap().unwrap().deserialize_generic().unwrap();
      let response = response.get("args").unwrap();
      assert_eq!(response, &json!( {"query1": "SENTINEL","query2": "0xDEADBEEF"}));
      assert_eq!(stream.pop().unwrap(), Ok(Packet::done("response")));
      let response: HttpResponse = stream.pop().unwrap().unwrap().deserialize().unwrap();
      assert_eq!(response.version, HttpVersion::Http11);

      Ok(())
    }

    #[test_logger::test(tokio::test)]
    async fn test_post_request() -> Result<()> {
      let (app_config, component_config) = get_config();
      let comp = get_component(app_config, component_config).await;
      let invocation = Invocation::test("test_post_request", Entity::local(POST_OP), Default::default())?;
      let packets = packet_stream!(("input", "SENTINEL"), ("number", 123));
      let mut stream = comp
        .handle(invocation, packets, None, panic_callback())
        .await?
        .collect::<Vec<_>>()
        .await;

      assert_eq!(stream.pop().unwrap(), Ok(Packet::done("body")));
      let response = stream.pop().unwrap().unwrap();
      println!("{:?}", response);
      let response = response.deserialize_generic().unwrap();
      let args = response.get("args").unwrap();
      assert_eq!(args, &json!( {"query1": "SENTINEL"}));
      let data = response.get("json").unwrap();
      assert_eq!(data, &json!( {"key": "SENTINEL","other":123}));
      assert_eq!(stream.pop().unwrap(), Ok(Packet::done("response")));
      let response: HttpResponse = stream.pop().unwrap().unwrap().deserialize().unwrap();
      assert_eq!(response.version, HttpVersion::Http11);

      Ok(())
    }
  }
}
