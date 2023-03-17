use std::collections::HashMap;

use hyper::http::response::Builder;
use hyper::{Body, Request, StatusCode};
use wick_interface_http as wick_http;

use super::HttpError;

pub(super) fn convert_method(method: &hyper::Method) -> Result<wick_http::HttpMethod, HttpError> {
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

pub(super) fn convert_scheme(scheme: Option<&hyper::http::uri::Scheme>) -> Result<wick_http::HttpScheme, HttpError> {
  scheme.map_or(Ok(wick_http::HttpScheme::Http), |scheme| {
    if scheme == &hyper::http::uri::Scheme::HTTP {
      Ok(wick_http::HttpScheme::Http)
    } else {
      Ok(wick_http::HttpScheme::Https)
    }
  })
}

pub(super) fn convert_authority(authority: Option<&hyper::http::uri::Authority>) -> Result<String, HttpError> {
  Ok(authority.map_or_else(String::default, |v| v.to_string()))
}

pub(super) fn convert_query_parameters(query: Option<&str>) -> Result<HashMap<String, Vec<String>>, HttpError> {
  let query = url::form_urlencoded::parse(query.unwrap_or_default().as_bytes())
    .into_owned()
    .collect::<Vec<(String, String)>>();
  let mut map = HashMap::new();
  for (key, value) in query {
    map.entry(key).or_insert_with(Vec::new).push(value);
  }
  Ok(map)
}

pub(super) fn convert_path(path: &str) -> Result<String, HttpError> {
  Ok(path.to_owned())
}

pub(super) fn convert_uri(url: &hyper::http::uri::Uri) -> Result<String, HttpError> {
  Ok(url.to_string())
}

pub(super) fn convert_version(version: hyper::http::Version) -> Result<wick_http::HttpVersion, HttpError> {
  match version {
    hyper::http::Version::HTTP_09 => Err(HttpError::UnsupportedVersion("HTTP/0.9".to_owned())),
    hyper::http::Version::HTTP_10 => Ok(wick_http::HttpVersion::Http10),
    hyper::http::Version::HTTP_11 => Ok(wick_http::HttpVersion::Http11),
    hyper::http::Version::HTTP_2 => Ok(wick_http::HttpVersion::Http20),
    _ => Err(HttpError::UnsupportedVersion("Future version".to_owned())),
  }
}

pub(super) fn convert_headers(headers: &hyper::http::HeaderMap) -> Result<HashMap<String, Vec<String>>, HttpError> {
  let mut map = HashMap::new();
  for (key, value) in headers {
    let key = key.as_str().to_owned();
    let value = value.to_str().unwrap().to_owned();
    map.entry(key).or_insert_with(Vec::new).push(value);
  }
  Ok(map)
}

pub(super) fn convert_request(req: Request<Body>) -> Result<(wick_http::HttpRequest, Body), HttpError> {
  Ok((
    wick_http::HttpRequest {
      method: convert_method(req.method())?,
      scheme: convert_scheme(req.uri().scheme())?,
      authority: convert_authority(req.uri().authority())?,
      query_parameters: convert_query_parameters(req.uri().query())?,
      path: convert_path(req.uri().path())?,
      uri: convert_uri(req.uri())?,
      version: convert_version(req.version())?,
      headers: convert_headers(req.headers())?,
    },
    req.into_body(),
  ))
}

pub(super) fn convert_status(code: wick_http::StatusCode) -> Result<StatusCode, HttpError> {
  StatusCode::from_bytes(code.value().unwrap().as_bytes()).map_err(|_e| HttpError::InvalidStatusCode(code.to_string()))
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
