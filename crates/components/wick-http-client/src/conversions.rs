use std::collections::HashMap;
use std::str::FromStr;

use futures::Stream;
use reqwest::header::HeaderMap;
use reqwest::Version;
use wick_interface_http::types::{HttpResponse, HttpVersion, StatusCode};

use crate::Error;

pub(crate) fn to_wick_response(
  res: reqwest::Response,
) -> Result<(HttpResponse, impl Stream<Item = Result<bytes::Bytes, reqwest::Error>>), Error> {
  let ours = HttpResponse {
    version: match res.version() {
      Version::HTTP_09 => unimplemented!("HTTP/0.9 is not supported"),
      Version::HTTP_10 => HttpVersion::Http10,
      Version::HTTP_11 => HttpVersion::Http11,
      Version::HTTP_2 => HttpVersion::Http20,
      x => unimplemented!("{:?} is not supported", x),
    },
    status: StatusCode::from_str(res.status().as_str()).unwrap_or(StatusCode::Unknown),
    headers: convert_headers(res.headers())?,
  };
  Ok((ours, res.bytes_stream()))
}

fn convert_headers(from_headers: &HeaderMap) -> Result<HashMap<String, Vec<String>>, Error> {
  let mut to_headers = HashMap::new();
  for (key, value) in from_headers {
    let key = key.as_str().to_owned();
    let value = value.to_str().map_err(|_| Error::InvalidHeader(key.clone()))?;
    to_headers.entry(key).or_insert_with(Vec::new).push(value.to_owned());
  }
  Ok(to_headers)
}
