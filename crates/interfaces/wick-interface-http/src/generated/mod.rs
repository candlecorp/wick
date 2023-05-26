pub use wick_component::{packet as wick_packet, wasmrs, wasmrs_codec};
#[allow(unused)]
pub(crate) type WickStream<T> = wick_component::wasmrs_rx::BoxFlux<T, wick_component::anyhow::Error>;
pub use wick_component::anyhow::Result;
pub use wick_component::flow_component::Context;
pub mod types {
  #[allow(unused)]
  use super::types;
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
      #[allow(clippy::match_single_binding)]
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
  impl TryFrom<u32> for HttpMethod {
    type Error = u32;
    fn try_from(i: u32) -> Result<Self, Self::Error> {
      #[allow(clippy::match_single_binding)]
      match i {
        _ => Err(i),
      }
    }
  }
  impl std::str::FromStr for HttpMethod {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
      #[allow(clippy::match_single_binding)]
      match s {
        _ => Err(s.to_owned()),
      }
    }
  }
  impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      #[allow(clippy::match_single_binding)]
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
      #[allow(clippy::match_single_binding)]
      match self {
        Self::Http => None,
        Self::Https => None,
      }
    }
  }
  impl TryFrom<u32> for HttpScheme {
    type Error = u32;
    fn try_from(i: u32) -> Result<Self, Self::Error> {
      #[allow(clippy::match_single_binding)]
      match i {
        _ => Err(i),
      }
    }
  }
  impl std::str::FromStr for HttpScheme {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
      #[allow(clippy::match_single_binding)]
      match s {
        _ => Err(s.to_owned()),
      }
    }
  }
  impl std::fmt::Display for HttpScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      #[allow(clippy::match_single_binding)]
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
      #[allow(clippy::match_single_binding)]
      match self {
        Self::Http10 => Some("1.0"),
        Self::Http11 => Some("1.1"),
        Self::Http20 => Some("2.0"),
      }
    }
  }
  impl TryFrom<u32> for HttpVersion {
    type Error = u32;
    fn try_from(i: u32) -> Result<Self, Self::Error> {
      #[allow(clippy::match_single_binding)]
      match i {
        _ => Err(i),
      }
    }
  }
  impl std::str::FromStr for HttpVersion {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
      #[allow(clippy::match_single_binding)]
      match s {
        "1.0" => Ok(Self::Http10),
        "1.1" => Ok(Self::Http11),
        "2.0" => Ok(Self::Http20),
        _ => Err(s.to_owned()),
      }
    }
  }
  impl std::fmt::Display for HttpVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      #[allow(clippy::match_single_binding)]
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
    Unknown,
  }
  impl StatusCode {
    #[allow(unused)]
    pub fn value(&self) -> Option<&'static str> {
      #[allow(clippy::match_single_binding)]
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
        Self::Unknown => Some("-1"),
      }
    }
  }
  impl TryFrom<u32> for StatusCode {
    type Error = u32;
    fn try_from(i: u32) -> Result<Self, Self::Error> {
      #[allow(clippy::match_single_binding)]
      match i {
        _ => Err(i),
      }
    }
  }
  impl std::str::FromStr for StatusCode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
      #[allow(clippy::match_single_binding)]
      match s {
        "100" => Ok(Self::Continue),
        "101" => Ok(Self::SwitchingProtocols),
        "200" => Ok(Self::Ok),
        "201" => Ok(Self::Created),
        "202" => Ok(Self::Accepted),
        "203" => Ok(Self::NonAuthoritativeInformation),
        "204" => Ok(Self::NoContent),
        "205" => Ok(Self::ResetContent),
        "206" => Ok(Self::PartialContent),
        "300" => Ok(Self::MultipleChoices),
        "301" => Ok(Self::MovedPermanently),
        "302" => Ok(Self::Found),
        "303" => Ok(Self::SeeOther),
        "304" => Ok(Self::NotModified),
        "307" => Ok(Self::TemporaryRedirect),
        "308" => Ok(Self::PermanentRedirect),
        "400" => Ok(Self::BadRequest),
        "401" => Ok(Self::Unauthorized),
        "402" => Ok(Self::PaymentRequired),
        "403" => Ok(Self::Forbidden),
        "404" => Ok(Self::NotFound),
        "405" => Ok(Self::MethodNotAllowed),
        "406" => Ok(Self::NotAcceptable),
        "407" => Ok(Self::ProxyAuthenticationRequired),
        "408" => Ok(Self::RequestTimeout),
        "409" => Ok(Self::Conflict),
        "410" => Ok(Self::Gone),
        "411" => Ok(Self::LengthRequired),
        "412" => Ok(Self::PreconditionFailed),
        "413" => Ok(Self::PayloadTooLarge),
        "414" => Ok(Self::UriTooLong),
        "415" => Ok(Self::UnsupportedMediaType),
        "416" => Ok(Self::RangeNotSatisfiable),
        "417" => Ok(Self::ExpectationFailed),
        "418" => Ok(Self::ImATeapot),
        "422" => Ok(Self::UnprocessableEntity),
        "423" => Ok(Self::Locked),
        "424" => Ok(Self::FailedDependency),
        "429" => Ok(Self::TooManyRequests),
        "500" => Ok(Self::InternalServerError),
        "501" => Ok(Self::NotImplemented),
        "502" => Ok(Self::BadGateway),
        "503" => Ok(Self::ServiceUnavailable),
        "504" => Ok(Self::GatewayTimeout),
        "505" => Ok(Self::HttpVersionNotSupported),
        "-1" => Ok(Self::Unknown),
        _ => Err(s.to_owned()),
      }
    }
  }
  impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      #[allow(clippy::match_single_binding)]
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
        Self::Unknown => f.write_str("Unknown"),
      }
    }
  }
  #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
  pub struct HttpResponse {
    pub version: types::HttpVersion,
    pub status: types::StatusCode,
    pub headers: std::collections::HashMap<String, Vec<String>>,
  }
  #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
  pub struct HttpRequest {
    pub method: types::HttpMethod,
    pub scheme: types::HttpScheme,
    pub authority: String,
    pub query_parameters: std::collections::HashMap<String, Vec<String>>,
    pub path: String,
    pub uri: String,
    pub version: types::HttpVersion,
    pub headers: std::collections::HashMap<String, Vec<String>>,
  }
}
#[derive(Default, Clone)]
pub struct Component;
impl Component {}
