use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;

use hyper::http::response::Builder;
use hyper::http::{HeaderName, HeaderValue};
use hyper::{Body, Request, Response, StatusCode, Uri};
use wick_interface_http::types as wick_http;

use super::HttpError;

pub(super) fn method_to_wick(method: &hyper::Method) -> Result<wick_http::HttpMethod, HttpError> {
  match method {
    &hyper::Method::GET => Ok(wick_http::HttpMethod::Get),
    &hyper::Method::POST => Ok(wick_http::HttpMethod::Post),
    &hyper::Method::PUT => Ok(wick_http::HttpMethod::Put),
    &hyper::Method::DELETE => Ok(wick_http::HttpMethod::Delete),
    &hyper::Method::HEAD => Ok(wick_http::HttpMethod::Head),
    &hyper::Method::OPTIONS => Ok(wick_http::HttpMethod::Options),
    &hyper::Method::TRACE => Ok(wick_http::HttpMethod::Trace),
    x => Err(HttpError::UnsupportedMethod(x.to_string())),
  }
}

pub(super) fn method_from_wick(method: &wick_http::HttpMethod) -> Result<hyper::Method, HttpError> {
  match method {
    &wick_http::HttpMethod::Get => Ok(hyper::Method::GET),
    &wick_http::HttpMethod::Post => Ok(hyper::Method::POST),
    &wick_http::HttpMethod::Put => Ok(hyper::Method::PUT),
    &wick_http::HttpMethod::Delete => Ok(hyper::Method::DELETE),
    &wick_http::HttpMethod::Head => Ok(hyper::Method::HEAD),
    &wick_http::HttpMethod::Options => Ok(hyper::Method::OPTIONS),
    &wick_http::HttpMethod::Trace => Ok(hyper::Method::TRACE),

    x => Err(HttpError::UnsupportedMethod(x.to_string())),
  }
}

pub(super) fn scheme_to_wick(scheme: Option<&hyper::http::uri::Scheme>) -> Result<wick_http::HttpScheme, HttpError> {
  scheme.map_or(Ok(wick_http::HttpScheme::Http), |scheme| {
    if scheme == &hyper::http::uri::Scheme::HTTP {
      Ok(wick_http::HttpScheme::Http)
    } else {
      Ok(wick_http::HttpScheme::Https)
    }
  })
}

pub(super) fn authority_to_wick(authority: Option<&hyper::http::uri::Authority>) -> Result<String, HttpError> {
  Ok(authority.map_or_else(String::default, |v| v.to_string()))
}

pub(super) fn query_params_to_wick(query: Option<&str>) -> Result<HashMap<String, Vec<String>>, HttpError> {
  let query = url::form_urlencoded::parse(query.unwrap_or_default().as_bytes())
    .into_owned()
    .collect::<Vec<(String, String)>>();
  let mut map = HashMap::new();
  for (key, value) in query {
    map.entry(key).or_insert_with(Vec::new).push(value);
  }
  Ok(map)
}

pub(super) fn path_to_wick(path: &str) -> Result<String, HttpError> {
  Ok(path.to_owned())
}

pub(super) fn uri_to_wick(url: &Uri) -> Result<String, HttpError> {
  Ok(url.to_string())
}

pub(super) fn version_to_wick(version: hyper::http::Version) -> Result<wick_http::HttpVersion, HttpError> {
  match version {
    hyper::http::Version::HTTP_09 => Err(HttpError::UnsupportedVersion("HTTP/0.9".to_owned())),
    hyper::http::Version::HTTP_10 => Ok(wick_http::HttpVersion::Http10),
    hyper::http::Version::HTTP_11 => Ok(wick_http::HttpVersion::Http11),
    hyper::http::Version::HTTP_2 => Ok(wick_http::HttpVersion::Http20),
    _ => Err(HttpError::UnsupportedVersion("Future version".to_owned())),
  }
}

pub(super) fn headers_to_wick(headers: &hyper::http::HeaderMap) -> Result<HashMap<String, Vec<String>>, HttpError> {
  let mut map = HashMap::new();
  for (key, value) in headers {
    let key = key.as_str().to_owned();
    let value = value.to_str().unwrap().to_owned();
    map.entry(key).or_insert_with(Vec::new).push(value);
  }
  Ok(map)
}

pub(super) fn request_and_body_to_wick<B>(
  req: Request<B>,
  remote_addr: SocketAddr,
) -> Result<(wick_http::HttpRequest, B), HttpError>
where
  B: Send + Sync + 'static,
{
  Ok((
    wick_http::HttpRequest {
      method: method_to_wick(req.method())?,
      scheme: scheme_to_wick(req.uri().scheme())?,
      authority: authority_to_wick(req.uri().authority())?,
      query_parameters: query_params_to_wick(req.uri().query())?,
      path: path_to_wick(req.uri().path())?,
      uri: uri_to_wick(req.uri())?,
      version: version_to_wick(req.version())?,
      headers: headers_to_wick(req.headers())?,
      remote_addr: remote_addr.to_string(),
    },
    req.into_body(),
  ))
}

pub(super) fn request_to_wick<B>(req: &Request<B>, remote_addr: SocketAddr) -> Result<wick_http::HttpRequest, HttpError>
where
  B: Send + Sync + 'static,
{
  Ok(wick_http::HttpRequest {
    method: method_to_wick(req.method())?,
    scheme: scheme_to_wick(req.uri().scheme())?,
    authority: authority_to_wick(req.uri().authority())?,
    query_parameters: query_params_to_wick(req.uri().query())?,
    path: path_to_wick(req.uri().path())?,
    uri: uri_to_wick(req.uri())?,
    version: version_to_wick(req.version())?,
    headers: headers_to_wick(req.headers())?,
    remote_addr: remote_addr.to_string(),
  })
}

pub(super) fn convert_status(code: wick_http::StatusCode) -> Result<StatusCode, HttpError> {
  StatusCode::from_bytes(code.value().unwrap().as_bytes()).map_err(|_e| HttpError::InvalidStatusCode(code.to_string()))
}

pub(super) fn convert_to_wick_status(code: StatusCode) -> Result<wick_http::StatusCode, HttpError> {
  wick_http::StatusCode::from_str(code.as_str()).map_err(|_e| HttpError::InvalidStatusCode(code.as_str().to_owned()))
}

pub(super) fn convert_response(mut builder: Builder, res: wick_http::HttpResponse) -> Result<Builder, HttpError> {
  builder = builder.status(convert_status(res.status)?);
  for header in res.headers {
    for value in header.1 {
      builder = builder.header(header.0.clone(), value);
    }
  }
  Ok(builder)
}

pub(super) fn merge_requests<B>(wick: &wick_http::HttpRequest, mut hyper: Request<B>) -> Result<Request<B>, HttpError>
where
  B: Send + Sync + 'static,
{
  let headers = hyper.headers_mut();
  headers.clear();
  for (name, values) in &wick.headers {
    if let Some(v) = values.get(0) {
      headers.insert(
        name
          .parse::<HeaderName>()
          .map_err(|_| HttpError::InvalidHeaderName(name.clone()))?,
        HeaderValue::from_str(v).map_err(|_| HttpError::InvalidHeaderValue(v.clone()))?,
      );
    }
  }
  *(hyper.method_mut()) = method_from_wick(&wick.method)?;
  let query_string = wick
    .query_parameters
    .iter()
    .flat_map(|(k, v)| v.iter().map(|v| format!("{}={}", k, v)).collect::<Vec<_>>())
    .collect::<Vec<String>>()
    .join("&");
  *(hyper.uri_mut()) = Uri::builder()
    .path_and_query(format!("{}?{}", wick.path, query_string).as_str())
    .build()
    .map_err(|e| HttpError::InvalidUri(e.to_string()))?;
  Ok(hyper)
}

pub(super) fn convert_to_wick_response(res: Response<Body>) -> Result<(wick_http::HttpResponse, Body), HttpError> {
  let (parts, body) = res.into_parts();

  Ok((
    wick_http::HttpResponse {
      version: version_to_wick(parts.version)?,
      status: convert_to_wick_status(parts.status)?,
      headers: headers_to_wick(&parts.headers)?,
    },
    body,
  ))
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;

  #[test]
  fn test_queryparams() -> Result<()> {
    let uri = Uri::from_static("http://localhost:8080/oidc/callback?code=0.AX0AW62NddvKnU2xw9eh4XLjBRG6sJD_e-FBg1_2xnhMOqcBAAA.AgABAAIAAAD--DLA3VO7QrddgJg7WevrAgDs_wUA9P_a9_u2vYzM5uuK5PDlBJvfDxCepuj-VRXZ-nF62AgxQz2Irlv47oXyyAIlDav9OOsvP_W_GGa3K0CezRZYxPKR5z4yWx_4q_c3GHC4bLGILYCJc2d7lZiyznKTOcHE33E6TlcFULxgmCYnHHblQmBNqKvCDXYqCbcxBT5jy_umsXVPoL4u1BywYnMW6joj7wGCr3JVknlN-TNya_qmnnTKxj7DPsMISmrKoQzz_PWVTGF3jf6qAccGrCiJkgb_F2A5FigkY2BIzwLUHxRAjMyFgik9JErYlTYc6IjPTH-PLSkqctWwSNHAfI4DShDqFcMKmGU1S4ASQ1hgMv3_keRaMffuhMklhYCbB2Xb-p1pQ5D44uPRryIKKYI0PfAqHmE95qIT91SyuD1GeP0n4AZDJtGR7XJ3YTjVX9rJCayi6i4LGv3PfV-WMJuKS4L6YHKuHJMK-rQBB0QZI4Rig1XaQBAU0YXg5zmuR5LjSsFvoiM-MJHZ6dEHg2P21ErGXDEDOi8MwFWRDI2NZorRv9LwM3ixYPeQo2gWW1Fz6K2JSsJdocrGnBmz2XAViUbeNgamkTsvkdxbQnjoaipw0cCCfYc-qHpgLnSXRv9RTqOIiC4l8Tt4YQUNXA&state=3dd445c8-0256-1dfd-e427-8d336ac2974f&session_state=6c5616bb-b5e8-4271-8003-8e66c84755d4#");

    let params = query_params_to_wick(uri.query())?;
    assert!(params.contains_key("code"));
    Ok(())
  }
}
