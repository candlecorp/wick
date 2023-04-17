#[allow(unused)]
use guest::*;
use wasmrs_guest as guest;
#[allow(unused)]
pub(crate) type WickStream<T> = BoxFlux<T, wick_component::anyhow::Error>;
pub use wick_component::anyhow::Result;
#[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
pub enum HttpMethod {
  Get,
  Post,
  Put,
  Delete,
  Patch,
  Head,
  Options,
  Trace,
}
impl HttpMethod {
  #[allow(unused)]
  pub fn value(&self) -> Option<&'static str> {
    match self {
      Self::Get => None,
      Self::Post => None,
      Self::Put => None,
      Self::Delete => None,
      Self::Patch => None,
      Self::Head => None,
      Self::Options => None,
      Self::Trace => None,
    }
  }
}
impl std::str::FromStr for HttpMethod {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "GET" => Ok(Self::Get),
      "POST" => Ok(Self::Post),
      "PUT" => Ok(Self::Put),
      "DELETE" => Ok(Self::Delete),
      "PATCH" => Ok(Self::Patch),
      "HEAD" => Ok(Self::Head),
      "OPTIONS" => Ok(Self::Options),
      "TRACE" => Ok(Self::Trace),
      _ => Err(s.to_owned()),
    }
  }
}
impl std::fmt::Display for HttpMethod {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Get => f.write_str("GET"),
      Self::Post => f.write_str("POST"),
      Self::Put => f.write_str("PUT"),
      Self::Delete => f.write_str("DELETE"),
      Self::Patch => f.write_str("PATCH"),
      Self::Head => f.write_str("HEAD"),
      Self::Options => f.write_str("OPTIONS"),
      Self::Trace => f.write_str("TRACE"),
    }
  }
}
#[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
pub enum HttpScheme {
  Http,
  Https,
}
impl HttpScheme {
  #[allow(unused)]
  pub fn value(&self) -> Option<&'static str> {
    match self {
      Self::Http => None,
      Self::Https => None,
    }
  }
}
impl std::str::FromStr for HttpScheme {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "HTTP" => Ok(Self::Http),
      "HTTPS" => Ok(Self::Https),
      _ => Err(s.to_owned()),
    }
  }
}
impl std::fmt::Display for HttpScheme {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Http => f.write_str("HTTP"),
      Self::Https => f.write_str("HTTPS"),
    }
  }
}
#[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
pub enum HttpVersion {
  Http10,
  Http11,
  Http20,
}
impl HttpVersion {
  #[allow(unused)]
  pub fn value(&self) -> Option<&'static str> {
    match self {
      Self::Http10 => Some("1.0"),
      Self::Http11 => Some("1.1"),
      Self::Http20 => Some("2.0"),
    }
  }
}
impl std::str::FromStr for HttpVersion {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "HTTP_1_0" => Ok(Self::Http10),
      "HTTP_1_1" => Ok(Self::Http11),
      "HTTP_2_0" => Ok(Self::Http20),
      _ => Err(s.to_owned()),
    }
  }
}
impl std::fmt::Display for HttpVersion {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Http10 => f.write_str("HTTP_1_0"),
      Self::Http11 => f.write_str("HTTP_1_1"),
      Self::Http20 => f.write_str("HTTP_2_0"),
    }
  }
}
#[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
pub enum StatusCode {
  Continue,
  SwitchingProtocols,
  Ok,
  Created,
  Accepted,
  NonAuthoritativeInformation,
  NoContent,
  ResetContent,
  PartialContent,
  MultipleChoices,
  MovedPermanently,
  Found,
  SeeOther,
  NotModified,
  TemporaryRedirect,
  PermanentRedirect,
  BadRequest,
  Unauthorized,
  PaymentRequired,
  Forbidden,
  NotFound,
  MethodNotAllowed,
  NotAcceptable,
  ProxyAuthenticationRequired,
  RequestTimeout,
  Conflict,
  Gone,
  LengthRequired,
  PreconditionFailed,
  PayloadTooLarge,
  UriTooLong,
  UnsupportedMediaType,
  RangeNotSatisfiable,
  ExpectationFailed,
  ImATeapot,
  UnprocessableEntity,
  Locked,
  FailedDependency,
  TooManyRequests,
  InternalServerError,
  NotImplemented,
  BadGateway,
  ServiceUnavailable,
  GatewayTimeout,
  HttpVersionNotSupported,
}
impl StatusCode {
  #[allow(unused)]
  pub fn value(&self) -> Option<&'static str> {
    match self {
      Self::Continue => Some("100"),
      Self::SwitchingProtocols => Some("101"),
      Self::Ok => Some("200"),
      Self::Created => Some("201"),
      Self::Accepted => Some("202"),
      Self::NonAuthoritativeInformation => Some("203"),
      Self::NoContent => Some("204"),
      Self::ResetContent => Some("205"),
      Self::PartialContent => Some("206"),
      Self::MultipleChoices => Some("300"),
      Self::MovedPermanently => Some("301"),
      Self::Found => Some("302"),
      Self::SeeOther => Some("303"),
      Self::NotModified => Some("304"),
      Self::TemporaryRedirect => Some("307"),
      Self::PermanentRedirect => Some("308"),
      Self::BadRequest => Some("400"),
      Self::Unauthorized => Some("401"),
      Self::PaymentRequired => Some("402"),
      Self::Forbidden => Some("403"),
      Self::NotFound => Some("404"),
      Self::MethodNotAllowed => Some("405"),
      Self::NotAcceptable => Some("406"),
      Self::ProxyAuthenticationRequired => Some("407"),
      Self::RequestTimeout => Some("408"),
      Self::Conflict => Some("409"),
      Self::Gone => Some("410"),
      Self::LengthRequired => Some("411"),
      Self::PreconditionFailed => Some("412"),
      Self::PayloadTooLarge => Some("413"),
      Self::UriTooLong => Some("414"),
      Self::UnsupportedMediaType => Some("415"),
      Self::RangeNotSatisfiable => Some("416"),
      Self::ExpectationFailed => Some("417"),
      Self::ImATeapot => Some("418"),
      Self::UnprocessableEntity => Some("422"),
      Self::Locked => Some("423"),
      Self::FailedDependency => Some("424"),
      Self::TooManyRequests => Some("429"),
      Self::InternalServerError => Some("500"),
      Self::NotImplemented => Some("501"),
      Self::BadGateway => Some("502"),
      Self::ServiceUnavailable => Some("503"),
      Self::GatewayTimeout => Some("504"),
      Self::HttpVersionNotSupported => Some("505"),
    }
  }
}
impl std::str::FromStr for StatusCode {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Continue" => Ok(Self::Continue),
      "SwitchingProtocols" => Ok(Self::SwitchingProtocols),
      "OK" => Ok(Self::Ok),
      "Created" => Ok(Self::Created),
      "Accepted" => Ok(Self::Accepted),
      "NonAuthoritativeInformation" => Ok(Self::NonAuthoritativeInformation),
      "NoContent" => Ok(Self::NoContent),
      "ResetContent" => Ok(Self::ResetContent),
      "PartialContent" => Ok(Self::PartialContent),
      "MultipleChoices" => Ok(Self::MultipleChoices),
      "MovedPermanently" => Ok(Self::MovedPermanently),
      "Found" => Ok(Self::Found),
      "SeeOther" => Ok(Self::SeeOther),
      "NotModified" => Ok(Self::NotModified),
      "TemporaryRedirect" => Ok(Self::TemporaryRedirect),
      "PermanentRedirect" => Ok(Self::PermanentRedirect),
      "BadRequest" => Ok(Self::BadRequest),
      "Unauthorized" => Ok(Self::Unauthorized),
      "PaymentRequired" => Ok(Self::PaymentRequired),
      "Forbidden" => Ok(Self::Forbidden),
      "NotFound" => Ok(Self::NotFound),
      "MethodNotAllowed" => Ok(Self::MethodNotAllowed),
      "NotAcceptable" => Ok(Self::NotAcceptable),
      "ProxyAuthenticationRequired" => Ok(Self::ProxyAuthenticationRequired),
      "RequestTimeout" => Ok(Self::RequestTimeout),
      "Conflict" => Ok(Self::Conflict),
      "Gone" => Ok(Self::Gone),
      "LengthRequired" => Ok(Self::LengthRequired),
      "PreconditionFailed" => Ok(Self::PreconditionFailed),
      "PayloadTooLarge" => Ok(Self::PayloadTooLarge),
      "URITooLong" => Ok(Self::UriTooLong),
      "UnsupportedMediaType" => Ok(Self::UnsupportedMediaType),
      "RangeNotSatisfiable" => Ok(Self::RangeNotSatisfiable),
      "ExpectationFailed" => Ok(Self::ExpectationFailed),
      "ImATeapot" => Ok(Self::ImATeapot),
      "UnprocessableEntity" => Ok(Self::UnprocessableEntity),
      "Locked" => Ok(Self::Locked),
      "FailedDependency" => Ok(Self::FailedDependency),
      "TooManyRequests" => Ok(Self::TooManyRequests),
      "InternalServerError" => Ok(Self::InternalServerError),
      "NotImplemented" => Ok(Self::NotImplemented),
      "BadGateway" => Ok(Self::BadGateway),
      "ServiceUnavailable" => Ok(Self::ServiceUnavailable),
      "GatewayTimeout" => Ok(Self::GatewayTimeout),
      "HTTPVersionNotSupported" => Ok(Self::HttpVersionNotSupported),
      _ => Err(s.to_owned()),
    }
  }
}
impl std::fmt::Display for StatusCode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Continue => f.write_str("Continue"),
      Self::SwitchingProtocols => f.write_str("SwitchingProtocols"),
      Self::Ok => f.write_str("OK"),
      Self::Created => f.write_str("Created"),
      Self::Accepted => f.write_str("Accepted"),
      Self::NonAuthoritativeInformation => f.write_str("NonAuthoritativeInformation"),
      Self::NoContent => f.write_str("NoContent"),
      Self::ResetContent => f.write_str("ResetContent"),
      Self::PartialContent => f.write_str("PartialContent"),
      Self::MultipleChoices => f.write_str("MultipleChoices"),
      Self::MovedPermanently => f.write_str("MovedPermanently"),
      Self::Found => f.write_str("Found"),
      Self::SeeOther => f.write_str("SeeOther"),
      Self::NotModified => f.write_str("NotModified"),
      Self::TemporaryRedirect => f.write_str("TemporaryRedirect"),
      Self::PermanentRedirect => f.write_str("PermanentRedirect"),
      Self::BadRequest => f.write_str("BadRequest"),
      Self::Unauthorized => f.write_str("Unauthorized"),
      Self::PaymentRequired => f.write_str("PaymentRequired"),
      Self::Forbidden => f.write_str("Forbidden"),
      Self::NotFound => f.write_str("NotFound"),
      Self::MethodNotAllowed => f.write_str("MethodNotAllowed"),
      Self::NotAcceptable => f.write_str("NotAcceptable"),
      Self::ProxyAuthenticationRequired => f.write_str("ProxyAuthenticationRequired"),
      Self::RequestTimeout => f.write_str("RequestTimeout"),
      Self::Conflict => f.write_str("Conflict"),
      Self::Gone => f.write_str("Gone"),
      Self::LengthRequired => f.write_str("LengthRequired"),
      Self::PreconditionFailed => f.write_str("PreconditionFailed"),
      Self::PayloadTooLarge => f.write_str("PayloadTooLarge"),
      Self::UriTooLong => f.write_str("URITooLong"),
      Self::UnsupportedMediaType => f.write_str("UnsupportedMediaType"),
      Self::RangeNotSatisfiable => f.write_str("RangeNotSatisfiable"),
      Self::ExpectationFailed => f.write_str("ExpectationFailed"),
      Self::ImATeapot => f.write_str("ImATeapot"),
      Self::UnprocessableEntity => f.write_str("UnprocessableEntity"),
      Self::Locked => f.write_str("Locked"),
      Self::FailedDependency => f.write_str("FailedDependency"),
      Self::TooManyRequests => f.write_str("TooManyRequests"),
      Self::InternalServerError => f.write_str("InternalServerError"),
      Self::NotImplemented => f.write_str("NotImplemented"),
      Self::BadGateway => f.write_str("BadGateway"),
      Self::ServiceUnavailable => f.write_str("ServiceUnavailable"),
      Self::GatewayTimeout => f.write_str("GatewayTimeout"),
      Self::HttpVersionNotSupported => f.write_str("HTTPVersionNotSupported"),
    }
  }
}
#[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
pub struct HttpResponse {
  pub version: HttpVersion,
  pub status: StatusCode,
  pub headers: std::collections::HashMap<String, Vec<String>>,
}
#[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
pub struct HttpRequest {
  pub method: HttpMethod,
  pub scheme: HttpScheme,
  pub authority: String,
  pub query_parameters: std::collections::HashMap<String, Vec<String>>,
  pub path: String,
  pub uri: String,
  pub version: HttpVersion,
  pub headers: std::collections::HashMap<String, Vec<String>>,
}
#[derive(Default, Clone)]
pub struct Component;
impl Component {}
