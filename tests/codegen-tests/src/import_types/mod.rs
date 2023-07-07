pub use async_trait::async_trait;
#[allow(unused)]
pub(crate) use wick_component::prelude::*;
#[allow(unused)]
pub(crate) use wick_component::wasmrs_rx::{Observable, Observer};
pub use wick_component::{datetime, packet as wick_packet, runtime, wasmrs, wasmrs_codec, wasmrs_rx};
#[allow(unused)]
pub(crate) type WickStream<T> = wick_component::wasmrs_rx::BoxFlux<T, Box<dyn std::error::Error + Send + Sync>>;
pub use wick_component::flow_component::Context;
#[no_mangle]
#[cfg(target_family = "wasm")]
extern "C" fn __wasmrs_init(guest_buffer_size: u32, host_buffer_size: u32, max_host_frame_len: u32) {
  wick_component::wasmrs_guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);
  wick_component::wasmrs_guest::register_request_response("wick", "__setup", Box::new(__setup));
  wick_component::wasmrs_guest::register_request_channel("wick", "echo", Box::new(Component::echo_wrapper));
  wick_component::wasmrs_guest::register_request_channel("wick", "testop", Box::new(Component::testop_wrapper));
}
#[cfg(target_family = "wasm")]
thread_local! { static __CONFIG : std :: cell :: UnsafeCell < Option < SetupPayload >> = std :: cell :: UnsafeCell :: new (None) ; }
#[derive(Debug, Clone, Default, serde :: Serialize, serde :: Deserialize, PartialEq)]
pub struct RootConfig {}
#[cfg(target_family = "wasm")]
#[derive(Debug, serde :: Deserialize)]
pub(crate) struct SetupPayload {
  #[allow(unused)]
  pub(crate) provided: std::collections::HashMap<String, wick_packet::ComponentReference>,
  #[allow(unused)]
  pub(crate) config: RootConfig,
}
#[cfg(target_family = "wasm")]
fn __setup(
  input: wasmrs_rx::BoxMono<wasmrs::Payload, wasmrs::PayloadError>,
) -> Result<wasmrs_rx::BoxMono<wasmrs::RawPayload, wasmrs::PayloadError>, wick_component::BoxError> {
  Ok(Box::pin(async move {
    let payload = input.await?;
    match wasmrs_codec::messagepack::deserialize::<SetupPayload>(&payload.data) {
      Ok(input) => {
        __CONFIG.with(|cell| {
          #[allow(unsafe_code)]
          unsafe { &mut *cell.get() }.replace(input);
        });
        Ok(wasmrs::RawPayload::new_data(None, None))
      }
      Err(e) => Err(wasmrs::PayloadError::application_error(e.to_string(), None)),
    }
  }))
}
#[allow(unused)]
#[cfg(target_family = "wasm")]
pub(crate) fn get_config() -> &'static SetupPayload {
  __CONFIG.with(|cell| {
    #[allow(unsafe_code)]
    unsafe { &*cell.get() }.as_ref().unwrap()
  })
}
#[allow(unused)]
#[cfg(target_family = "wasm")]
pub(crate) fn get_root_config() -> &'static RootConfig {
  __CONFIG.with(|cell| {
    #[allow(unsafe_code)]
    &unsafe { &*cell.get() }.as_ref().unwrap().config
  })
}
pub(crate) trait RootConfigContext {
  fn root_config(&self) -> &'static RootConfig;
}
impl<T> RootConfigContext for Context<T>
where
  T: std::fmt::Debug + wick_component::flow_component::LocalAwareSend,
{
  fn root_config(&self) -> &'static RootConfig {
    #[cfg(target_family = "wasm")]
    {
      get_root_config()
    }
    #[cfg(not(target_family = "wasm"))]
    {
      unimplemented!("root_config is only available in wasm builds")
    }
  }
}
#[doc = "Additional generated types"]
pub mod types {
  #[allow(unused)]
  use super::types;
  #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
  #[doc = "a useful struct"]
  pub struct LocalStruct {
    #[serde(rename = "field1")]
    pub field1: String,
    #[serde(rename = "inner")]
    pub inner: types::LocalStructInner,
    #[serde(rename = "time")]
    #[serde(deserialize_with = "wick_component::datetime::serde::from_str_or_integer")]
    pub time: wick_component::datetime::DateTime,
  }
  #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
  pub struct LocalStructInner {
    #[serde(rename = "field1")]
    pub field1: String,
    #[serde(rename = "field2")]
    pub field2: String,
  }
  #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
  #[doc = "a weird union"]
  #[serde(untagged)]
  pub enum LocalUnion {
    #[doc = "A string value."]
    String(String),
    #[doc = "A LocalStructInner value."]
    LocalStructInner(types::LocalStructInner),
    #[doc = "A datetime value."]
    Datetime(wick_component::datetime::DateTime),
  }
  pub mod aaa {
    #[allow(unused)]
    use super::aaa;
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP method enum"]
    #[serde(into = "String", try_from = "wick_component::serde::enum_repr::StringOrNum")]
    pub enum HttpMethod {
      #[doc = "HTTP GET method"]
      Get,
      #[doc = "HTTP POST method"]
      Post,
      #[doc = "HTTP PUT method"]
      Put,
      #[doc = "HTTP DELETE method"]
      Delete,
      #[doc = "HTTP PATCH method"]
      Patch,
      #[doc = "HTTP HEAD method"]
      Head,
      #[doc = "HTTP OPTIONS method"]
      Options,
      #[doc = "HTTP TRACE method"]
      Trace,
    }
    impl TryFrom<wick_component::serde::enum_repr::StringOrNum> for HttpMethod {
      type Error = String;
      fn try_from(value: wick_component::serde::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
        }
      }
    }
    impl From<HttpMethod> for String {
      fn from(value: HttpMethod) -> Self {
        value.value().map_or_else(|| value.to_string(), |v| v.to_owned())
      }
    }
    impl HttpMethod {
      #[allow(unused)]
      #[doc = "Returns the value of the enum variant as a string."]
      #[must_use]
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
          "Get" => Ok(Self::Get),
          "Post" => Ok(Self::Post),
          "Put" => Ok(Self::Put),
          "Delete" => Ok(Self::Delete),
          "Patch" => Ok(Self::Patch),
          "Head" => Ok(Self::Head),
          "Options" => Ok(Self::Options),
          "Trace" => Ok(Self::Trace),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for HttpMethod {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Get => f.write_str("Get"),
          Self::Post => f.write_str("Post"),
          Self::Put => f.write_str("Put"),
          Self::Delete => f.write_str("Delete"),
          Self::Patch => f.write_str("Patch"),
          Self::Head => f.write_str("Head"),
          Self::Options => f.write_str("Options"),
          Self::Trace => f.write_str("Trace"),
        }
      }
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP request"]
    pub struct HttpRequest {
      #[doc = "method from request line enum"]
      #[serde(rename = "method")]
      pub method: HttpMethod,
      #[doc = "scheme from request line enum"]
      #[serde(rename = "scheme")]
      pub scheme: HttpScheme,
      #[doc = "domain/port and any authentication from request line. optional"]
      #[serde(rename = "authority")]
      pub authority: String,
      #[doc = "query parameters from request line. optional"]
      #[serde(rename = "query_parameters")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub query_parameters: std::collections::HashMap<String, Vec<String>>,
      #[doc = "path from request line (not including query parameters)"]
      #[serde(rename = "path")]
      pub path: String,
      #[doc = "full URI from request line"]
      #[serde(rename = "uri")]
      pub uri: String,
      #[doc = "HTTP version enum"]
      #[serde(rename = "version")]
      pub version: HttpVersion,
      #[doc = "All request headers. Duplicates are comma separated"]
      #[serde(rename = "headers")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub headers: std::collections::HashMap<String, Vec<String>>,
      #[doc = "The remote address of the connected client"]
      #[serde(rename = "remote_addr")]
      pub remote_addr: String,
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP response"]
    pub struct HttpResponse {
      #[doc = "HTTP version enum"]
      #[serde(rename = "version")]
      pub version: HttpVersion,
      #[doc = "status code enum"]
      #[serde(rename = "status")]
      pub status: StatusCode,
      #[doc = "All response headers. Supports duplicates."]
      #[serde(rename = "headers")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub headers: std::collections::HashMap<String, Vec<String>>,
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP scheme"]
    #[serde(into = "String", try_from = "wick_component::serde::enum_repr::StringOrNum")]
    pub enum HttpScheme {
      #[doc = "HTTP scheme"]
      Http,
      #[doc = "HTTPS scheme"]
      Https,
    }
    impl TryFrom<wick_component::serde::enum_repr::StringOrNum> for HttpScheme {
      type Error = String;
      fn try_from(value: wick_component::serde::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
        }
      }
    }
    impl From<HttpScheme> for String {
      fn from(value: HttpScheme) -> Self {
        value.value().map_or_else(|| value.to_string(), |v| v.to_owned())
      }
    }
    impl HttpScheme {
      #[allow(unused)]
      #[doc = "Returns the value of the enum variant as a string."]
      #[must_use]
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
          "Http" => Ok(Self::Http),
          "Https" => Ok(Self::Https),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for HttpScheme {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Http => f.write_str("Http"),
          Self::Https => f.write_str("Https"),
        }
      }
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP version"]
    #[serde(into = "String", try_from = "wick_component::serde::enum_repr::StringOrNum")]
    pub enum HttpVersion {
      #[doc = "HTTP 1.0 version"]
      Http10,
      #[doc = "HTTP 1.1 version"]
      Http11,
      #[doc = "HTTP 2.0 version"]
      Http20,
    }
    impl TryFrom<wick_component::serde::enum_repr::StringOrNum> for HttpVersion {
      type Error = String;
      fn try_from(value: wick_component::serde::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
        }
      }
    }
    impl From<HttpVersion> for String {
      fn from(value: HttpVersion) -> Self {
        value.value().map_or_else(|| value.to_string(), |v| v.to_owned())
      }
    }
    impl HttpVersion {
      #[allow(unused)]
      #[doc = "Returns the value of the enum variant as a string."]
      #[must_use]
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
          "Http10" => Ok(Self::Http10),
          "1.1" => Ok(Self::Http11),
          "Http11" => Ok(Self::Http11),
          "2.0" => Ok(Self::Http20),
          "Http20" => Ok(Self::Http20),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for HttpVersion {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Http10 => f.write_str("1.0"),
          Self::Http11 => f.write_str("1.1"),
          Self::Http20 => f.write_str("2.0"),
        }
      }
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "A response from pre-request middleware"]
    #[serde(untagged)]
    pub enum RequestMiddlewareResponse {
      #[doc = "A HttpRequest value."]
      HttpRequest(HttpRequest),
      #[doc = "A HttpResponse value."]
      HttpResponse(HttpResponse),
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP status code"]
    #[serde(into = "String", try_from = "wick_component::serde::enum_repr::StringOrNum")]
    pub enum StatusCode {
      #[doc = "Continue status code"]
      Continue,
      #[doc = "SwitchingProtocols status code"]
      SwitchingProtocols,
      #[doc = "HTTP OK status code"]
      Ok,
      #[doc = "Created status code"]
      Created,
      #[doc = "Accepted status code"]
      Accepted,
      #[doc = "NonAuthoritativeInformation status code"]
      NonAuthoritativeInformation,
      #[doc = "NoContent status code"]
      NoContent,
      #[doc = "ResetContent status code"]
      ResetContent,
      #[doc = "PartialContent status code"]
      PartialContent,
      #[doc = "MultipleChoices status code"]
      MultipleChoices,
      #[doc = "MovedPermanently status code"]
      MovedPermanently,
      #[doc = "Found status code"]
      Found,
      #[doc = "SeeOther status code"]
      SeeOther,
      #[doc = "NotModified status code"]
      NotModified,
      #[doc = "TemporaryRedirect status code"]
      TemporaryRedirect,
      #[doc = "PermanentRedirect status code"]
      PermanentRedirect,
      #[doc = "BadRequest status code"]
      BadRequest,
      #[doc = "Unauthorized status code"]
      Unauthorized,
      #[doc = "PaymentRequired status code"]
      PaymentRequired,
      #[doc = "Forbidden status code"]
      Forbidden,
      #[doc = "NotFound status code"]
      NotFound,
      #[doc = "MethodNotAllowed status code"]
      MethodNotAllowed,
      #[doc = "NotAcceptable status code"]
      NotAcceptable,
      #[doc = "ProxyAuthenticationRequired status code"]
      ProxyAuthenticationRequired,
      #[doc = "RequestTimeout status code"]
      RequestTimeout,
      #[doc = "Conflict status code"]
      Conflict,
      #[doc = "Gone status code"]
      Gone,
      #[doc = "LengthRequired status code"]
      LengthRequired,
      #[doc = "PreconditionFailed status code"]
      PreconditionFailed,
      #[doc = "PayloadTooLarge status code"]
      PayloadTooLarge,
      #[doc = "URITooLong status code"]
      UriTooLong,
      #[doc = "UnsupportedMediaType status code"]
      UnsupportedMediaType,
      #[doc = "RangeNotSatisfiable status code"]
      RangeNotSatisfiable,
      #[doc = "ExpectationFailed status code"]
      ExpectationFailed,
      #[doc = "ImATeapot status code"]
      ImATeapot,
      #[doc = "UnprocessableEntity status code"]
      UnprocessableEntity,
      #[doc = "Locked status code"]
      Locked,
      #[doc = "FailedDependency status code"]
      FailedDependency,
      #[doc = "TooManyRequests status code"]
      TooManyRequests,
      #[doc = "InternalServerError status code"]
      InternalServerError,
      #[doc = "NotImplemented status code"]
      NotImplemented,
      #[doc = "BadGateway status code"]
      BadGateway,
      #[doc = "ServiceUnavailable status code"]
      ServiceUnavailable,
      #[doc = "GatewayTimeout status code"]
      GatewayTimeout,
      #[doc = "HTTPVersionNotSupported status code"]
      HttpVersionNotSupported,
      #[doc = "Indicates an unknown status code"]
      Unknown,
    }
    impl TryFrom<wick_component::serde::enum_repr::StringOrNum> for StatusCode {
      type Error = String;
      fn try_from(value: wick_component::serde::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
        }
      }
    }
    impl From<StatusCode> for String {
      fn from(value: StatusCode) -> Self {
        value.value().map_or_else(|| value.to_string(), |v| v.to_owned())
      }
    }
    impl StatusCode {
      #[allow(unused)]
      #[doc = "Returns the value of the enum variant as a string."]
      #[must_use]
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
          "Continue" => Ok(Self::Continue),
          "101" => Ok(Self::SwitchingProtocols),
          "SwitchingProtocols" => Ok(Self::SwitchingProtocols),
          "200" => Ok(Self::Ok),
          "Ok" => Ok(Self::Ok),
          "201" => Ok(Self::Created),
          "Created" => Ok(Self::Created),
          "202" => Ok(Self::Accepted),
          "Accepted" => Ok(Self::Accepted),
          "203" => Ok(Self::NonAuthoritativeInformation),
          "NonAuthoritativeInformation" => Ok(Self::NonAuthoritativeInformation),
          "204" => Ok(Self::NoContent),
          "NoContent" => Ok(Self::NoContent),
          "205" => Ok(Self::ResetContent),
          "ResetContent" => Ok(Self::ResetContent),
          "206" => Ok(Self::PartialContent),
          "PartialContent" => Ok(Self::PartialContent),
          "300" => Ok(Self::MultipleChoices),
          "MultipleChoices" => Ok(Self::MultipleChoices),
          "301" => Ok(Self::MovedPermanently),
          "MovedPermanently" => Ok(Self::MovedPermanently),
          "302" => Ok(Self::Found),
          "Found" => Ok(Self::Found),
          "303" => Ok(Self::SeeOther),
          "SeeOther" => Ok(Self::SeeOther),
          "304" => Ok(Self::NotModified),
          "NotModified" => Ok(Self::NotModified),
          "307" => Ok(Self::TemporaryRedirect),
          "TemporaryRedirect" => Ok(Self::TemporaryRedirect),
          "308" => Ok(Self::PermanentRedirect),
          "PermanentRedirect" => Ok(Self::PermanentRedirect),
          "400" => Ok(Self::BadRequest),
          "BadRequest" => Ok(Self::BadRequest),
          "401" => Ok(Self::Unauthorized),
          "Unauthorized" => Ok(Self::Unauthorized),
          "402" => Ok(Self::PaymentRequired),
          "PaymentRequired" => Ok(Self::PaymentRequired),
          "403" => Ok(Self::Forbidden),
          "Forbidden" => Ok(Self::Forbidden),
          "404" => Ok(Self::NotFound),
          "NotFound" => Ok(Self::NotFound),
          "405" => Ok(Self::MethodNotAllowed),
          "MethodNotAllowed" => Ok(Self::MethodNotAllowed),
          "406" => Ok(Self::NotAcceptable),
          "NotAcceptable" => Ok(Self::NotAcceptable),
          "407" => Ok(Self::ProxyAuthenticationRequired),
          "ProxyAuthenticationRequired" => Ok(Self::ProxyAuthenticationRequired),
          "408" => Ok(Self::RequestTimeout),
          "RequestTimeout" => Ok(Self::RequestTimeout),
          "409" => Ok(Self::Conflict),
          "Conflict" => Ok(Self::Conflict),
          "410" => Ok(Self::Gone),
          "Gone" => Ok(Self::Gone),
          "411" => Ok(Self::LengthRequired),
          "LengthRequired" => Ok(Self::LengthRequired),
          "412" => Ok(Self::PreconditionFailed),
          "PreconditionFailed" => Ok(Self::PreconditionFailed),
          "413" => Ok(Self::PayloadTooLarge),
          "PayloadTooLarge" => Ok(Self::PayloadTooLarge),
          "414" => Ok(Self::UriTooLong),
          "UriTooLong" => Ok(Self::UriTooLong),
          "415" => Ok(Self::UnsupportedMediaType),
          "UnsupportedMediaType" => Ok(Self::UnsupportedMediaType),
          "416" => Ok(Self::RangeNotSatisfiable),
          "RangeNotSatisfiable" => Ok(Self::RangeNotSatisfiable),
          "417" => Ok(Self::ExpectationFailed),
          "ExpectationFailed" => Ok(Self::ExpectationFailed),
          "418" => Ok(Self::ImATeapot),
          "ImATeapot" => Ok(Self::ImATeapot),
          "422" => Ok(Self::UnprocessableEntity),
          "UnprocessableEntity" => Ok(Self::UnprocessableEntity),
          "423" => Ok(Self::Locked),
          "Locked" => Ok(Self::Locked),
          "424" => Ok(Self::FailedDependency),
          "FailedDependency" => Ok(Self::FailedDependency),
          "429" => Ok(Self::TooManyRequests),
          "TooManyRequests" => Ok(Self::TooManyRequests),
          "500" => Ok(Self::InternalServerError),
          "InternalServerError" => Ok(Self::InternalServerError),
          "501" => Ok(Self::NotImplemented),
          "NotImplemented" => Ok(Self::NotImplemented),
          "502" => Ok(Self::BadGateway),
          "BadGateway" => Ok(Self::BadGateway),
          "503" => Ok(Self::ServiceUnavailable),
          "ServiceUnavailable" => Ok(Self::ServiceUnavailable),
          "504" => Ok(Self::GatewayTimeout),
          "GatewayTimeout" => Ok(Self::GatewayTimeout),
          "505" => Ok(Self::HttpVersionNotSupported),
          "HttpVersionNotSupported" => Ok(Self::HttpVersionNotSupported),
          "-1" => Ok(Self::Unknown),
          "Unknown" => Ok(Self::Unknown),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for StatusCode {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Continue => f.write_str("100"),
          Self::SwitchingProtocols => f.write_str("101"),
          Self::Ok => f.write_str("200"),
          Self::Created => f.write_str("201"),
          Self::Accepted => f.write_str("202"),
          Self::NonAuthoritativeInformation => f.write_str("203"),
          Self::NoContent => f.write_str("204"),
          Self::ResetContent => f.write_str("205"),
          Self::PartialContent => f.write_str("206"),
          Self::MultipleChoices => f.write_str("300"),
          Self::MovedPermanently => f.write_str("301"),
          Self::Found => f.write_str("302"),
          Self::SeeOther => f.write_str("303"),
          Self::NotModified => f.write_str("304"),
          Self::TemporaryRedirect => f.write_str("307"),
          Self::PermanentRedirect => f.write_str("308"),
          Self::BadRequest => f.write_str("400"),
          Self::Unauthorized => f.write_str("401"),
          Self::PaymentRequired => f.write_str("402"),
          Self::Forbidden => f.write_str("403"),
          Self::NotFound => f.write_str("404"),
          Self::MethodNotAllowed => f.write_str("405"),
          Self::NotAcceptable => f.write_str("406"),
          Self::ProxyAuthenticationRequired => f.write_str("407"),
          Self::RequestTimeout => f.write_str("408"),
          Self::Conflict => f.write_str("409"),
          Self::Gone => f.write_str("410"),
          Self::LengthRequired => f.write_str("411"),
          Self::PreconditionFailed => f.write_str("412"),
          Self::PayloadTooLarge => f.write_str("413"),
          Self::UriTooLong => f.write_str("414"),
          Self::UnsupportedMediaType => f.write_str("415"),
          Self::RangeNotSatisfiable => f.write_str("416"),
          Self::ExpectationFailed => f.write_str("417"),
          Self::ImATeapot => f.write_str("418"),
          Self::UnprocessableEntity => f.write_str("422"),
          Self::Locked => f.write_str("423"),
          Self::FailedDependency => f.write_str("424"),
          Self::TooManyRequests => f.write_str("429"),
          Self::InternalServerError => f.write_str("500"),
          Self::NotImplemented => f.write_str("501"),
          Self::BadGateway => f.write_str("502"),
          Self::ServiceUnavailable => f.write_str("503"),
          Self::GatewayTimeout => f.write_str("504"),
          Self::HttpVersionNotSupported => f.write_str("505"),
          Self::Unknown => f.write_str("-1"),
        }
      }
    }
  }
  pub mod zzz {
    #[allow(unused)]
    use super::zzz;
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP method enum"]
    #[serde(into = "String", try_from = "wick_component::serde::enum_repr::StringOrNum")]
    pub enum HttpMethod {
      #[doc = "HTTP GET method"]
      Get,
      #[doc = "HTTP POST method"]
      Post,
      #[doc = "HTTP PUT method"]
      Put,
      #[doc = "HTTP DELETE method"]
      Delete,
      #[doc = "HTTP PATCH method"]
      Patch,
      #[doc = "HTTP HEAD method"]
      Head,
      #[doc = "HTTP OPTIONS method"]
      Options,
      #[doc = "HTTP TRACE method"]
      Trace,
    }
    impl TryFrom<wick_component::serde::enum_repr::StringOrNum> for HttpMethod {
      type Error = String;
      fn try_from(value: wick_component::serde::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
        }
      }
    }
    impl From<HttpMethod> for String {
      fn from(value: HttpMethod) -> Self {
        value.value().map_or_else(|| value.to_string(), |v| v.to_owned())
      }
    }
    impl HttpMethod {
      #[allow(unused)]
      #[doc = "Returns the value of the enum variant as a string."]
      #[must_use]
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
          "Get" => Ok(Self::Get),
          "Post" => Ok(Self::Post),
          "Put" => Ok(Self::Put),
          "Delete" => Ok(Self::Delete),
          "Patch" => Ok(Self::Patch),
          "Head" => Ok(Self::Head),
          "Options" => Ok(Self::Options),
          "Trace" => Ok(Self::Trace),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for HttpMethod {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Get => f.write_str("Get"),
          Self::Post => f.write_str("Post"),
          Self::Put => f.write_str("Put"),
          Self::Delete => f.write_str("Delete"),
          Self::Patch => f.write_str("Patch"),
          Self::Head => f.write_str("Head"),
          Self::Options => f.write_str("Options"),
          Self::Trace => f.write_str("Trace"),
        }
      }
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP request"]
    pub struct HttpRequest {
      #[doc = "method from request line enum"]
      #[serde(rename = "method")]
      pub method: HttpMethod,
      #[doc = "scheme from request line enum"]
      #[serde(rename = "scheme")]
      pub scheme: HttpScheme,
      #[doc = "domain/port and any authentication from request line. optional"]
      #[serde(rename = "authority")]
      pub authority: String,
      #[doc = "query parameters from request line. optional"]
      #[serde(rename = "query_parameters")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub query_parameters: std::collections::HashMap<String, Vec<String>>,
      #[doc = "path from request line (not including query parameters)"]
      #[serde(rename = "path")]
      pub path: String,
      #[doc = "full URI from request line"]
      #[serde(rename = "uri")]
      pub uri: String,
      #[doc = "HTTP version enum"]
      #[serde(rename = "version")]
      pub version: HttpVersion,
      #[doc = "All request headers. Duplicates are comma separated"]
      #[serde(rename = "headers")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub headers: std::collections::HashMap<String, Vec<String>>,
      #[doc = "The remote address of the connected client"]
      #[serde(rename = "remote_addr")]
      pub remote_addr: String,
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP response"]
    pub struct HttpResponse {
      #[doc = "HTTP version enum"]
      #[serde(rename = "version")]
      pub version: HttpVersion,
      #[doc = "status code enum"]
      #[serde(rename = "status")]
      pub status: StatusCode,
      #[doc = "All response headers. Supports duplicates."]
      #[serde(rename = "headers")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub headers: std::collections::HashMap<String, Vec<String>>,
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP scheme"]
    #[serde(into = "String", try_from = "wick_component::serde::enum_repr::StringOrNum")]
    pub enum HttpScheme {
      #[doc = "HTTP scheme"]
      Http,
      #[doc = "HTTPS scheme"]
      Https,
    }
    impl TryFrom<wick_component::serde::enum_repr::StringOrNum> for HttpScheme {
      type Error = String;
      fn try_from(value: wick_component::serde::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
        }
      }
    }
    impl From<HttpScheme> for String {
      fn from(value: HttpScheme) -> Self {
        value.value().map_or_else(|| value.to_string(), |v| v.to_owned())
      }
    }
    impl HttpScheme {
      #[allow(unused)]
      #[doc = "Returns the value of the enum variant as a string."]
      #[must_use]
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
          "Http" => Ok(Self::Http),
          "Https" => Ok(Self::Https),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for HttpScheme {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Http => f.write_str("Http"),
          Self::Https => f.write_str("Https"),
        }
      }
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP version"]
    #[serde(into = "String", try_from = "wick_component::serde::enum_repr::StringOrNum")]
    pub enum HttpVersion {
      #[doc = "HTTP 1.0 version"]
      Http10,
      #[doc = "HTTP 1.1 version"]
      Http11,
      #[doc = "HTTP 2.0 version"]
      Http20,
    }
    impl TryFrom<wick_component::serde::enum_repr::StringOrNum> for HttpVersion {
      type Error = String;
      fn try_from(value: wick_component::serde::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
        }
      }
    }
    impl From<HttpVersion> for String {
      fn from(value: HttpVersion) -> Self {
        value.value().map_or_else(|| value.to_string(), |v| v.to_owned())
      }
    }
    impl HttpVersion {
      #[allow(unused)]
      #[doc = "Returns the value of the enum variant as a string."]
      #[must_use]
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
          "Http10" => Ok(Self::Http10),
          "1.1" => Ok(Self::Http11),
          "Http11" => Ok(Self::Http11),
          "2.0" => Ok(Self::Http20),
          "Http20" => Ok(Self::Http20),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for HttpVersion {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Http10 => f.write_str("1.0"),
          Self::Http11 => f.write_str("1.1"),
          Self::Http20 => f.write_str("2.0"),
        }
      }
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "A response from pre-request middleware"]
    #[serde(untagged)]
    pub enum RequestMiddlewareResponse {
      #[doc = "A HttpRequest value."]
      HttpRequest(HttpRequest),
      #[doc = "A HttpResponse value."]
      HttpResponse(HttpResponse),
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP status code"]
    #[serde(into = "String", try_from = "wick_component::serde::enum_repr::StringOrNum")]
    pub enum StatusCode {
      #[doc = "Continue status code"]
      Continue,
      #[doc = "SwitchingProtocols status code"]
      SwitchingProtocols,
      #[doc = "HTTP OK status code"]
      Ok,
      #[doc = "Created status code"]
      Created,
      #[doc = "Accepted status code"]
      Accepted,
      #[doc = "NonAuthoritativeInformation status code"]
      NonAuthoritativeInformation,
      #[doc = "NoContent status code"]
      NoContent,
      #[doc = "ResetContent status code"]
      ResetContent,
      #[doc = "PartialContent status code"]
      PartialContent,
      #[doc = "MultipleChoices status code"]
      MultipleChoices,
      #[doc = "MovedPermanently status code"]
      MovedPermanently,
      #[doc = "Found status code"]
      Found,
      #[doc = "SeeOther status code"]
      SeeOther,
      #[doc = "NotModified status code"]
      NotModified,
      #[doc = "TemporaryRedirect status code"]
      TemporaryRedirect,
      #[doc = "PermanentRedirect status code"]
      PermanentRedirect,
      #[doc = "BadRequest status code"]
      BadRequest,
      #[doc = "Unauthorized status code"]
      Unauthorized,
      #[doc = "PaymentRequired status code"]
      PaymentRequired,
      #[doc = "Forbidden status code"]
      Forbidden,
      #[doc = "NotFound status code"]
      NotFound,
      #[doc = "MethodNotAllowed status code"]
      MethodNotAllowed,
      #[doc = "NotAcceptable status code"]
      NotAcceptable,
      #[doc = "ProxyAuthenticationRequired status code"]
      ProxyAuthenticationRequired,
      #[doc = "RequestTimeout status code"]
      RequestTimeout,
      #[doc = "Conflict status code"]
      Conflict,
      #[doc = "Gone status code"]
      Gone,
      #[doc = "LengthRequired status code"]
      LengthRequired,
      #[doc = "PreconditionFailed status code"]
      PreconditionFailed,
      #[doc = "PayloadTooLarge status code"]
      PayloadTooLarge,
      #[doc = "URITooLong status code"]
      UriTooLong,
      #[doc = "UnsupportedMediaType status code"]
      UnsupportedMediaType,
      #[doc = "RangeNotSatisfiable status code"]
      RangeNotSatisfiable,
      #[doc = "ExpectationFailed status code"]
      ExpectationFailed,
      #[doc = "ImATeapot status code"]
      ImATeapot,
      #[doc = "UnprocessableEntity status code"]
      UnprocessableEntity,
      #[doc = "Locked status code"]
      Locked,
      #[doc = "FailedDependency status code"]
      FailedDependency,
      #[doc = "TooManyRequests status code"]
      TooManyRequests,
      #[doc = "InternalServerError status code"]
      InternalServerError,
      #[doc = "NotImplemented status code"]
      NotImplemented,
      #[doc = "BadGateway status code"]
      BadGateway,
      #[doc = "ServiceUnavailable status code"]
      ServiceUnavailable,
      #[doc = "GatewayTimeout status code"]
      GatewayTimeout,
      #[doc = "HTTPVersionNotSupported status code"]
      HttpVersionNotSupported,
      #[doc = "Indicates an unknown status code"]
      Unknown,
    }
    impl TryFrom<wick_component::serde::enum_repr::StringOrNum> for StatusCode {
      type Error = String;
      fn try_from(value: wick_component::serde::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
        }
      }
    }
    impl From<StatusCode> for String {
      fn from(value: StatusCode) -> Self {
        value.value().map_or_else(|| value.to_string(), |v| v.to_owned())
      }
    }
    impl StatusCode {
      #[allow(unused)]
      #[doc = "Returns the value of the enum variant as a string."]
      #[must_use]
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
          "Continue" => Ok(Self::Continue),
          "101" => Ok(Self::SwitchingProtocols),
          "SwitchingProtocols" => Ok(Self::SwitchingProtocols),
          "200" => Ok(Self::Ok),
          "Ok" => Ok(Self::Ok),
          "201" => Ok(Self::Created),
          "Created" => Ok(Self::Created),
          "202" => Ok(Self::Accepted),
          "Accepted" => Ok(Self::Accepted),
          "203" => Ok(Self::NonAuthoritativeInformation),
          "NonAuthoritativeInformation" => Ok(Self::NonAuthoritativeInformation),
          "204" => Ok(Self::NoContent),
          "NoContent" => Ok(Self::NoContent),
          "205" => Ok(Self::ResetContent),
          "ResetContent" => Ok(Self::ResetContent),
          "206" => Ok(Self::PartialContent),
          "PartialContent" => Ok(Self::PartialContent),
          "300" => Ok(Self::MultipleChoices),
          "MultipleChoices" => Ok(Self::MultipleChoices),
          "301" => Ok(Self::MovedPermanently),
          "MovedPermanently" => Ok(Self::MovedPermanently),
          "302" => Ok(Self::Found),
          "Found" => Ok(Self::Found),
          "303" => Ok(Self::SeeOther),
          "SeeOther" => Ok(Self::SeeOther),
          "304" => Ok(Self::NotModified),
          "NotModified" => Ok(Self::NotModified),
          "307" => Ok(Self::TemporaryRedirect),
          "TemporaryRedirect" => Ok(Self::TemporaryRedirect),
          "308" => Ok(Self::PermanentRedirect),
          "PermanentRedirect" => Ok(Self::PermanentRedirect),
          "400" => Ok(Self::BadRequest),
          "BadRequest" => Ok(Self::BadRequest),
          "401" => Ok(Self::Unauthorized),
          "Unauthorized" => Ok(Self::Unauthorized),
          "402" => Ok(Self::PaymentRequired),
          "PaymentRequired" => Ok(Self::PaymentRequired),
          "403" => Ok(Self::Forbidden),
          "Forbidden" => Ok(Self::Forbidden),
          "404" => Ok(Self::NotFound),
          "NotFound" => Ok(Self::NotFound),
          "405" => Ok(Self::MethodNotAllowed),
          "MethodNotAllowed" => Ok(Self::MethodNotAllowed),
          "406" => Ok(Self::NotAcceptable),
          "NotAcceptable" => Ok(Self::NotAcceptable),
          "407" => Ok(Self::ProxyAuthenticationRequired),
          "ProxyAuthenticationRequired" => Ok(Self::ProxyAuthenticationRequired),
          "408" => Ok(Self::RequestTimeout),
          "RequestTimeout" => Ok(Self::RequestTimeout),
          "409" => Ok(Self::Conflict),
          "Conflict" => Ok(Self::Conflict),
          "410" => Ok(Self::Gone),
          "Gone" => Ok(Self::Gone),
          "411" => Ok(Self::LengthRequired),
          "LengthRequired" => Ok(Self::LengthRequired),
          "412" => Ok(Self::PreconditionFailed),
          "PreconditionFailed" => Ok(Self::PreconditionFailed),
          "413" => Ok(Self::PayloadTooLarge),
          "PayloadTooLarge" => Ok(Self::PayloadTooLarge),
          "414" => Ok(Self::UriTooLong),
          "UriTooLong" => Ok(Self::UriTooLong),
          "415" => Ok(Self::UnsupportedMediaType),
          "UnsupportedMediaType" => Ok(Self::UnsupportedMediaType),
          "416" => Ok(Self::RangeNotSatisfiable),
          "RangeNotSatisfiable" => Ok(Self::RangeNotSatisfiable),
          "417" => Ok(Self::ExpectationFailed),
          "ExpectationFailed" => Ok(Self::ExpectationFailed),
          "418" => Ok(Self::ImATeapot),
          "ImATeapot" => Ok(Self::ImATeapot),
          "422" => Ok(Self::UnprocessableEntity),
          "UnprocessableEntity" => Ok(Self::UnprocessableEntity),
          "423" => Ok(Self::Locked),
          "Locked" => Ok(Self::Locked),
          "424" => Ok(Self::FailedDependency),
          "FailedDependency" => Ok(Self::FailedDependency),
          "429" => Ok(Self::TooManyRequests),
          "TooManyRequests" => Ok(Self::TooManyRequests),
          "500" => Ok(Self::InternalServerError),
          "InternalServerError" => Ok(Self::InternalServerError),
          "501" => Ok(Self::NotImplemented),
          "NotImplemented" => Ok(Self::NotImplemented),
          "502" => Ok(Self::BadGateway),
          "BadGateway" => Ok(Self::BadGateway),
          "503" => Ok(Self::ServiceUnavailable),
          "ServiceUnavailable" => Ok(Self::ServiceUnavailable),
          "504" => Ok(Self::GatewayTimeout),
          "GatewayTimeout" => Ok(Self::GatewayTimeout),
          "505" => Ok(Self::HttpVersionNotSupported),
          "HttpVersionNotSupported" => Ok(Self::HttpVersionNotSupported),
          "-1" => Ok(Self::Unknown),
          "Unknown" => Ok(Self::Unknown),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for StatusCode {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Continue => f.write_str("100"),
          Self::SwitchingProtocols => f.write_str("101"),
          Self::Ok => f.write_str("200"),
          Self::Created => f.write_str("201"),
          Self::Accepted => f.write_str("202"),
          Self::NonAuthoritativeInformation => f.write_str("203"),
          Self::NoContent => f.write_str("204"),
          Self::ResetContent => f.write_str("205"),
          Self::PartialContent => f.write_str("206"),
          Self::MultipleChoices => f.write_str("300"),
          Self::MovedPermanently => f.write_str("301"),
          Self::Found => f.write_str("302"),
          Self::SeeOther => f.write_str("303"),
          Self::NotModified => f.write_str("304"),
          Self::TemporaryRedirect => f.write_str("307"),
          Self::PermanentRedirect => f.write_str("308"),
          Self::BadRequest => f.write_str("400"),
          Self::Unauthorized => f.write_str("401"),
          Self::PaymentRequired => f.write_str("402"),
          Self::Forbidden => f.write_str("403"),
          Self::NotFound => f.write_str("404"),
          Self::MethodNotAllowed => f.write_str("405"),
          Self::NotAcceptable => f.write_str("406"),
          Self::ProxyAuthenticationRequired => f.write_str("407"),
          Self::RequestTimeout => f.write_str("408"),
          Self::Conflict => f.write_str("409"),
          Self::Gone => f.write_str("410"),
          Self::LengthRequired => f.write_str("411"),
          Self::PreconditionFailed => f.write_str("412"),
          Self::PayloadTooLarge => f.write_str("413"),
          Self::UriTooLong => f.write_str("414"),
          Self::UnsupportedMediaType => f.write_str("415"),
          Self::RangeNotSatisfiable => f.write_str("416"),
          Self::ExpectationFailed => f.write_str("417"),
          Self::ImATeapot => f.write_str("418"),
          Self::UnprocessableEntity => f.write_str("422"),
          Self::Locked => f.write_str("423"),
          Self::FailedDependency => f.write_str("424"),
          Self::TooManyRequests => f.write_str("429"),
          Self::InternalServerError => f.write_str("500"),
          Self::NotImplemented => f.write_str("501"),
          Self::BadGateway => f.write_str("502"),
          Self::ServiceUnavailable => f.write_str("503"),
          Self::GatewayTimeout => f.write_str("504"),
          Self::HttpVersionNotSupported => f.write_str("505"),
          Self::Unknown => f.write_str("-1"),
        }
      }
    }
  }
  pub mod http {
    #[allow(unused)]
    use super::http;
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP method enum"]
    #[serde(into = "String", try_from = "wick_component::serde::enum_repr::StringOrNum")]
    pub enum HttpMethod {
      #[doc = "HTTP GET method"]
      Get,
      #[doc = "HTTP POST method"]
      Post,
      #[doc = "HTTP PUT method"]
      Put,
      #[doc = "HTTP DELETE method"]
      Delete,
      #[doc = "HTTP PATCH method"]
      Patch,
      #[doc = "HTTP HEAD method"]
      Head,
      #[doc = "HTTP OPTIONS method"]
      Options,
      #[doc = "HTTP TRACE method"]
      Trace,
    }
    impl TryFrom<wick_component::serde::enum_repr::StringOrNum> for HttpMethod {
      type Error = String;
      fn try_from(value: wick_component::serde::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
        }
      }
    }
    impl From<HttpMethod> for String {
      fn from(value: HttpMethod) -> Self {
        value.value().map_or_else(|| value.to_string(), |v| v.to_owned())
      }
    }
    impl HttpMethod {
      #[allow(unused)]
      #[doc = "Returns the value of the enum variant as a string."]
      #[must_use]
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
          "Get" => Ok(Self::Get),
          "Post" => Ok(Self::Post),
          "Put" => Ok(Self::Put),
          "Delete" => Ok(Self::Delete),
          "Patch" => Ok(Self::Patch),
          "Head" => Ok(Self::Head),
          "Options" => Ok(Self::Options),
          "Trace" => Ok(Self::Trace),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for HttpMethod {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Get => f.write_str("Get"),
          Self::Post => f.write_str("Post"),
          Self::Put => f.write_str("Put"),
          Self::Delete => f.write_str("Delete"),
          Self::Patch => f.write_str("Patch"),
          Self::Head => f.write_str("Head"),
          Self::Options => f.write_str("Options"),
          Self::Trace => f.write_str("Trace"),
        }
      }
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP request"]
    pub struct HttpRequest {
      #[doc = "method from request line enum"]
      #[serde(rename = "method")]
      pub method: HttpMethod,
      #[doc = "scheme from request line enum"]
      #[serde(rename = "scheme")]
      pub scheme: HttpScheme,
      #[doc = "domain/port and any authentication from request line. optional"]
      #[serde(rename = "authority")]
      pub authority: String,
      #[doc = "query parameters from request line. optional"]
      #[serde(rename = "query_parameters")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub query_parameters: std::collections::HashMap<String, Vec<String>>,
      #[doc = "path from request line (not including query parameters)"]
      #[serde(rename = "path")]
      pub path: String,
      #[doc = "full URI from request line"]
      #[serde(rename = "uri")]
      pub uri: String,
      #[doc = "HTTP version enum"]
      #[serde(rename = "version")]
      pub version: HttpVersion,
      #[doc = "All request headers. Duplicates are comma separated"]
      #[serde(rename = "headers")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub headers: std::collections::HashMap<String, Vec<String>>,
      #[doc = "The remote address of the connected client"]
      #[serde(rename = "remote_addr")]
      pub remote_addr: String,
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP response"]
    pub struct HttpResponse {
      #[doc = "HTTP version enum"]
      #[serde(rename = "version")]
      pub version: HttpVersion,
      #[doc = "status code enum"]
      #[serde(rename = "status")]
      pub status: StatusCode,
      #[doc = "All response headers. Supports duplicates."]
      #[serde(rename = "headers")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub headers: std::collections::HashMap<String, Vec<String>>,
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP scheme"]
    #[serde(into = "String", try_from = "wick_component::serde::enum_repr::StringOrNum")]
    pub enum HttpScheme {
      #[doc = "HTTP scheme"]
      Http,
      #[doc = "HTTPS scheme"]
      Https,
    }
    impl TryFrom<wick_component::serde::enum_repr::StringOrNum> for HttpScheme {
      type Error = String;
      fn try_from(value: wick_component::serde::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
        }
      }
    }
    impl From<HttpScheme> for String {
      fn from(value: HttpScheme) -> Self {
        value.value().map_or_else(|| value.to_string(), |v| v.to_owned())
      }
    }
    impl HttpScheme {
      #[allow(unused)]
      #[doc = "Returns the value of the enum variant as a string."]
      #[must_use]
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
          "Http" => Ok(Self::Http),
          "Https" => Ok(Self::Https),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for HttpScheme {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Http => f.write_str("Http"),
          Self::Https => f.write_str("Https"),
        }
      }
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP version"]
    #[serde(into = "String", try_from = "wick_component::serde::enum_repr::StringOrNum")]
    pub enum HttpVersion {
      #[doc = "HTTP 1.0 version"]
      Http10,
      #[doc = "HTTP 1.1 version"]
      Http11,
      #[doc = "HTTP 2.0 version"]
      Http20,
    }
    impl TryFrom<wick_component::serde::enum_repr::StringOrNum> for HttpVersion {
      type Error = String;
      fn try_from(value: wick_component::serde::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
        }
      }
    }
    impl From<HttpVersion> for String {
      fn from(value: HttpVersion) -> Self {
        value.value().map_or_else(|| value.to_string(), |v| v.to_owned())
      }
    }
    impl HttpVersion {
      #[allow(unused)]
      #[doc = "Returns the value of the enum variant as a string."]
      #[must_use]
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
          "Http10" => Ok(Self::Http10),
          "1.1" => Ok(Self::Http11),
          "Http11" => Ok(Self::Http11),
          "2.0" => Ok(Self::Http20),
          "Http20" => Ok(Self::Http20),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for HttpVersion {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Http10 => f.write_str("1.0"),
          Self::Http11 => f.write_str("1.1"),
          Self::Http20 => f.write_str("2.0"),
        }
      }
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "A response from pre-request middleware"]
    #[serde(untagged)]
    pub enum RequestMiddlewareResponse {
      #[doc = "A HttpRequest value."]
      HttpRequest(HttpRequest),
      #[doc = "A HttpResponse value."]
      HttpResponse(HttpResponse),
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    #[doc = "HTTP status code"]
    #[serde(into = "String", try_from = "wick_component::serde::enum_repr::StringOrNum")]
    pub enum StatusCode {
      #[doc = "Continue status code"]
      Continue,
      #[doc = "SwitchingProtocols status code"]
      SwitchingProtocols,
      #[doc = "HTTP OK status code"]
      Ok,
      #[doc = "Created status code"]
      Created,
      #[doc = "Accepted status code"]
      Accepted,
      #[doc = "NonAuthoritativeInformation status code"]
      NonAuthoritativeInformation,
      #[doc = "NoContent status code"]
      NoContent,
      #[doc = "ResetContent status code"]
      ResetContent,
      #[doc = "PartialContent status code"]
      PartialContent,
      #[doc = "MultipleChoices status code"]
      MultipleChoices,
      #[doc = "MovedPermanently status code"]
      MovedPermanently,
      #[doc = "Found status code"]
      Found,
      #[doc = "SeeOther status code"]
      SeeOther,
      #[doc = "NotModified status code"]
      NotModified,
      #[doc = "TemporaryRedirect status code"]
      TemporaryRedirect,
      #[doc = "PermanentRedirect status code"]
      PermanentRedirect,
      #[doc = "BadRequest status code"]
      BadRequest,
      #[doc = "Unauthorized status code"]
      Unauthorized,
      #[doc = "PaymentRequired status code"]
      PaymentRequired,
      #[doc = "Forbidden status code"]
      Forbidden,
      #[doc = "NotFound status code"]
      NotFound,
      #[doc = "MethodNotAllowed status code"]
      MethodNotAllowed,
      #[doc = "NotAcceptable status code"]
      NotAcceptable,
      #[doc = "ProxyAuthenticationRequired status code"]
      ProxyAuthenticationRequired,
      #[doc = "RequestTimeout status code"]
      RequestTimeout,
      #[doc = "Conflict status code"]
      Conflict,
      #[doc = "Gone status code"]
      Gone,
      #[doc = "LengthRequired status code"]
      LengthRequired,
      #[doc = "PreconditionFailed status code"]
      PreconditionFailed,
      #[doc = "PayloadTooLarge status code"]
      PayloadTooLarge,
      #[doc = "URITooLong status code"]
      UriTooLong,
      #[doc = "UnsupportedMediaType status code"]
      UnsupportedMediaType,
      #[doc = "RangeNotSatisfiable status code"]
      RangeNotSatisfiable,
      #[doc = "ExpectationFailed status code"]
      ExpectationFailed,
      #[doc = "ImATeapot status code"]
      ImATeapot,
      #[doc = "UnprocessableEntity status code"]
      UnprocessableEntity,
      #[doc = "Locked status code"]
      Locked,
      #[doc = "FailedDependency status code"]
      FailedDependency,
      #[doc = "TooManyRequests status code"]
      TooManyRequests,
      #[doc = "InternalServerError status code"]
      InternalServerError,
      #[doc = "NotImplemented status code"]
      NotImplemented,
      #[doc = "BadGateway status code"]
      BadGateway,
      #[doc = "ServiceUnavailable status code"]
      ServiceUnavailable,
      #[doc = "GatewayTimeout status code"]
      GatewayTimeout,
      #[doc = "HTTPVersionNotSupported status code"]
      HttpVersionNotSupported,
      #[doc = "Indicates an unknown status code"]
      Unknown,
    }
    impl TryFrom<wick_component::serde::enum_repr::StringOrNum> for StatusCode {
      type Error = String;
      fn try_from(value: wick_component::serde::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
        }
      }
    }
    impl From<StatusCode> for String {
      fn from(value: StatusCode) -> Self {
        value.value().map_or_else(|| value.to_string(), |v| v.to_owned())
      }
    }
    impl StatusCode {
      #[allow(unused)]
      #[doc = "Returns the value of the enum variant as a string."]
      #[must_use]
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
          "Continue" => Ok(Self::Continue),
          "101" => Ok(Self::SwitchingProtocols),
          "SwitchingProtocols" => Ok(Self::SwitchingProtocols),
          "200" => Ok(Self::Ok),
          "Ok" => Ok(Self::Ok),
          "201" => Ok(Self::Created),
          "Created" => Ok(Self::Created),
          "202" => Ok(Self::Accepted),
          "Accepted" => Ok(Self::Accepted),
          "203" => Ok(Self::NonAuthoritativeInformation),
          "NonAuthoritativeInformation" => Ok(Self::NonAuthoritativeInformation),
          "204" => Ok(Self::NoContent),
          "NoContent" => Ok(Self::NoContent),
          "205" => Ok(Self::ResetContent),
          "ResetContent" => Ok(Self::ResetContent),
          "206" => Ok(Self::PartialContent),
          "PartialContent" => Ok(Self::PartialContent),
          "300" => Ok(Self::MultipleChoices),
          "MultipleChoices" => Ok(Self::MultipleChoices),
          "301" => Ok(Self::MovedPermanently),
          "MovedPermanently" => Ok(Self::MovedPermanently),
          "302" => Ok(Self::Found),
          "Found" => Ok(Self::Found),
          "303" => Ok(Self::SeeOther),
          "SeeOther" => Ok(Self::SeeOther),
          "304" => Ok(Self::NotModified),
          "NotModified" => Ok(Self::NotModified),
          "307" => Ok(Self::TemporaryRedirect),
          "TemporaryRedirect" => Ok(Self::TemporaryRedirect),
          "308" => Ok(Self::PermanentRedirect),
          "PermanentRedirect" => Ok(Self::PermanentRedirect),
          "400" => Ok(Self::BadRequest),
          "BadRequest" => Ok(Self::BadRequest),
          "401" => Ok(Self::Unauthorized),
          "Unauthorized" => Ok(Self::Unauthorized),
          "402" => Ok(Self::PaymentRequired),
          "PaymentRequired" => Ok(Self::PaymentRequired),
          "403" => Ok(Self::Forbidden),
          "Forbidden" => Ok(Self::Forbidden),
          "404" => Ok(Self::NotFound),
          "NotFound" => Ok(Self::NotFound),
          "405" => Ok(Self::MethodNotAllowed),
          "MethodNotAllowed" => Ok(Self::MethodNotAllowed),
          "406" => Ok(Self::NotAcceptable),
          "NotAcceptable" => Ok(Self::NotAcceptable),
          "407" => Ok(Self::ProxyAuthenticationRequired),
          "ProxyAuthenticationRequired" => Ok(Self::ProxyAuthenticationRequired),
          "408" => Ok(Self::RequestTimeout),
          "RequestTimeout" => Ok(Self::RequestTimeout),
          "409" => Ok(Self::Conflict),
          "Conflict" => Ok(Self::Conflict),
          "410" => Ok(Self::Gone),
          "Gone" => Ok(Self::Gone),
          "411" => Ok(Self::LengthRequired),
          "LengthRequired" => Ok(Self::LengthRequired),
          "412" => Ok(Self::PreconditionFailed),
          "PreconditionFailed" => Ok(Self::PreconditionFailed),
          "413" => Ok(Self::PayloadTooLarge),
          "PayloadTooLarge" => Ok(Self::PayloadTooLarge),
          "414" => Ok(Self::UriTooLong),
          "UriTooLong" => Ok(Self::UriTooLong),
          "415" => Ok(Self::UnsupportedMediaType),
          "UnsupportedMediaType" => Ok(Self::UnsupportedMediaType),
          "416" => Ok(Self::RangeNotSatisfiable),
          "RangeNotSatisfiable" => Ok(Self::RangeNotSatisfiable),
          "417" => Ok(Self::ExpectationFailed),
          "ExpectationFailed" => Ok(Self::ExpectationFailed),
          "418" => Ok(Self::ImATeapot),
          "ImATeapot" => Ok(Self::ImATeapot),
          "422" => Ok(Self::UnprocessableEntity),
          "UnprocessableEntity" => Ok(Self::UnprocessableEntity),
          "423" => Ok(Self::Locked),
          "Locked" => Ok(Self::Locked),
          "424" => Ok(Self::FailedDependency),
          "FailedDependency" => Ok(Self::FailedDependency),
          "429" => Ok(Self::TooManyRequests),
          "TooManyRequests" => Ok(Self::TooManyRequests),
          "500" => Ok(Self::InternalServerError),
          "InternalServerError" => Ok(Self::InternalServerError),
          "501" => Ok(Self::NotImplemented),
          "NotImplemented" => Ok(Self::NotImplemented),
          "502" => Ok(Self::BadGateway),
          "BadGateway" => Ok(Self::BadGateway),
          "503" => Ok(Self::ServiceUnavailable),
          "ServiceUnavailable" => Ok(Self::ServiceUnavailable),
          "504" => Ok(Self::GatewayTimeout),
          "GatewayTimeout" => Ok(Self::GatewayTimeout),
          "505" => Ok(Self::HttpVersionNotSupported),
          "HttpVersionNotSupported" => Ok(Self::HttpVersionNotSupported),
          "-1" => Ok(Self::Unknown),
          "Unknown" => Ok(Self::Unknown),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for StatusCode {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Continue => f.write_str("100"),
          Self::SwitchingProtocols => f.write_str("101"),
          Self::Ok => f.write_str("200"),
          Self::Created => f.write_str("201"),
          Self::Accepted => f.write_str("202"),
          Self::NonAuthoritativeInformation => f.write_str("203"),
          Self::NoContent => f.write_str("204"),
          Self::ResetContent => f.write_str("205"),
          Self::PartialContent => f.write_str("206"),
          Self::MultipleChoices => f.write_str("300"),
          Self::MovedPermanently => f.write_str("301"),
          Self::Found => f.write_str("302"),
          Self::SeeOther => f.write_str("303"),
          Self::NotModified => f.write_str("304"),
          Self::TemporaryRedirect => f.write_str("307"),
          Self::PermanentRedirect => f.write_str("308"),
          Self::BadRequest => f.write_str("400"),
          Self::Unauthorized => f.write_str("401"),
          Self::PaymentRequired => f.write_str("402"),
          Self::Forbidden => f.write_str("403"),
          Self::NotFound => f.write_str("404"),
          Self::MethodNotAllowed => f.write_str("405"),
          Self::NotAcceptable => f.write_str("406"),
          Self::ProxyAuthenticationRequired => f.write_str("407"),
          Self::RequestTimeout => f.write_str("408"),
          Self::Conflict => f.write_str("409"),
          Self::Gone => f.write_str("410"),
          Self::LengthRequired => f.write_str("411"),
          Self::PreconditionFailed => f.write_str("412"),
          Self::PayloadTooLarge => f.write_str("413"),
          Self::UriTooLong => f.write_str("414"),
          Self::UnsupportedMediaType => f.write_str("415"),
          Self::RangeNotSatisfiable => f.write_str("416"),
          Self::ExpectationFailed => f.write_str("417"),
          Self::ImATeapot => f.write_str("418"),
          Self::UnprocessableEntity => f.write_str("422"),
          Self::Locked => f.write_str("423"),
          Self::FailedDependency => f.write_str("424"),
          Self::TooManyRequests => f.write_str("429"),
          Self::InternalServerError => f.write_str("500"),
          Self::NotImplemented => f.write_str("501"),
          Self::BadGateway => f.write_str("502"),
          Self::ServiceUnavailable => f.write_str("503"),
          Self::GatewayTimeout => f.write_str("504"),
          Self::HttpVersionNotSupported => f.write_str("505"),
          Self::Unknown => f.write_str("-1"),
        }
      }
    }
  }
}
#[doc = "Types associated with the `echo` operation"]
pub mod echo {
  use super::*;
  #[derive(Debug, Clone, Default, serde :: Serialize, serde :: Deserialize, PartialEq)]
  pub struct Config {}
  pub struct Outputs {
    #[allow(unused)]
    pub(crate) output: wick_packet::Output<types::http::HttpRequest>,
    pub(crate) time: wick_packet::Output<wick_component::datetime::DateTime>,
  }
  impl Outputs {
    pub fn new(channel: wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError>) -> Self {
      Self {
        output: wick_packet::Output::new("output", channel.clone()),
        time: wick_packet::Output::new("time", channel.clone()),
      }
    }
    #[allow(unused)]
    pub fn broadcast_open(&mut self) {
      self.output.open_bracket();
      self.time.open_bracket();
    }
    #[allow(unused)]
    pub fn broadcast_close(&mut self) {
      self.output.close_bracket();
      self.time.close_bracket();
    }
    #[allow(unused)]
    pub fn broadcast_err(&mut self, err: impl AsRef<str>) {
      self.output.error(&err);
      self.time.error(&err);
    }
  }
}
# [async_trait :: async_trait (? Send)]
#[cfg(target_family = "wasm")]
pub trait EchoOperation {
  type Error: std::fmt::Display;
  type Outputs;
  type Config: std::fmt::Debug;
  #[allow(unused)]
  async fn echo(
    input: WickStream<types::http::HttpRequest>,
    time: WickStream<wick_component::datetime::DateTime>,
    outputs: Self::Outputs,
    ctx: wick_component::flow_component::Context<Self::Config>,
  ) -> std::result::Result<(), Self::Error>;
}
#[async_trait::async_trait]
#[cfg(not(target_family = "wasm"))]
pub trait EchoOperation {
  type Error: std::fmt::Display + Send;
  type Outputs: Send;
  type Config: std::fmt::Debug + Send;
  #[allow(unused)]
  async fn echo(
    input: WickStream<types::http::HttpRequest>,
    time: WickStream<wick_component::datetime::DateTime>,
    outputs: Self::Outputs,
    ctx: wick_component::flow_component::Context<Self::Config>,
  ) -> std::result::Result<(), Self::Error>;
}
#[doc = "Types associated with the `testop` operation"]
pub mod testop {
  use super::*;
  #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
  pub struct Config {
    #[serde(rename = "A")]
    pub a: String,
    #[serde(rename = "B")]
    pub b: u32,
  }
  impl Default for Config {
    fn default() -> Self {
      Self {
        a: Default::default(),
        b: Default::default(),
      }
    }
  }
  pub struct Outputs {
    #[allow(unused)]
    pub(crate) output: wick_packet::Output<String>,
  }
  impl Outputs {
    pub fn new(channel: wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError>) -> Self {
      Self {
        output: wick_packet::Output::new("output", channel.clone()),
      }
    }
    #[allow(unused)]
    pub fn broadcast_open(&mut self) {
      self.output.open_bracket();
    }
    #[allow(unused)]
    pub fn broadcast_close(&mut self) {
      self.output.close_bracket();
    }
    #[allow(unused)]
    pub fn broadcast_err(&mut self, err: impl AsRef<str>) {
      self.output.error(&err);
    }
  }
}
# [async_trait :: async_trait (? Send)]
#[cfg(target_family = "wasm")]
pub trait TestopOperation {
  type Error: std::fmt::Display;
  type Outputs;
  type Config: std::fmt::Debug;
  #[allow(unused)]
  async fn testop(
    message: WickStream<types::http::HttpResponse>,
    outputs: Self::Outputs,
    ctx: wick_component::flow_component::Context<Self::Config>,
  ) -> std::result::Result<(), Self::Error>;
}
#[async_trait::async_trait]
#[cfg(not(target_family = "wasm"))]
pub trait TestopOperation {
  type Error: std::fmt::Display + Send;
  type Outputs: Send;
  type Config: std::fmt::Debug + Send;
  #[allow(unused)]
  async fn testop(
    message: WickStream<types::http::HttpResponse>,
    outputs: Self::Outputs,
    ctx: wick_component::flow_component::Context<Self::Config>,
  ) -> std::result::Result<(), Self::Error>;
}
#[derive(Default, Clone)]
#[doc = "The struct that the component implementation hinges around"]
pub struct Component;
impl Component {
  fn echo_wrapper(
    mut input: wasmrs_rx::BoxFlux<wasmrs::Payload, wasmrs::PayloadError>,
  ) -> std::result::Result<
    wasmrs_rx::BoxFlux<wasmrs::RawPayload, wasmrs::PayloadError>,
    Box<dyn std::error::Error + Send + Sync>,
  > {
    let (channel, rx) = wasmrs_rx::FluxChannel::<wasmrs::RawPayload, wasmrs::PayloadError>::new_parts();
    let outputs = echo::Outputs::new(channel.clone());
    runtime::spawn("echo_wrapper", async move {
      let (config, input, time) = wick_component :: payload_fan_out ! (input , raw : false , Box < dyn std :: error :: Error + Send + Sync > , echo :: Config , [("input" , types :: http :: HttpRequest) , ("time" , wick_component :: datetime :: DateTime) ,]);
      let config = match config.await {
        Ok(Ok(config)) => config,
        Err(e) => {
          let _ = channel
            .send_result(wick_packet::Packet::component_error(format!("Component sent invalid context: {}", e)).into());
          return;
        }
        Ok(Err(e)) => {
          let _ = channel
            .send_result(wick_packet::Packet::component_error(format!("Component sent invalid context: {}", e)).into());
          return;
        }
      };
      if let Err(e) = Component::echo(Box::pin(input), Box::pin(time), outputs, config).await {
        let _ = channel.send_result(wick_packet::Packet::component_error(e.to_string()).into());
      }
    });
    Ok(Box::pin(rx))
  }
  fn testop_wrapper(
    mut input: wasmrs_rx::BoxFlux<wasmrs::Payload, wasmrs::PayloadError>,
  ) -> std::result::Result<
    wasmrs_rx::BoxFlux<wasmrs::RawPayload, wasmrs::PayloadError>,
    Box<dyn std::error::Error + Send + Sync>,
  > {
    let (channel, rx) = wasmrs_rx::FluxChannel::<wasmrs::RawPayload, wasmrs::PayloadError>::new_parts();
    let outputs = testop::Outputs::new(channel.clone());
    runtime::spawn("testop_wrapper", async move {
      let (config, message) = wick_component :: payload_fan_out ! (input , raw : false , Box < dyn std :: error :: Error + Send + Sync > , testop :: Config , [("message" , types :: http :: HttpResponse) ,]);
      let config = match config.await {
        Ok(Ok(config)) => config,
        Err(e) => {
          let _ = channel
            .send_result(wick_packet::Packet::component_error(format!("Component sent invalid context: {}", e)).into());
          return;
        }
        Ok(Err(e)) => {
          let _ = channel
            .send_result(wick_packet::Packet::component_error(format!("Component sent invalid context: {}", e)).into());
          return;
        }
      };
      if let Err(e) = Component::testop(Box::pin(message), outputs, config).await {
        let _ = channel.send_result(wick_packet::Packet::component_error(e.to_string()).into());
      }
    });
    Ok(Box::pin(rx))
  }
}
