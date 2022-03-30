use std::sync::Arc;
use std::task::{Context, Poll};

use http::header::{ACCESS_CONTROL_REQUEST_HEADERS, ORIGIN};
use http::{HeaderMap, HeaderValue, Method, Request, Response, StatusCode, Uri};
use http_body::Body;
use tokio_stream::StreamExt;
use tonic::body::{empty_body, BoxBody};
use vino_entity::Entity;
use vino_rpc::SharedRpcHandler;
use vino_transport::{Invocation, TransportMap, TransportWrapper};
use vino_types::HostedType;

use crate::cors::Cors;
use crate::error::HttpError;
use crate::{BoxFuture, Config};

const PREFIX: &str = "rpc";

#[derive(Clone)]
#[must_use]
#[allow(missing_debug_implementations)]
pub struct ProviderService {
  provider: SharedRpcHandler,
  cors: Arc<Cors>,
}

impl ProviderService {
  pub fn new(provider: SharedRpcHandler, config: Config) -> Self {
    Self {
      provider,
      cors: Arc::new(Cors::new(config)),
    }
  }
}

impl tower_service::Service<Request<hyper::Body>> for ProviderService {
  type Response = Response<BoxBody>;
  type Error = HttpError;
  type Future = BoxFuture<Self::Response, Self::Error>;

  fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, mut req: Request<hyper::Body>) -> Self::Future {
    trace!("HTTP:REQUEST:{}", req.uri());
    let provider = self.provider.clone();
    let cors = self.cors.clone();
    let fut = async move {
      let kind = RequestKind::new(&req);
      trace!("HTTP:REQUEST:KIND:{:?}", kind);
      match kind {
        RequestKind::Invocation { target } => {
          let headers = match cors.simple(req.headers()) {
            Ok(headers) => headers,
            Err(e) => {
              warn!("HTTP:INVOKE:FORBIDDEN:{}", e);
              return Ok(response(StatusCode::FORBIDDEN));
            }
          };
          trace!("HTTP:INVOKE");
          let bytes = match req.data().await.unwrap_or_else(|| Ok(vec![].into())) {
            Ok(bytes) => bytes,
            Err(e) => {
              warn!("HTTP:INVOKE:ERROR:{}", e);
              return Err(HttpError::InvocationFailed(e.to_string()));
            }
          };

          let payload: TransportMap = if bytes.is_empty() {
            TransportMap::new()
          } else {
            match TransportMap::from_json_str(&String::from_utf8_lossy(&bytes)) {
              Ok(payload) => payload,
              Err(e) => {
                warn!("HTTP:INVOKE:ERROR:{}", e);
                return Err(HttpError::InvocationFailed(e.to_string()));
              }
            }
          };
          let invocation = Invocation::new(Entity::system("http", ""), target, payload, None);
          match provider.invoke(invocation).await {
            Ok(stream) => {
              trace!("HTTP:INVOKE:OK");

              let packets: Vec<_> = stream.collect().await;
              let response = ProviderServiceResponse::from(packets);
              let mut response: Response<BoxBody> = response.into();
              response.headers_mut().extend(headers);

              Ok(response)
            }
            Err(e) => {
              warn!("HTTP:INVOKE:ERROR:{}", e);
              Err(HttpError::InvocationFailed(e.to_string()))
            }
          }
        }
        RequestKind::List => {
          trace!("HTTP:LIST");
          let headers = match cors.simple(req.headers()) {
            Ok(headers) => headers,
            Err(e) => {
              warn!("HTTP:INVOKE:FORBIDDEN:{}", e);
              return Ok(response(StatusCode::FORBIDDEN));
            }
          };
          match provider.get_list() {
            Ok(sig) => {
              trace!("HTTP:LIST:OK");
              let response = ProviderServiceResponse::from(sig);
              let mut response: Response<BoxBody> = response.into();
              response.headers_mut().extend(headers);
              Ok(response)
            }
            Err(e) => {
              warn!("HTTP:LIST:ERROR:{}", e);

              Err(HttpError::InvocationFailed(e.to_string()))
            }
          }
        }
        RequestKind::Preflight {
          origin,
          request_headers,
        } => {
          trace!("HTTP:PREFLIGHT:{}", origin.to_str().unwrap_or_default());
          match cors.preflight(req.headers(), origin, request_headers) {
            Ok(headers) => {
              trace!("HTTP:PREFLIGHT:OK:{:?}", headers);
              Ok(no_content(headers))
            }
            Err(e) => {
              warn!("HTTP:PREFLIGHT:ERROR:{}", e);
              Ok(response(StatusCode::FORBIDDEN))
            }
          }
        }
        RequestKind::Other(uri) => {
          warn!("HTTP:OTHER[{}]", uri);
          Ok(response(StatusCode::FORBIDDEN))
        }
      }
    };

    Box::pin(fut)
  }
}

fn no_content(headers: HeaderMap) -> Response<BoxBody> {
  let res = Response::builder().status(StatusCode::NO_CONTENT).body(empty_body());
  let mut res = res.unwrap();

  res.headers_mut().extend(headers);

  res
}

fn response(status: StatusCode) -> Response<BoxBody> {
  let res = Response::builder().status(status).body(empty_body());
  res.unwrap()
}

#[derive(Debug, PartialEq)]
pub enum RequestKind<'a> {
  Invocation {
    target: Entity,
  },
  List,
  Preflight {
    origin: &'a HeaderValue,
    request_headers: &'a HeaderValue,
  },
  Other(&'a Uri),
}

impl<'a> RequestKind<'a> {
  fn new(req: &'a Request<hyper::Body>) -> Self {
    let method = req.method();
    let headers = req.headers();

    if let (&Method::OPTIONS, Some(origin), Some(value)) =
      (method, headers.get(ORIGIN), headers.get(ACCESS_CONTROL_REQUEST_HEADERS))
    {
      return RequestKind::Preflight {
        origin,
        request_headers: value,
      };
    }
    if let Some(kind) = has_rpc_call(req) {
      return kind;
    }
    RequestKind::Other(req.uri())
  }
}

pub fn has_rpc_call(req: &Request<hyper::Body>) -> Option<RequestKind> {
  // let has_json_body = matches!(content_type(req.headers()), Some("application/json"));
  trace!("uri: {}", req.uri().path());
  let mut parts = req.uri().path().split('/').peekable();

  loop {
    if parts.peek() == Some(&"") {
      parts.next();
    } else {
      break;
    }
  }

  let prefix = parts.next();
  if prefix.is_none() || prefix != Some(PREFIX) {
    error!("Request for {:?} failed for invalid path.", prefix);
    return None;
  }

  parts.next().and_then(|command| {
    debug!("HTTP:COMMAND[{}]", command);
    match command {
      "invoke" => parts.next().map(|component| {
        let entity = Entity::local_component(component);
        RequestKind::Invocation { target: entity }
      }),
      "list" => Some(RequestKind::List),
      _ => None,
    }
  })
}

fn _content_type(headers: &HeaderMap) -> Option<&str> {
  headers
    .get(http::header::CONTENT_TYPE)
    .and_then(|val| val.to_str().ok())
}

impl tonic::transport::NamedService for ProviderService {
  const NAME: &'static str = PREFIX;
}

#[derive(Debug)]
#[must_use]
pub struct ProviderServiceResponse {
  status: ResponseStatus,
  body: Vec<u8>,
}

#[derive(Debug, Copy, Clone)]
pub enum ResponseStatus {
  Ok,
  Error,
}

impl ResponseStatus {
  #[must_use]
  pub fn to_http_code(&self) -> StatusCode {
    match self {
      ResponseStatus::Ok => http::StatusCode::OK,
      ResponseStatus::Error => http::StatusCode::INTERNAL_SERVER_ERROR,
    }
  }
}

impl ProviderServiceResponse {
  pub fn new(body: Vec<u8>) -> Self {
    Self {
      body,
      status: ResponseStatus::Ok,
    }
  }
}

impl From<Vec<TransportWrapper>> for ProviderServiceResponse {
  fn from(res: Vec<TransportWrapper>) -> Self {
    let simplified: Vec<_> = res.into_iter().map(|p| p.into_json()).collect();
    let json = serde_json::to_string(&simplified).unwrap_or_default();
    ProviderServiceResponse::new(json.as_bytes().to_vec())
  }
}

impl From<Vec<HostedType>> for ProviderServiceResponse {
  fn from(res: Vec<HostedType>) -> Self {
    let json = serde_json::to_string(&res).unwrap();
    ProviderServiceResponse::new(json.as_bytes().to_vec())
  }
}

impl From<ProviderServiceResponse> for Response<BoxBody> {
  fn from(res: ProviderServiceResponse) -> Self {
    let body = hyper::Body::from(res.body)
      .map_err(|e| tonic::Status::failed_precondition(e.to_string()))
      .boxed_unsync();
    http::Response::builder()
      .status(res.status.to_http_code())
      .body(body)
      .unwrap()
  }
}
