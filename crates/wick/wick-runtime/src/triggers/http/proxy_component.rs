use std::net::SocketAddr;

use futures::future::BoxFuture;
use hyper::{Body, Request, Response, StatusCode};
use wick_config::config::UrlResource;

use super::{HttpError, RawRouter};

#[derive()]
#[must_use]
pub(super) struct ProxyComponent {
  url: String,
  strip: Option<String>,
}

impl ProxyComponent {
  pub(super) fn new(url: UrlResource, strip: Option<String>) -> Self {
    let url = url.to_string();
    let url = url.trim_end_matches('/').to_owned();
    debug!(url = %url, "wick:http:proxy: proxying");
    Self { url, strip }
  }
}

impl RawRouter for ProxyComponent {
  fn handle(
    &self,
    remote_addr: SocketAddr,
    mut request: Request<Body>,
  ) -> BoxFuture<Result<Response<Body>, HttpError>> {
    let url = self.url.clone();
    let client_ip = remote_addr.ip();
    if let Some(to_strip) = &self.strip {
      let orig_path = request.uri().path_and_query().unwrap().as_str().to_owned();
      let path = orig_path.trim_start_matches(to_strip);
      *request.uri_mut() = path.parse().unwrap();
      trace!(to= url, orig = orig_path, uri = %request.uri(), "http:trigger:proxy proxying");
    } else {
      trace!(to= url, uri = %request.uri(), "http:trigger:proxy proxying");
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
