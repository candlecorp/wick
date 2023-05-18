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

use super::HttpRouter;
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
  type Error = hyper::Error;
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
      match router {
        Some(h) => match h {
          HttpRouter::Raw(r) => match r.component.handle(remote_addr, engine, req).await {
            Ok(res) => {
              let status: u16 = res.status().into();

              if status >= 400 {
                span.in_scope(|| {
                  error!(
                    time=%start,
                    path,
                    status=%res.status(),
                    "error",
                  );
                });
              };

              Ok(res)
            }
            Err(e) => {
              span.in_scope(|| {
                error!(
                  time=%start,
                  path,
                  error=%e,
                  "internal error",
                );
              });

              Ok(make_ise(e))
            }
          },
        },
        None => Ok(make_ise("")),
      }
    })
  }
}

fn make_ise(e: impl std::fmt::Display) -> Response<Body> {
  Builder::new()
    .status(StatusCode::INTERNAL_SERVER_ERROR)
    .body(Body::from(e.to_string()))
    .unwrap()
}
