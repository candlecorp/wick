use std::net::SocketAddr;
use std::path::PathBuf;
use std::task::{Context, Poll};

use futures::future::BoxFuture;
use hyper::service::Service;
use hyper::{Body, Method, Request, Response};
use hyper_staticfile::{resolve_path, ResolveResult, ResponseBuilder};

use super::{HttpError, RawRouter};

static ID: &str = "wick:http:static";

#[derive()]
#[must_use]
pub(super) struct StaticComponent {
  handler: Static,
}

impl StaticComponent {
  pub(super) fn new(root: PathBuf, strip: Option<String>) -> Self {
    debug!(directory = %root.display(), "{}: serving", ID);
    let handler = Static::new(root, strip);
    Self { handler }
  }
}

impl RawRouter for StaticComponent {
  fn handle(&self, _remote_addr: SocketAddr, request: Request<Body>) -> BoxFuture<Result<Response<Body>, HttpError>> {
    let handler = self.handler.clone();
    let fut = async move {
      let response = handler
        .serve(request)
        .await
        .map_err(|e| HttpError::OperationError(e.to_string()))?;
      Ok(response)
    };
    Box::pin(fut)
  }
}

#[derive(Clone)]
struct Static {
  root: PathBuf,
  strip: Option<String>,
}

impl Static {
  fn new(root: impl Into<PathBuf>, strip: Option<String>) -> Self {
    let root = root.into();
    Static { root, strip }
  }

  /// Serve a request.
  async fn serve<B>(self, request: Request<B>) -> Result<Response<Body>, std::io::Error>
  where
    B: Send + Sync + 'static,
  {
    let Self { root, strip } = self;
    // Handle only `GET`/`HEAD` and absolute paths.
    match *request.method() {
      Method::HEAD | Method::GET => {}
      _ => {
        #[allow(clippy::expect_used)]
        return Ok(
          ResponseBuilder::new()
            .request(&request)
            .build(ResolveResult::MethodNotMatched)
            .expect("unable to build response"),
        );
      }
    }

    let path = strip.map_or_else(
      || request.uri().path(),
      |path| request.uri().path().trim_start_matches(&path),
    );

    resolve_path(root, path).await.map(|result| {
      #[allow(clippy::expect_used)]
      ResponseBuilder::new()
        .request(&request)
        .build(result)
        .expect("unable to build response")
    })
  }
}

impl<B: Send + Sync + 'static> Service<Request<B>> for Static {
  type Response = Response<Body>;
  type Error = std::io::Error;
  type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

  fn poll_ready(&mut self, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, request: Request<B>) -> Self::Future {
    Box::pin(self.clone().serve(request))
  }
}
