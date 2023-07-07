use anyhow::Result;
use wick_component::{json, Value};
use wick_interface_http::types::{HttpRequest, HttpResponse, RequestMiddlewareResponse};

#[test_logger::test(tokio::test)]
async fn request_repr() -> Result<()> {
  let expected = HttpRequest {
    version: wick_interface_http::types::HttpVersion::Http11,
    headers: [("accept".to_owned(), vec!["*/*".to_owned()])].into(),
    authority: "localhost:8080".to_owned(),
    path: "/redirect".to_owned(),
    remote_addr: "127.0.0.1:53371".to_owned(),
    uri: "/redirect?url=https://google.com".to_owned(),
    method: wick_interface_http::types::HttpMethod::Get,
    query_parameters: [("url".to_owned(), vec!["https://google.com".to_owned()])].into(),
    scheme: wick_interface_http::types::HttpScheme::Http,
  };
  let expected_json = json!({
    "authority":"localhost:8080",
    "headers":{
      "accept":["*/*"],
    },
    "method":"Get",
    "path":"/redirect",
    "query_parameters":{"url":["https://google.com"]},
    "remote_addr":"127.0.0.1:53371",
    "scheme":"Http",
    "uri":"/redirect?url=https://google.com",
    "version":"1.1"
  });

  let expected_as_bytes = wick_component::wasmrs_codec::messagepack::serialize(&expected)?;

  let actual_as_json: Value = wick_component::wasmrs_codec::messagepack::deserialize(&expected_as_bytes)?;
  println!("expected_json: {}", expected_json);
  println!("actual_json: {}", actual_as_json);
  assert_eq!(actual_as_json, expected_json);

  let actual: HttpRequest = wick_component::wasmrs_codec::messagepack::deserialize(&expected_as_bytes)?;
  assert_eq!(actual, expected);

  let expected_json_as_bytes = wick_component::wasmrs_codec::messagepack::serialize(&expected_json)?;
  let actual_json: HttpRequest = wick_component::wasmrs_codec::messagepack::deserialize(&expected_json_as_bytes)?;
  assert_eq!(actual_json, expected);

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn response_repr() -> Result<()> {
  let expected = HttpResponse {
    status: wick_interface_http::types::StatusCode::TemporaryRedirect,
    version: wick_interface_http::types::HttpVersion::Http11,
    headers: [("location".to_owned(), vec!["https://google.com".to_owned()])].into(),
  };
  let expected_json = json!({"headers":{"location":["https://google.com"]},"status":"307","version":"1.1"});

  let expected_as_bytes = wick_component::wasmrs_codec::messagepack::serialize(&expected)?;

  let actual_as_json: Value = wick_component::wasmrs_codec::messagepack::deserialize(&expected_as_bytes)?;
  println!("as_json: {}", actual_as_json);
  assert_eq!(actual_as_json, expected_json);

  let expected_json_as_bytes = wick_component::wasmrs_codec::messagepack::serialize(&expected)?;

  let actual: HttpResponse = wick_component::wasmrs_codec::messagepack::deserialize(&expected_as_bytes)?;
  assert_eq!(actual, expected);

  let actual_json: HttpResponse = wick_component::wasmrs_codec::messagepack::deserialize(&expected_json_as_bytes)?;
  assert_eq!(actual_json, expected);

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn enum_repr() -> Result<()> {
  let expected = HttpResponse {
    status: wick_interface_http::types::StatusCode::TemporaryRedirect,
    version: wick_interface_http::types::HttpVersion::Http11,
    headers: [("location".to_owned(), vec!["https://google.com".to_owned()])].into(),
  };

  let expected_as_bytes =
    wick_component::wasmrs_codec::messagepack::serialize(&RequestMiddlewareResponse::HttpResponse(expected.clone()))?;

  let actual: HttpResponse = wick_component::wasmrs_codec::messagepack::deserialize(&expected_as_bytes)?;
  assert_eq!(actual, expected);

  Ok(())
}
