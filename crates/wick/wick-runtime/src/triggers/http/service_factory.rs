use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::Future;
use hyper::http::response::Builder;
use hyper::server::conn::AddrStream;
use hyper::service::Service;
use hyper::{Body, Request, Response, StatusCode};
use tracing::{Id, Span};
use uuid::Uuid;
use wick_interface_http::types::RequestMiddlewareResponse;
use wick_packet::Invocation;

use super::component_utils::{handle_request_middleware, handle_response_middleware};
use super::conversions::{convert_response, convert_to_wick_response, merge_requests, request_to_wick};
use super::{HttpError, HttpRouter, RawRouterHandler};
use crate::Runtime;

pub(super) struct ServiceFactory {
  engine: Arc<Runtime>,
  routers: Arc<Vec<HttpRouter>>,
  span: Option<Id>,
}

impl ServiceFactory {
  pub(super) fn new(engine: Runtime, routers: Vec<HttpRouter>, span: Option<Id>) -> Self {
    Self {
      engine: Arc::new(engine),
      routers: Arc::new(routers),
      span,
    }
  }
}

impl Service<&AddrStream> for ServiceFactory {
  type Response = ResponseService;
  type Error = hyper::Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync>>;

  fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, conn: &AddrStream) -> Self::Future {
    let engine = self.engine.clone();
    let routers = self.routers.clone();

    let remote_addr = conn.remote_addr();
    let span = self.span.clone();

    let fut = async move { Ok(ResponseService::new(remote_addr, engine, routers, span)) };
    Box::pin(fut)
  }
}

pub(super) struct ResponseService {
  remote_addr: SocketAddr,
  engine: Arc<Runtime>,
  routers: Arc<Vec<HttpRouter>>,
  span: Option<Id>,
}

impl ResponseService {
  fn new(remote_addr: SocketAddr, engine: Arc<Runtime>, routers: Arc<Vec<HttpRouter>>, span: Option<Id>) -> Self {
    Self {
      remote_addr,
      engine,
      routers,
      span,
    }
  }
}

impl Service<Request<Body>> for ResponseService {
  type Response = Response<Body>;
  type Error = HttpError;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

  fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, req: Request<Body>) -> Self::Future {
    let remote_addr = self.remote_addr;
    let time = chrono::Local::now().format("%d/%b/%Y:%H:%M:%S %z");
    let path = req.uri().path().to_owned();
    let tx_id = Invocation::new_tx_id();
    let span = info_span!("http:request",%tx_id);
    span.follows_from(self.span.clone());

    span.in_scope(|| {
      info!(
        time = %time,
        method = %req.method(),
        path = req.uri().path(),
        version = ?req.version(),
      );
    });
    let engine = self.engine.clone();
    let router = self
      .routers
      .iter()
      .find(|r| req.uri().path().starts_with(r.path()))
      .cloned();

    Box::pin(async move {
      let start = chrono::Local::now().format("%d/%b/%Y:%H:%M:%S %z");
      let response = match router {
        Some(h) => match h {
          HttpRouter::Raw(r) => match handle(tx_id, req, r, engine.clone(), remote_addr, &span).await {
            Ok(v) => v,
            Err(e) => {
              span.in_scope(|| {
                error!(
                  time=%start,
                  path,
                  error=%e,
                  "internal error",
                );
              });
              make_ise(None)
            }
          },
        },
        None => Builder::new()
          .status(StatusCode::NOT_FOUND)
          .body(Body::default())
          .unwrap(),
      };
      let status: u16 = response.status().into();

      if status >= 400 {
        span.in_scope(|| {
          error!(
            time=%start,
            path,
            status=%response.status(),
            "error",
          );
        });
      };

      Ok(response)
    })
  }
}

async fn handle(
  tx_id: Uuid,
  req: Request<Body>,
  r: RawRouterHandler,
  engine: Arc<Runtime>,
  remote_addr: SocketAddr,
  span: &Span,
) -> Result<Response<Body>, HttpError> {
  let pre_span = info_span!(parent: span, "pre-request", service.name="pre-request");
  let (wick_request_object, early_response) =
    run_request_middleware(tx_id, &req, engine.clone(), &r, remote_addr, &pre_span).await?;
  // if we have an early response, skip the main handler.
  let response = if let Some(response) = early_response {
    response
  } else {
    let req = merge_requests(&wick_request_object, req)?;
    let main_span = info_span!(parent: span, "request");
    r.component
      .handle(tx_id, remote_addr, engine.clone(), req, &main_span)
      .await?
  };
  let post_span = info_span!(parent: span, "post-request");
  run_response_middleware(tx_id, wick_request_object, response, engine.clone(), &r, &post_span).await
}

async fn run_request_middleware<B>(
  tx_id: Uuid,
  req: &Request<B>,
  engine: Arc<Runtime>,
  r: &RawRouterHandler,
  remote_addr: SocketAddr,
  span: &Span,
) -> Result<(wick_interface_http::types::HttpRequest, Option<Response<Body>>), HttpError>
where
  B: Send + Sync + 'static,
{
  let mut wick_req = request_to_wick(req, remote_addr)?;
  for (entity, config) in &r.middleware.request {
    let response =
      handle_request_middleware(tx_id, entity.clone(), config.clone(), engine.clone(), &wick_req, span).await?;
    match response {
      Some(RequestMiddlewareResponse::HttpRequest(req)) => wick_req = req,
      Some(RequestMiddlewareResponse::HttpResponse(res)) => {
        let builder = convert_response(Response::builder(), res)?;
        let res = builder.body(Body::empty()).unwrap();
        return Ok((wick_req, Some(res)));
      }
      None => {
        // do nothing
      }
    }
  }
  Ok((wick_req, None))
}

async fn run_response_middleware(
  tx_id: Uuid,
  wick_req: wick_interface_http::types::HttpRequest,
  response: Response<Body>,
  engine: Arc<Runtime>,
  r: &RawRouterHandler,
  span: &Span,
) -> Result<Response<Body>, HttpError> {
  let (mut response, body) = convert_to_wick_response(response)?;
  for (entity, config) in &r.middleware.response {
    let modified_response = handle_response_middleware(
      tx_id,
      entity.clone(),
      config.clone(),
      engine.clone(),
      &wick_req,
      &response,
      span,
    )
    .await?;
    if let Some(r) = modified_response {
      response = r;
    }
  }

  let response = convert_response(Response::builder(), response)?
    .body(body)
    .map_err(|_e| HttpError::InternalError(super::InternalError::Builder))?;

  Ok(response)
}

fn make_ise(e: Option<String>) -> Response<Body> {
  Builder::new()
    .status(StatusCode::INTERNAL_SERVER_ERROR)
    .body(Body::from(e.map_or_else(
      || "Internal Server Error. Check log for details".to_owned(),
      |msg| format!("{}. Check log for details", msg),
    )))
    .unwrap()
}
