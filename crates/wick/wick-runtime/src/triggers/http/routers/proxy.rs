use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::future::BoxFuture;
use hyper::{Body, Request, Response, StatusCode};
use tracing::Span;
use url::Url;
use uuid::Uuid;
use wick_config::config::{AppConfiguration, ImportBinding, ProxyRouterConfig, TriggerKind, WickRouter};

use super::super::{HttpError, HttpRouter, RawRouter};
use crate::dev::prelude::{RuntimeError, *};
use crate::resources::{Resource, ResourceKind};
use crate::runtime::RuntimeConstraint;
use crate::triggers::http::middleware::resolve_middleware_components;
use crate::triggers::http::{index_to_router_id, RawRouterHandler};
use crate::Runtime;

#[derive()]
#[must_use]
pub(super) struct ProxyRouter {
  url: String,
  strip: Option<String>,
}

impl ProxyRouter {
  pub(super) fn new(url: Url, strip: Option<String>) -> Self {
    let url = url.to_string();
    let url = url.trim_end_matches('/').to_owned();

    Self { strip, url }
  }
}

impl RawRouter for ProxyRouter {
  fn handle(
    &self,
    _tx_id: Uuid,
    remote_addr: SocketAddr,
    _runtime: Arc<Runtime>,
    mut request: Request<Body>,
    span: &Span,
  ) -> BoxFuture<Result<Response<Body>, HttpError>> {
    let span = info_span!(parent: span, "proxy");
    let url = self.url.clone();
    let client_ip = remote_addr.ip();
    if let Some(to_strip) = &self.strip {
      let orig_path = request.uri().path_and_query().unwrap().as_str().to_owned();
      let path = orig_path.trim_start_matches(to_strip);
      *request.uri_mut() = path.parse().unwrap();
      span.in_scope(|| trace!(to= url, orig = orig_path, uri = %request.uri(), "http:trigger:proxy proxying"));
    } else {
      span.in_scope(|| trace!(to= url, uri = %request.uri(), "http:trigger:proxy proxying"));
    }
    // the proxy library does not set the appropriate host header, but if we delete
    // the header, it will get made correctly for us.
    request.headers_mut().remove("host");
    let fut = async move {
      match hyper_reverse_proxy::call(client_ip, &url, request).await {
        Ok(response) => Ok(response),
        Err(_error) => Ok(
          Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap(),
        ),
      }
    };
    Box::pin(fut)
  }
}

pub(crate) fn register_proxy_router(
  index: usize,
  resources: Arc<HashMap<String, Resource>>,
  app_config: &AppConfiguration,
  router_config: &ProxyRouterConfig,
) -> Result<(Vec<ImportBinding>, HttpRouter, Vec<RuntimeConstraint>), RuntimeError> {
  trace!(index, "registering proxy router");
  let (middleware, mut bindings) = resolve_middleware_components(index, app_config, router_config)?;
  let url = resources.get(router_config.url()).ok_or_else(|| {
    RuntimeError::ResourceNotFound(
      TriggerKind::Http.into(),
      format!("url resource {} not found", router_config.url()),
    )
  })?;
  let url = match url {
    Resource::Url(s) => s.clone(),
    _ => {
      return Err(RuntimeError::InvalidResourceType(
        TriggerKind::Http.into(),
        ResourceKind::Url,
        url.kind(),
      ))
    }
  };
  let strip_path = if router_config.strip_path() {
    Some(router_config.path().to_owned())
  } else {
    None
  };
  let router = ProxyRouter::new(url, strip_path);
  let router_component = config::ComponentDefinition::Native(config::components::NativeComponent {});
  let router_binding = config::ImportBinding::component(index_to_router_id(index), router_component);
  bindings.push(router_binding);
  Ok((
    bindings,
    HttpRouter::Raw(RawRouterHandler {
      path: router_config.path().to_owned(),
      component: Arc::new(router),
      middleware,
    }),
    vec![],
  ))
}
