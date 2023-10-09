use std::collections::HashMap;
use std::sync::Arc;

use anyhow::anyhow;
use eventsource_stream::Eventsource;
use flow_component::{BoxFuture, Component, ComponentError, RuntimeCallback};
use futures::{Stream, StreamExt, TryStreamExt};
use reqwest::header::CONTENT_TYPE;
use reqwest::{ClientBuilder, Method, Request, RequestBuilder};
use serde_json::{Map, Value};
use tracing::Span;
use url::Url;
use wick_config::config::components::{
  ComponentConfig,
  HttpClientComponentConfig,
  HttpClientOperationDefinition,
  OperationConfig,
};
use wick_config::config::{Codec, HttpEvent, HttpMethod, LiquidJsonConfig, Metadata, UrlResource};
use wick_config::{ConfigValidation, Resolver};
use wick_interface_types::{ComponentSignature, OperationSignatures};
use wick_packet::{Base64Bytes, FluxChannel, Invocation, Observer, Packet, PacketSender, PacketStream, RuntimeConfig};

use crate::error::Error;
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Debug, Clone)]
#[must_use]
pub struct HttpClientComponent {
  base: Url,
  signature: ComponentSignature,
  config: HttpClientComponentConfig,
  root_config: Option<RuntimeConfig>,
  path_templates: HashMap<String, Arc<(String, String)>>,
  client: reqwest::Client,
}

impl HttpClientComponent {
  #[allow(clippy::needless_pass_by_value)]
  pub fn new(
    config: HttpClientComponentConfig,
    root_config: Option<RuntimeConfig>,
    metadata: Option<Metadata>,
    resolver: &Resolver,
  ) -> Result<Self, ComponentError> {
    validate(&config, resolver)?;
    let addr: UrlResource = resolver(config.resource())
      .and_then(|r| r.try_resource())
      .and_then(|r| r.try_url())?;

    let proxy_details = config.proxy().map(|proxy| {
      let addr = resolver(proxy.resource())
        .and_then(|r| r.try_resource())
        .and_then(|r| r.try_url())
        .ok();

      let username = proxy.username();
      let password = proxy.password();

      (addr, username, password)
    });

    let (proxy_addr, proxy_username, proxy_password) = match proxy_details {
      Some((addr, user, pass)) => (addr, Some(user).unwrap(), Some(pass).unwrap()),
      None => (None, None, None),
    };

    let timeout = config.timeout().map_or_else(
      || std::time::Duration::from_secs(5),
      |timeout| std::time::Duration::from_secs(u64::from(timeout)),
    );

    let mut sig = ComponentSignature::new_named("wick/component/http");
    sig.metadata.version = metadata.map(|v| v.version().to_owned());
    sig.operations = config.operation_signatures();
    sig.config = config.config().to_vec();

    let url = addr
      .url()
      .value()
      .cloned()
      .ok_or_else(|| anyhow!("Internal Error - Invalid resource"))?;

    let mut path_templates = HashMap::new();
    for ops in config.operations() {
      path_templates.insert(
        ops.name().to_owned(),
        Arc::new((ops.path().to_owned(), ops.path().to_owned())),
      );
    }

    if proxy_addr == Some(addr.clone()) {
      return Err(Error::ProxyLoop(addr.url().value().unwrap().clone()).into());
    }

    let client = match proxy_addr {
      Some(proxy_addr) => {
        let mut proxy = reqwest::Proxy::all(proxy_addr.url().value().unwrap().clone())?;

        if let (Some(username), Some(password)) = (&proxy_username, &proxy_password) {
          proxy = proxy.basic_auth(username.as_str(), password.as_str());
        }

        ClientBuilder::new()
          .proxy(proxy)
          .connect_timeout(timeout)
          .user_agent(APP_USER_AGENT)
          .build()?
      }
      None => ClientBuilder::new()
        .connect_timeout(timeout)
        .user_agent(APP_USER_AGENT)
        .build()
        .unwrap(),
    };

    Ok(Self {
      signature: sig,
      base: url,
      path_templates,
      client,
      root_config,
      config,
    })
  }
}

impl Component for HttpClientComponent {
  fn handle(
    &self,
    invocation: Invocation,
    op_config: Option<RuntimeConfig>,
    _callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let config = self.config.clone();
    let baseurl = self.base.clone();
    let codec = config.codec().copied();
    let opdef = get_op_by_name(&config, invocation.target().operation_id());
    let path_template = opdef
      .as_ref()
      .and_then(|op| self.path_templates.get(op.name()).cloned());
    let client = self.client.clone();

    Box::pin(async move {
      let (tx, rx) = invocation.make_response();
      let span = invocation.span().clone();
      let fut = handle(
        opdef,
        tx.clone(),
        invocation,
        self.root_config.clone(),
        op_config,
        codec,
        path_template,
        baseurl,
        client,
      );
      tokio::spawn(async move {
        if let Err(e) = fut.await {
          span.in_scope(|| error!(error = %e, "http:client"));
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

fn get_op_by_name(config: &HttpClientComponentConfig, name: &str) -> Option<HttpClientOperationDefinition> {
  config.operations().iter().find(|op| op.name() == name).cloned()
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
async fn handle(
  opdef: Option<HttpClientOperationDefinition>,
  tx: FluxChannel<Packet, wick_packet::Error>,
  invocation: Invocation,
  root_config: Option<RuntimeConfig>,
  op_config: Option<RuntimeConfig>,
  codec: Option<Codec>,
  path_template: Option<Arc<(String, String)>>,
  baseurl: Url,
  client: reqwest::Client,
) -> anyhow::Result<()> {
  if baseurl.cannot_be_a_base() {
    return Err(Error::InvalidBaseUrl(baseurl).into());
  }
  let Some(opdef) = opdef else {
    return Err(Error::OpNotFound(invocation.target().operation_id().to_owned()).into());
  };
  // Defer to operation codec, then to client codec, then to default.
  let codec = opdef.codec().copied().unwrap_or(codec.unwrap_or_default());
  let template = path_template.unwrap();

  let input_list: Vec<_> = opdef.inputs().iter().map(|i| i.name.clone()).collect();
  let (invocation, stream) = invocation.split();
  let mut inputs = wick_packet::StreamMap::from_stream(stream, input_list);
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
            invocation.trace(|| error!(port=%k,error=%e, "http:client"));
            e
          })
          .unwrap_or(Value::Null);
        (k, v)
      })
      .collect();
    let inputs = Value::Object(inputs);

    invocation.trace(|| trace!(inputs=?inputs, "http:client:inputs"));
    let ctx = LiquidJsonConfig::make_context(
      Some(inputs),
      root_config.as_ref(),
      op_config.as_ref(),
      None,
      Some(&invocation.inherent),
    )?;

    let body = match opdef.body() {
      Some(body) => match body.render(&ctx) {
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

    invocation.trace(|| trace!(url= %request_url,body=?body, "http:client:request"));

    let request = match opdef.method() {
      HttpMethod::Get => Request::new(Method::GET, request_url),
      HttpMethod::Post => Request::new(Method::POST, request_url),
      HttpMethod::Put => Request::new(Method::PUT, request_url),
      HttpMethod::Delete => Request::new(Method::DELETE, request_url),
    };

    let request_builder = RequestBuilder::from_parts(client.clone(), request);
    let mut request_builder = if let Some(body) = body {
      match codec {
        Codec::Json => request_builder.json(&body),
        Codec::Raw => {
          unimplemented!("raw bodies not supported yet")
        }
        Codec::FormData => request_builder.form(&body),
        Codec::Text => request_builder.body(body.to_string()),
        Codec::EventStream => {
          unimplemented!("Event stream is not a valid client content-type")
        }
      }
    } else {
      request_builder
    };

    if let Some(headers) = opdef.headers() {
      for (header, values) in headers {
        for value in values {
          let Ok(value) = liquid_json::render_string(value, &ctx) else {
            let _ = tx.error(wick_packet::Error::component_error(format!(
              "Can't render template {}",
              value
            )));
            break 'outer;
          };
          request_builder = request_builder.header(header, value);
        }
      }
    }

    let (client, request) = request_builder.build_split();
    let request = request.unwrap();

    invocation.trace(|| debug!(request=?request, "http:client:request"));

    let response = match client.execute(request).await {
      Ok(r) => r,
      Err(e) => {
        let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
        break 'outer;
      }
    };

    invocation.trace(|| debug!(status=%response.status(), "http:client:response_status"));

    let codec = response.headers().get(CONTENT_TYPE).map_or(Codec::Raw, |value| {
      let value = value.to_str().unwrap();
      let (value, _other) = value.split_once(';').unwrap_or((value, ""));
      match value {
        "application/json" => Codec::Json,
        "application/x-www-form-urlencoded" => Codec::FormData,
        "text/event-stream" => Codec::EventStream,
        _ => Codec::Raw,
      }
    });

    let (our_response, body_stream) = match crate::conversions::to_wick_response(response) {
      Ok(r) => r,
      Err(e) => {
        let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
        break 'outer;
      }
    };
    invocation.trace(|| debug!(response = ?our_response, "http:client:response"));

    let _ = tx.send(Packet::encode("response", our_response));
    handles.push(tokio::spawn(output_task(
      invocation.span.clone(),
      codec,
      Box::pin(body_stream),
      tx.clone(),
    )));
  }
  let _ = tx.send(Packet::done("response"));
  let _ = futures::future::join_all(handles).await;
  let _ = tx.send(Packet::done("body"));

  Ok(())
}

fn output_task(
  span: Span,
  codec: Codec,
  mut body_stream: std::pin::Pin<Box<impl Stream<Item = Result<Base64Bytes, reqwest::Error>> + Send + 'static>>,
  tx: PacketSender,
) -> BoxFuture<'static, ()> {
  let task = async move {
    match codec {
      Codec::EventStream => {
        let mut stream = body_stream.map(Into::into).eventsource();
        while let Some(event) = stream.next().await {
          match event {
            Ok(event) => {
              let wick_event = HttpEvent::new(Some(event.event), event.data, Some(event.id), event.retry);
              span.in_scope(|| debug!("{} {}", format!("{:?}", wick_event), "http:client:response_body"));
              let _ = tx.send(Packet::encode("body", wick_event));
            }
            Err(e) => {
              let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
              return;
            }
          }
        }
      }
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
        span.in_scope(|| trace!(%json, "http:client:response_body"));
        let _ = tx.send(Packet::encode("body", json));
      }
      Codec::Raw => {
        let _ = tx.send(Packet::open_bracket("body"));
        while let Some(Ok(bytes)) = body_stream.next().await {
          span.in_scope(|| debug!(len = bytes.len(), "http:client:response_body"));

          let _ = tx.send(Packet::encode("body", bytes));
        }
        let _ = tx.send(Packet::close_bracket("body"));
      }
      Codec::FormData => unreachable!("Form data on the response is not supported."),
      Codec::Text => {
        let bytes: Vec<Base64Bytes> = match body_stream.try_collect().await {
          Ok(r) => r,
          Err(e) => {
            let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
            return;
          }
        };
        let bytes = bytes.concat();

        let text = match String::from_utf8(bytes) {
          Ok(r) => r,
          Err(e) => {
            let _ = tx.error(wick_packet::Error::component_error(e.to_string()));
            return;
          }
        };
        span.in_scope(|| trace!(%text, "response body"));
        let _ = tx.send(Packet::encode("body", text));
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
  use wick_config::config::components::{
    HttpClientComponentConfig,
    HttpClientComponentConfigBuilder,
    HttpClientOperationDefinition,
  };
  use wick_config::config::{AppConfiguration, ResourceDefinition};
  use wick_interface_types::{Field, Type};
  use wick_packet::{packet_stream, Entity};

  use super::*;

  #[test]
  const fn test_component() {
    const fn is_send_sync<T: Send + Sync>() {}
    is_send_sync::<HttpClientComponent>();
  }

  static GET_OP: &str = "get";
  static POST_OP: &str = "post";
  static POST_OP_TEXT: &str = "post_text";

  fn get_config() -> (AppConfiguration, HttpClientComponentConfig) {
    let mut config = HttpClientComponentConfigBuilder::default()
      .resource("base")
      .codec(Codec::Json)
      .build()
      .unwrap();

    let get_headers = Some(HashMap::from([(
      "Authorization".to_owned(),
      vec!["Bearer {{ctx.config.secret}}".to_owned()],
    )]));

    config.operations_mut().push(
      HttpClientOperationDefinition::new_get(
        GET_OP,
        "get?query1={{input}}&query2={{ctx.config.secret}}",
        vec![Field::new("input", Type::String)],
        get_headers,
      )
      .config([Field::new("secret", Type::String)])
      .build()
      .unwrap(),
    );

    let post_headers = Some(HashMap::from([(
      "X-Custom-Header".to_owned(),
      vec!["{{input}}".to_owned()],
    )]));

    config.operations_mut().push(
      HttpClientOperationDefinition::new_post(
        POST_OP,
        "post?query1={{input}}",
        vec![
          Field::new("input", Type::String),
          Field::new(
            "number",
            Type::List {
              ty: Box::new(Type::I64),
            },
          ),
        ],
        Some(json!({"key": "{{input}}","other":"{{number | each: '{\"value\": {{el}} }' | json | output }}"}).into()),
        post_headers.clone(),
      )
      .build()
      .unwrap(),
    );

    config.operations_mut().push(
      HttpClientOperationDefinition::new_post(
        POST_OP_TEXT,
        "post?query1={{input}}",
        vec![Field::new("input", Type::String), Field::new("payload", Type::String)],
        Some(json!({"key": "{{input}}","other":"{{number | each: '{\"value\": {{el}} }' | json | output }}"}).into()),
        post_headers,
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

  fn get_component(app_config: &AppConfiguration, component_config: HttpClientComponentConfig) -> HttpClientComponent {
    let resolver = app_config.resolver();
    HttpClientComponent::new(component_config, None, None, &resolver).unwrap()
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
      let comp = get_component(&app_config, component_config);
      let packets = packet_stream!(("input", "SENTINEL"));
      let invocation = Invocation::test("test_get_request", Entity::local(GET_OP), packets, Default::default())?;
      let config = json!({"secret":"0xDEADBEEF"});
      let mut stream = comp
        .handle(invocation, Some(RuntimeConfig::from_value(config)?), panic_callback())
        .await?
        .collect::<Vec<_>>()
        .await;

      assert_eq!(stream.pop().unwrap(), Ok(Packet::done("body")));
      let response = stream.pop().unwrap().unwrap().decode_value().unwrap();
      let response_args = response.get("args").unwrap();
      assert_eq!(response_args, &json!( {"query1": "SENTINEL","query2": "0xDEADBEEF"}));
      let response_headers = response.get("headers").unwrap();
      assert_eq!(
        response_headers.get("Authorization").unwrap(),
        &json!("Bearer 0xDEADBEEF")
      );
      assert_eq!(stream.pop().unwrap(), Ok(Packet::done("response")));
      let response: HttpResponse = stream.pop().unwrap().unwrap().decode().unwrap();
      assert_eq!(response.version, HttpVersion::Http11);

      Ok(())
    }

    #[test_logger::test(tokio::test)]
    async fn test_post_request() -> Result<()> {
      let (app_config, component_config) = get_config();
      let comp = get_component(&app_config, component_config);
      let packets = packet_stream!(("input", "SENTINEL"), ("number", [123, 345, 678]));
      let invocation = Invocation::test("test_post_request", Entity::local(POST_OP), packets, Default::default())?;
      let stream = comp
        .handle(invocation, Default::default(), panic_callback())
        .await?
        .filter(|p| futures::future::ready(p.as_ref().map_or(false, |p| p.has_data())))
        .collect::<Vec<_>>()
        .await;

      let packets = stream.into_iter().collect::<Result<Vec<_>, _>>()?;
      for packet in packets {
        if packet.port() == "body" {
          println!("{:?}", packet);
          let response = packet.decode_value().unwrap();
          println!("as json: {:?}", response);

          let args = response.get("args").unwrap();
          assert_eq!(args, &json!( {"query1": "SENTINEL"}));
          let data = response.get("json").unwrap();
          assert_eq!(
            data,
            &json!( {"key": "SENTINEL","other":[{"value":123},{"value":345},{"value":678}]})
          );
          let response_headers = response.get("headers").unwrap();
          assert_eq!(
            response_headers.get("Content-Type").unwrap(),
            &json!("application/json")
          );
          assert_eq!(response_headers.get("X-Custom-Header").unwrap(), &json!("SENTINEL"));
        } else {
          let response: HttpResponse = packet.decode().unwrap();
          assert_eq!(response.version, HttpVersion::Http11);
        }
      }

      Ok(())
    }

    #[test_logger::test(tokio::test)]
    async fn test_text_post_request() -> Result<()> {
      let (app_config, component_config) = get_config();
      let comp = get_component(&app_config, component_config);
      let packets = packet_stream!(("input", "SENTINEL"), ("payload", "<xml>FOOBAR</xml>"));
      let invocation = Invocation::test(
        "test_text_post_request",
        Entity::local(POST_OP_TEXT),
        packets,
        Default::default(),
      )?;
      let stream = comp
        .handle(invocation, Default::default(), panic_callback())
        .await?
        .filter(|p| futures::future::ready(p.as_ref().map_or(false, |p| p.has_data())))
        .collect::<Vec<_>>()
        .await;

      let packets = stream.into_iter().collect::<Result<Vec<_>, _>>()?;
      for packet in packets {
        if packet.port() == "body" {
          println!("{:?}", packet);
          let response = packet.decode_value().unwrap();
          println!("as json: {:?}", response);

          let args = response.get("args").unwrap();
          assert_eq!(args, &json!( {"query1": "SENTINEL"}));
          let data = response.get("data").unwrap().to_string();
          assert_eq!(data, "<xml>FOOBAR</xml>");
          let response_headers = response.get("headers").unwrap();
          assert_eq!(
            response_headers.get("Content-Type").unwrap(),
            &json!("application/json")
          );
          assert_eq!(response_headers.get("X-Custom-Header").unwrap(), &json!("SENTINEL"));
        } else {
          let response: HttpResponse = packet.decode().unwrap();
          assert_eq!(response.version, HttpVersion::Http11);
        }
      }

      Ok(())
    }

    #[test_logger::test(tokio::test)]
    async fn test_event_stream() -> Result<()> {
      let (app_config, component_config) = get_config();
      let comp = get_component(&app_config, component_config);

      let event_stream = "data: {\"id\":\"1\",\"object\":\"event1\"}\n\n\
         data: {\"id\":\"2\",\"object\":\"event2\"}\n\n";
      let packets = packet_stream!(("input", event_stream));

      let invocation = Invocation::test(
        "test_event_stream",
        Entity::local("event_stream_op"),
        packets,
        Default::default(),
      )?;
      let stream = comp
        .handle(invocation, Default::default(), panic_callback())
        .await?
        .filter(|p| futures::future::ready(p.as_ref().map_or(false, |p| p.has_data())))
        .collect::<Vec<_>>()
        .await;

      let packets = stream.into_iter().collect::<Result<Vec<_>, _>>()?;
      for packet in packets {
        if packet.port() == "body" {
          let response: HttpEvent = packet.decode().unwrap();
          let response_id = response.get_id().as_ref().unwrap();
          let response_event = response.get_event().as_ref().unwrap();
          assert!(response_id == "1" && response_event == "event1" || response_id == "2" && response_event == "event2");
        } else {
          let response: HttpResponse = packet.decode().unwrap();
          assert_eq!(response.version, HttpVersion::Http11);
        }
      }

      Ok(())
    }
  }
}
