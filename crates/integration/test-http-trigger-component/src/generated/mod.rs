pub use async_trait::async_trait;
pub use bytes::Bytes;
#[allow(unused)]
pub(crate) use wick_component::wasmrs_rx::{Observable, Observer};
pub use wick_component::{packet as wick_packet, runtime, wasmrs, wasmrs_codec, wasmrs_rx};
#[allow(unused)]
pub(crate) type WickStream<T> = wick_component::wasmrs_rx::BoxFlux<T, wick_component::anyhow::Error>;
pub use wick_component::anyhow::Result;
pub use wick_component::flow_component::Context;
#[no_mangle]
#[cfg(target_family = "wasm")]
extern "C" fn __wasmrs_init(guest_buffer_size: u32, host_buffer_size: u32, max_host_frame_len: u32) {
  wasmrs_guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);
  wasmrs_guest::register_request_response("wick", "__setup", Box::new(__setup));
  wasmrs_guest::register_request_channel("wick", "http_handler", Box::new(Component::http_handler_wrapper));
}
#[cfg(target_family = "wasm")]
thread_local! { static __CONFIG : std :: cell :: UnsafeCell < Option < SetupPayload >> = std :: cell :: UnsafeCell :: new (None) ; }
#[cfg(target_family = "wasm")]
#[derive(Debug, serde :: Deserialize)]
pub(crate) struct SetupPayload {
  #[allow(unused)]
  pub(crate) provided: std::collections::HashMap<String, wick_packet::ComponentReference>,
}
#[cfg(target_family = "wasm")]
fn __setup(
  input: wasmrs_rx::BoxMono<wasmrs::Payload, wasmrs::PayloadError>,
) -> Result<wasmrs_rx::BoxMono<wasmrs::RawPayload, wasmrs::PayloadError>, wick_component::BoxError> {
  Ok(Box::pin(async move {
    match input.await {
      Ok(payload) => {
        let input = wasmrs_codec::messagepack::deserialize::<SetupPayload>(&payload.data).unwrap();
        __CONFIG.with(|cell| {
          #[allow(unsafe_code)]
          unsafe { &mut *cell.get() }.replace(input);
        });
        Ok(wasmrs::RawPayload::new_data(None, None))
      }
      Err(e) => {
        return Err(e);
      }
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
pub mod types {
  #[allow(unused)]
  use super::types;
  pub mod http {
    #[allow(unused)]
    use super::http;
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
  }
}
#[derive(Debug, Clone, Default, serde :: Serialize, serde :: Deserialize, PartialEq)]
pub struct OpHttpHandlerConfig {}
pub struct OpHttpHandlerOutputs {
  #[allow(unused)]
  pub(crate) body: wick_packet::Output<bytes::Bytes>,
  pub(crate) response: wick_packet::Output<types::http::HttpResponse>,
}
impl OpHttpHandlerOutputs {
  pub fn new(channel: wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError>) -> Self {
    Self {
      body: wick_packet::Output::new("body", channel.clone()),
      response: wick_packet::Output::new("response", channel.clone()),
    }
  }
}
# [cfg_attr (target_family = "wasm" , async_trait :: async_trait (? Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
pub trait OpHttpHandler {
  #[allow(unused)]
  async fn http_handler(
    request: WickStream<types::http::HttpRequest>,
    body: WickStream<bytes::Bytes>,
    outputs: OpHttpHandlerOutputs,
    ctx: wick_component::flow_component::Context<OpHttpHandlerConfig>,
  ) -> Result<()> {
    unimplemented!()
  }
}
#[derive(Default, Clone)]
pub struct Component;
impl Component {
  fn http_handler_wrapper(
    mut input: wasmrs_rx::BoxFlux<wasmrs::Payload, wasmrs::PayloadError>,
  ) -> std::result::Result<
    wasmrs_rx::BoxFlux<wasmrs::RawPayload, wasmrs::PayloadError>,
    Box<dyn std::error::Error + Send + Sync>,
  > {
    let (channel, rx) = wasmrs_rx::FluxChannel::<wasmrs::RawPayload, wasmrs::PayloadError>::new_parts();
    let outputs = OpHttpHandlerOutputs::new(channel.clone());
    runtime::spawn("http_handler_wrapper", async move {
      let (config, request, body) = wick_component :: payload_fan_out ! (input , raw : false , OpHttpHandlerConfig , [("request" , types :: http :: HttpRequest) , ("body" , bytes :: Bytes) ,]);
      let config = match config.await {
        Ok(Ok(config)) => config,
        _ => {
          let _ = channel.send_result(wick_packet::Packet::component_error("Component sent invalid context").into());
          return;
        }
      };
      if let Err(e) = Component::http_handler(Box::pin(request), Box::pin(body), outputs, config).await {
        let _ = channel.send_result(wick_packet::Packet::component_error(e.to_string()).into());
      }
    });
    Ok(Box::pin(rx))
  }
}
