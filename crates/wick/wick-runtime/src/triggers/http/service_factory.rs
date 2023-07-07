use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::Future;
use hyper::http::response::Builder;
use hyper::server::conn::AddrStream;
use hyper::service::Service;
use hyper::{Body, Request, Response, StatusCode};
use tracing::Span;
use wick_interface_http::types::RequestMiddlewareResponse;

use super::component_utils::{handle_request_middleware, handle_response_middleware};
use super::conversions::{convert_response, convert_to_wick_response, merge_requests, request_to_wick};
use super::{HttpError, HttpRouter, RawRouterHandler};
use crate::Runtime;

pub(super) struct ServiceFactory {
  engine: Arc<Runtime>,
  routers: Arc<Vec<HttpRouter>>,
  span: Span,
}

impl ServiceFactory {
  pub(super) fn new(engine: Runtime, routers: Vec<HttpRouter>, span: Span) -> Self {
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
    let span = debug_span!("connection", remote = %remote_addr);
    span.follows_from(&self.span);

    let fut = async move { Ok(ResponseService::new(remote_addr, engine, routers, span)) };
    Box::pin(fut)
  }
}

pub(super) struct ResponseService {
  remote_addr: SocketAddr,
  engine: Arc<Runtime>,
  routers: Arc<Vec<HttpRouter>>,
  span: Span,
}

impl ResponseService {
  fn new(remote_addr: SocketAddr, engine: Arc<Runtime>, routers: Arc<Vec<HttpRouter>>, span: Span) -> Self {
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
    let span = info_span!("request");
    span.follows_from(&self.span);

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
          HttpRouter::Raw(r) => match handle(req, r, engine.clone(), remote_addr).await {
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
  req: Request<Body>,
  r: RawRouterHandler,
  engine: Arc<Runtime>,
  remote_addr: SocketAddr,
) -> Result<Response<Body>, HttpError> {
  let (wick_request_object, early_response) = run_request_middleware(&req, engine.clone(), &r, remote_addr).await?;
  // if we have an early response, skip the main handler.
  let response = if let Some(response) = early_response {
    response
  } else {
    let req = merge_requests(&wick_request_object, req)?;
    r.component.handle(remote_addr, engine.clone(), req).await?
  };
  run_response_middleware(wick_request_object, response, engine.clone(), &r).await
}

async fn run_request_middleware<B>(
  req: &Request<B>,
  engine: Arc<Runtime>,
  r: &RawRouterHandler,
  remote_addr: SocketAddr,
) -> Result<(wick_interface_http::types::HttpRequest, Option<Response<Body>>), HttpError>
where
  B: Send + Sync + 'static,
{
  let mut wick_req = request_to_wick(req, remote_addr)?;
  for (entity, config) in &r.middleware.request {
    let response = handle_request_middleware(entity.clone(), config.clone(), engine.clone(), &wick_req).await?;
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
  wick_req: wick_interface_http::types::HttpRequest,
  response: Response<Body>,
  engine: Arc<Runtime>,
  r: &RawRouterHandler,
) -> Result<Response<Body>, HttpError> {
  let (mut response, body) = convert_to_wick_response(response)?;
  for (entity, config) in &r.middleware.response {
    let modified_response =
      handle_response_middleware(entity.clone(), config.clone(), engine.clone(), &wick_req, &response).await?;
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
