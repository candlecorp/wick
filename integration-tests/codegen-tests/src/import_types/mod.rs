pub use async_trait::async_trait;
pub use wick_component::flow_component::Context;
#[allow(unused)]
pub(crate) use wick_component::WickStream;
#[allow(unused)]
pub(crate) use wick_component::*;
#[no_mangle]
#[cfg(target_family = "wasm")]
extern "C" fn __wasmrs_init(guest_buffer_size: u32, host_buffer_size: u32, max_host_frame_len: u32) {
  wick_component::wasmrs_guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);
  wick_component::wasmrs_guest::register_request_response("wick", "__setup", Box::new(__setup));
  wick_component::wasmrs_guest::register_request_channel("wick", "echo", Box::new(Component::echo_wrapper));
  wick_component::wasmrs_guest::register_request_channel("wick", "testop", Box::new(Component::testop_wrapper));
}
#[cfg(target_family = "wasm")]
mod provided {
mod provided {
  #[allow(unused)]
  use super::*;
  #[allow(unused)]
  pub struct Dep1Component {
    component: wick_packet::ComponentReference,
    inherent: flow_component::InherentContext,
  }
  impl Dep1Component {
    pub fn new(component: wick_packet::ComponentReference, inherent: flow_component::InherentContext) -> Self {
      Self { component, inherent }
    }
    #[allow(unused)]
    pub fn component(&self) -> &wick_packet::ComponentReference {
      &self.component
    }
    #[allow(unused)]
    pub fn echo(
      &self,
      input: impl wick_component::Stream<Item = Result<types::http::HttpRequest, wick_component::AnyError>> + 'static,
    ) -> std::result::Result<(WickStream<types::http::HttpRequest>), wick_packet::Error> {
      let input = input.map(wick_component::wick_packet::into_packet("input"));
      let stream = wick_component::empty();
      let stream = stream
        .merge(input)
        .chain(wick_component::iter_raw(vec![Ok(Packet::done("input"))]));
      let stream = wick_packet::PacketStream::new(Box::pin(stream));
      let mut stream = self.echo_raw(stream)?;
      Ok(wick_component::payload_fan_out!(
          stream, raw : false, wick_component::AnyError, [("output",
          types::http::HttpRequest)]
      ))
    }
    #[allow(unused)]
    pub fn echo_raw<T: Into<wick_packet::PacketStream>>(
    pub fn echo_raw<T: Into<wick_packet::PacketStream>>(
      &self,
      stream: T,
      stream: T,
    ) -> std::result::Result<wick_packet::PacketStream, wick_packet::Error> {
      Ok(
        self
          .component
          .call("echo", stream.into(), None, self.inherent.clone().into())?,
      )
    }
  }
}
#[cfg(target_family = "wasm")]
pub use provided::*;
#[cfg(target_family = "wasm")]
mod imported {
  #[allow(unused)]
  use super::*;
  #[allow(unused)]
  pub struct ImportedComponentComponent {
    component: wick_packet::ComponentReference,
    inherent: flow_component::InherentContext,
  }
  impl ImportedComponentComponent {
    pub fn new(component: wick_packet::ComponentReference, inherent: flow_component::InherentContext) -> Self {
      Self { component, inherent }
    }
    #[allow(unused)]
    pub fn component(&self) -> &wick_packet::ComponentReference {
      &self.component
    }
    #[allow(unused)]
    pub fn add(
      &self,
      left: impl wick_component::Stream<Item = Result<u64, wick_component::AnyError>> + 'static,
      right: impl wick_component::Stream<Item = Result<u64, wick_component::AnyError>> + 'static,
    ) -> std::result::Result<(WickStream<u64>), wick_packet::Error> {
      let left = left.map(wick_component::wick_packet::into_packet("left"));
      let right = right.map(wick_component::wick_packet::into_packet("right"));
      let stream = wick_component::empty();
      let stream = stream.merge(left).merge(right).chain(wick_component::iter_raw(vec![
        Ok(Packet::done("left")),
        Ok(Packet::done("right")),
      ]));
      let stream = wick_packet::PacketStream::new(Box::pin(stream));
      let mut stream = self.add_raw(stream)?;
      Ok(wick_component::payload_fan_out!(
          stream, raw : false, wick_component::AnyError, [("output", u64)]
      ))
    }
    #[allow(unused)]
    pub fn add_raw<T: Into<wick_packet::PacketStream>>(
      &self,
      stream: T,
    ) -> std::result::Result<wick_packet::PacketStream, wick_packet::Error> {
      Ok(
        self
          .component
          .call("add", stream.into(), None, self.inherent.clone().into())?,
      )
    }
    #[allow(unused)]
    pub fn error(
      &self,
      input: impl wick_component::Stream<Item = Result<String, wick_component::AnyError>> + 'static,
    ) -> std::result::Result<(WickStream<String>), wick_packet::Error> {
      let input = input.map(wick_component::wick_packet::into_packet("input"));
      let stream = wick_component::empty();
      let stream = stream
        .merge(input)
        .chain(wick_component::iter_raw(vec![Ok(Packet::done("input"))]));
      let stream = wick_packet::PacketStream::new(Box::pin(stream));
      let mut stream = self.error_raw(stream)?;
      Ok(wick_component::payload_fan_out!(
          stream, raw : false, wick_component::AnyError, [("output", String)]
      ))
    }
    #[allow(unused)]
    pub fn error_raw<T: Into<wick_packet::PacketStream>>(
      &self,
      stream: T,
    ) -> std::result::Result<wick_packet::PacketStream, wick_packet::Error> {
      Ok(
        self
          .component
          .call("error", stream.into(), None, self.inherent.clone().into())?,
          .call("error", stream.into(), None, self.inherent.clone().into())?,
      )
    }
    #[allow(unused)]
    pub fn validate(
      &self,
      input: impl wick_component::Stream<Item = Result<String, wick_component::AnyError>> + 'static,
    ) -> std::result::Result<(WickStream<String>), wick_packet::Error> {
      let input = input.map(wick_component::wick_packet::into_packet("input"));
      let stream = wick_component::empty();
      let stream = stream
        .merge(input)
        .chain(wick_component::iter_raw(vec![Ok(Packet::done("input"))]));
      let stream = wick_packet::PacketStream::new(Box::pin(stream));
      let mut stream = self.validate_raw(stream)?;
      Ok(wick_component::payload_fan_out!(
          stream, raw : false, wick_component::AnyError, [("output", String)]
      ))
    }
    #[allow(unused)]
    pub fn validate_raw<T: Into<wick_packet::PacketStream>>(
      &self,
      stream: T,
    ) -> std::result::Result<wick_packet::PacketStream, wick_packet::Error> {
      Ok(
        self
          .component
          .call("validate", stream.into(), None, self.inherent.clone().into())?,
      )
    }
  }
}
#[cfg(target_family = "wasm")]
pub use imported::*;
pub use imported::*;
#[allow(unused)]
#[cfg(target_family = "wasm")]
mod provided_wasm {
  #[allow(unused)]
  use super::*;
  pub(crate) struct Provided {
    pub dep1: Dep1Component,
  }
  pub(crate) trait ProvidedContext {
    fn provided(&self) -> Provided;
  }
  impl<T> ProvidedContext for wick_component::flow_component::Context<T>
  where
    T: std::fmt::Debug,
  {
    fn provided(&self) -> Provided {
      let config = get_config();
      let inherent = self.inherent.clone();
      Provided {
        dep1: Dep1Component::new(config.provided.get("DEP1").cloned().unwrap(), inherent.clone()),
      }
      let config = get_config();
      let inherent = self.inherent.clone();
      Provided {
        dep1: Dep1Component::new(config.provided.get("DEP1").cloned().unwrap(), inherent.clone()),
      }
    }
  }
}
#[cfg(target_family = "wasm")]
pub(crate) use provided_wasm::*;
#[allow(unused)]
#[cfg(target_family = "wasm")]
mod imported_wasm {
  #[allow(unused)]
  use super::*;
  pub(crate) struct Imported {
    pub http: HttpComponent,
    pub aaa: AaaComponent,
    pub zzz: ZzzComponent,
    pub imported_component: ImportedComponentComponent,
  }
  pub(crate) trait ImportedContext {
    fn imported(&self) -> Imported;
  }
  impl<T> ImportedContext for wick_component::flow_component::Context<T>
  where
    T: std::fmt::Debug,
  {
    fn imported(&self) -> Imported {
      let config = get_config();
      let inherent = self.inherent.clone();
      Imported {
        http: HttpComponent::new(config.imported.get("http").cloned().unwrap(), inherent.clone()),
        aaa: AaaComponent::new(config.imported.get("AAA").cloned().unwrap(), inherent.clone()),
        zzz: ZzzComponent::new(config.imported.get("ZZZ").cloned().unwrap(), inherent.clone()),
        imported_component: ImportedComponentComponent::new(
          config.imported.get("IMPORTED_COMPONENT").cloned().unwrap(),
          inherent.clone(),
        ),
      }
    }
  }
}
#[cfg(target_family = "wasm")]
pub(crate) use imported_wasm::*;
#[allow(unused)]
#[cfg(target_family = "wasm")]
mod imported_wasm {
  #[allow(unused)]
  use super::*;
  pub(crate) struct Imported {
    pub http: HttpComponent,
    pub aaa: AaaComponent,
    pub zzz: ZzzComponent,
    pub imported_component: ImportedComponentComponent,
  }
  pub(crate) trait ImportedContext {
    fn imported(&self) -> Imported;
  }
  impl<T> ImportedContext for wick_component::flow_component::Context<T>
  where
    T: std::fmt::Debug,
  {
    fn imported(&self) -> Imported {
      let config = get_config();
      let inherent = self.inherent.clone();
      Imported {
        http: HttpComponent::new(config.imported.get("http").cloned().unwrap(), inherent.clone()),
        aaa: AaaComponent::new(config.imported.get("AAA").cloned().unwrap(), inherent.clone()),
        zzz: ZzzComponent::new(config.imported.get("ZZZ").cloned().unwrap(), inherent.clone()),
        imported_component: ImportedComponentComponent::new(
          config.imported.get("IMPORTED_COMPONENT").cloned().unwrap(),
          inherent.clone(),
        ),
      }
    }
  }
}
#[cfg(target_family = "wasm")]
pub(crate) use imported_wasm::*;
#[cfg(target_family = "wasm")]
thread_local! {
    static __CONFIG : std::cell::UnsafeCell < Option < SetupPayload >> =
    std::cell::UnsafeCell::new(None);
}
#[derive(Debug, Clone, Default, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
#[allow(clippy::exhaustive_structs)]
pub struct RootConfig {}
#[cfg(target_family = "wasm")]
#[derive(Debug, ::serde::Deserialize)]
pub(crate) struct SetupPayload {
  #[allow(unused)]
  pub(crate) provided: std::collections::HashMap<String, wick_packet::ComponentReference>,
  #[allow(unused)]
  pub(crate) imported: std::collections::HashMap<String, wick_packet::ComponentReference>,
  #[allow(unused)]
  pub(crate) imported: std::collections::HashMap<String, wick_packet::ComponentReference>,
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
///Additional generated types
pub mod types {
  #[allow(unused)]
  use super::types;
  #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
  ///a useful struct
  #[allow(clippy::exhaustive_structs)]
  pub struct LocalStruct {
    #[serde(rename = "field1")]
    pub field1: String,
    #[serde(rename = "inner")]
    pub inner: types::LocalStructInner,
    #[serde(rename = "time")]
    #[serde(deserialize_with = "wick_component::datetime::serde::from_str_or_integer")]
    pub time: wick_component::datetime::DateTime,
  }
  #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
  #[allow(clippy::exhaustive_structs)]
  pub struct LocalStructInner {
    #[serde(rename = "field1")]
    pub field1: String,
    #[serde(rename = "field2")]
    pub field2: String,
  }
  #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
  ///a weird union
  #[serde(untagged)]
  pub enum LocalUnion {
    ///A string value.
    String(String),
    ///A LocalStructInner value.
    LocalStructInner(types::LocalStructInner),
    ///A datetime value.
    Datetime(wick_component::datetime::DateTime),
  }
  pub mod aaa {
    #[allow(unused)]
    use super::aaa;
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP method enum
    #[serde(into = "String", try_from = "wick_component::serde_util::enum_repr::StringOrNum")]
    #[allow(clippy::exhaustive_enums)]
    pub enum HttpMethod {
      ///HTTP GET method
      Get,
      ///HTTP POST method
      Post,
      ///HTTP PUT method
      Put,
      ///HTTP DELETE method
      Delete,
      ///HTTP PATCH method
      Patch,
      ///HTTP HEAD method
      Head,
      ///HTTP OPTIONS method
      Options,
      ///HTTP TRACE method
      Trace,
    }
    impl TryFrom<wick_component::serde_util::enum_repr::StringOrNum> for HttpMethod {
      type Error = String;
      fn try_from(value: wick_component::serde_util::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde_util::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde_util::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde_util::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
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
      ///Returns the value of the enum variant as a string.
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
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP request
    #[allow(clippy::exhaustive_structs)]
    pub struct HttpRequest {
      ///method from request line enum
      #[serde(rename = "method")]
      pub method: HttpMethod,
      ///scheme from request line enum
      #[serde(rename = "scheme")]
      pub scheme: HttpScheme,
      ///domain/port and any authentication from request line. optional
      #[serde(rename = "authority")]
      pub authority: String,
      ///query parameters from request line. optional
      #[serde(rename = "query_parameters")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub query_parameters: std::collections::HashMap<String, Vec<String>>,
      ///path from request line (not including query parameters)
      #[serde(rename = "path")]
      pub path: String,
      ///full URI from request line
      #[serde(rename = "uri")]
      pub uri: String,
      ///HTTP version enum
      #[serde(rename = "version")]
      pub version: HttpVersion,
      ///All request headers. Duplicates are comma separated
      #[serde(rename = "headers")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub headers: std::collections::HashMap<String, Vec<String>>,
      ///The remote address of the connected client
      #[serde(rename = "remote_addr")]
      pub remote_addr: String,
    }
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP response
    #[allow(clippy::exhaustive_structs)]
    pub struct HttpResponse {
      ///HTTP version enum
      #[serde(rename = "version")]
      pub version: HttpVersion,
      ///status code enum
      #[serde(rename = "status")]
      pub status: StatusCode,
      ///All response headers. Supports duplicates.
      #[serde(rename = "headers")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub headers: std::collections::HashMap<String, Vec<String>>,
    }
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP scheme
    #[serde(into = "String", try_from = "wick_component::serde_util::enum_repr::StringOrNum")]
    #[allow(clippy::exhaustive_enums)]
    pub enum HttpScheme {
      ///HTTP scheme
      Http,
      ///HTTPS scheme
      Https,
    }
    impl TryFrom<wick_component::serde_util::enum_repr::StringOrNum> for HttpScheme {
      type Error = String;
      fn try_from(value: wick_component::serde_util::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde_util::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde_util::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde_util::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
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
      ///Returns the value of the enum variant as a string.
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
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP version
    #[serde(into = "String", try_from = "wick_component::serde_util::enum_repr::StringOrNum")]
    #[allow(clippy::exhaustive_enums)]
    pub enum HttpVersion {
      ///HTTP 1.0 version
      Http10,
      ///HTTP 1.1 version
      Http11,
      ///HTTP 2.0 version
      Http20,
    }
    impl TryFrom<wick_component::serde_util::enum_repr::StringOrNum> for HttpVersion {
      type Error = String;
      fn try_from(value: wick_component::serde_util::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde_util::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde_util::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde_util::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
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
      ///Returns the value of the enum variant as a string.
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
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///A response from pre-request middleware
    #[serde(untagged)]
    pub enum RequestMiddlewareResponse {
      ///A HttpRequest value.
      HttpRequest(HttpRequest),
      ///A HttpResponse value.
      HttpResponse(HttpResponse),
    }
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP status code
    #[serde(into = "String", try_from = "wick_component::serde_util::enum_repr::StringOrNum")]
    #[allow(clippy::exhaustive_enums)]
    pub enum StatusCode {
      ///Continue status code
      Continue,
      ///SwitchingProtocols status code
      SwitchingProtocols,
      ///HTTP OK status code
      Ok,
      ///Created status code
      Created,
      ///Accepted status code
      Accepted,
      ///NonAuthoritativeInformation status code
      NonAuthoritativeInformation,
      ///NoContent status code
      NoContent,
      ///ResetContent status code
      ResetContent,
      ///PartialContent status code
      PartialContent,
      ///MultipleChoices status code
      MultipleChoices,
      ///MovedPermanently status code
      MovedPermanently,
      ///Found status code
      Found,
      ///SeeOther status code
      SeeOther,
      ///NotModified status code
      NotModified,
      ///TemporaryRedirect status code
      TemporaryRedirect,
      ///PermanentRedirect status code
      PermanentRedirect,
      ///BadRequest status code
      BadRequest,
      ///Unauthorized status code
      Unauthorized,
      ///PaymentRequired status code
      PaymentRequired,
      ///Forbidden status code
      Forbidden,
      ///NotFound status code
      NotFound,
      ///MethodNotAllowed status code
      MethodNotAllowed,
      ///NotAcceptable status code
      NotAcceptable,
      ///ProxyAuthenticationRequired status code
      ProxyAuthenticationRequired,
      ///RequestTimeout status code
      RequestTimeout,
      ///Conflict status code
      Conflict,
      ///Gone status code
      Gone,
      ///LengthRequired status code
      LengthRequired,
      ///PreconditionFailed status code
      PreconditionFailed,
      ///PayloadTooLarge status code
      PayloadTooLarge,
      ///URITooLong status code
      UriTooLong,
      ///UnsupportedMediaType status code
      UnsupportedMediaType,
      ///RangeNotSatisfiable status code
      RangeNotSatisfiable,
      ///ExpectationFailed status code
      ExpectationFailed,
      ///ImATeapot status code
      ImATeapot,
      ///UnprocessableEntity status code
      UnprocessableEntity,
      ///Locked status code
      Locked,
      ///FailedDependency status code
      FailedDependency,
      ///TooManyRequests status code
      TooManyRequests,
      ///InternalServerError status code
      InternalServerError,
      ///NotImplemented status code
      NotImplemented,
      ///BadGateway status code
      BadGateway,
      ///ServiceUnavailable status code
      ServiceUnavailable,
      ///GatewayTimeout status code
      GatewayTimeout,
      ///HTTPVersionNotSupported status code
      HttpVersionNotSupported,
      ///Indicates an unknown status code
      Unknown,
    }
    impl TryFrom<wick_component::serde_util::enum_repr::StringOrNum> for StatusCode {
      type Error = String;
      fn try_from(value: wick_component::serde_util::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde_util::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde_util::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde_util::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
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
      ///Returns the value of the enum variant as a string.
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
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP method enum
    #[serde(into = "String", try_from = "wick_component::serde_util::enum_repr::StringOrNum")]
    #[allow(clippy::exhaustive_enums)]
    pub enum HttpMethod {
      ///HTTP GET method
      Get,
      ///HTTP POST method
      Post,
      ///HTTP PUT method
      Put,
      ///HTTP DELETE method
      Delete,
      ///HTTP PATCH method
      Patch,
      ///HTTP HEAD method
      Head,
      ///HTTP OPTIONS method
      Options,
      ///HTTP TRACE method
      Trace,
    }
    impl TryFrom<wick_component::serde_util::enum_repr::StringOrNum> for HttpMethod {
      type Error = String;
      fn try_from(value: wick_component::serde_util::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde_util::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde_util::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde_util::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
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
      ///Returns the value of the enum variant as a string.
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
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP request
    #[allow(clippy::exhaustive_structs)]
    pub struct HttpRequest {
      ///method from request line enum
      #[serde(rename = "method")]
      pub method: HttpMethod,
      ///scheme from request line enum
      #[serde(rename = "scheme")]
      pub scheme: HttpScheme,
      ///domain/port and any authentication from request line. optional
      #[serde(rename = "authority")]
      pub authority: String,
      ///query parameters from request line. optional
      #[serde(rename = "query_parameters")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub query_parameters: std::collections::HashMap<String, Vec<String>>,
      ///path from request line (not including query parameters)
      #[serde(rename = "path")]
      pub path: String,
      ///full URI from request line
      #[serde(rename = "uri")]
      pub uri: String,
      ///HTTP version enum
      #[serde(rename = "version")]
      pub version: HttpVersion,
      ///All request headers. Duplicates are comma separated
      #[serde(rename = "headers")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub headers: std::collections::HashMap<String, Vec<String>>,
      ///The remote address of the connected client
      #[serde(rename = "remote_addr")]
      pub remote_addr: String,
    }
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP response
    #[allow(clippy::exhaustive_structs)]
    pub struct HttpResponse {
      ///HTTP version enum
      #[serde(rename = "version")]
      pub version: HttpVersion,
      ///status code enum
      #[serde(rename = "status")]
      pub status: StatusCode,
      ///All response headers. Supports duplicates.
      #[serde(rename = "headers")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub headers: std::collections::HashMap<String, Vec<String>>,
    }
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP scheme
    #[serde(into = "String", try_from = "wick_component::serde_util::enum_repr::StringOrNum")]
    #[allow(clippy::exhaustive_enums)]
    pub enum HttpScheme {
      ///HTTP scheme
      Http,
      ///HTTPS scheme
      Https,
    }
    impl TryFrom<wick_component::serde_util::enum_repr::StringOrNum> for HttpScheme {
      type Error = String;
      fn try_from(value: wick_component::serde_util::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde_util::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde_util::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde_util::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
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
      ///Returns the value of the enum variant as a string.
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
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP version
    #[serde(into = "String", try_from = "wick_component::serde_util::enum_repr::StringOrNum")]
    #[allow(clippy::exhaustive_enums)]
    pub enum HttpVersion {
      ///HTTP 1.0 version
      Http10,
      ///HTTP 1.1 version
      Http11,
      ///HTTP 2.0 version
      Http20,
    }
    impl TryFrom<wick_component::serde_util::enum_repr::StringOrNum> for HttpVersion {
      type Error = String;
      fn try_from(value: wick_component::serde_util::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde_util::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde_util::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde_util::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
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
      ///Returns the value of the enum variant as a string.
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
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///A response from pre-request middleware
    #[serde(untagged)]
    pub enum RequestMiddlewareResponse {
      ///A HttpRequest value.
      HttpRequest(HttpRequest),
      ///A HttpResponse value.
      HttpResponse(HttpResponse),
    }
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP status code
    #[serde(into = "String", try_from = "wick_component::serde_util::enum_repr::StringOrNum")]
    #[allow(clippy::exhaustive_enums)]
    pub enum StatusCode {
      ///Continue status code
      Continue,
      ///SwitchingProtocols status code
      SwitchingProtocols,
      ///HTTP OK status code
      Ok,
      ///Created status code
      Created,
      ///Accepted status code
      Accepted,
      ///NonAuthoritativeInformation status code
      NonAuthoritativeInformation,
      ///NoContent status code
      NoContent,
      ///ResetContent status code
      ResetContent,
      ///PartialContent status code
      PartialContent,
      ///MultipleChoices status code
      MultipleChoices,
      ///MovedPermanently status code
      MovedPermanently,
      ///Found status code
      Found,
      ///SeeOther status code
      SeeOther,
      ///NotModified status code
      NotModified,
      ///TemporaryRedirect status code
      TemporaryRedirect,
      ///PermanentRedirect status code
      PermanentRedirect,
      ///BadRequest status code
      BadRequest,
      ///Unauthorized status code
      Unauthorized,
      ///PaymentRequired status code
      PaymentRequired,
      ///Forbidden status code
      Forbidden,
      ///NotFound status code
      NotFound,
      ///MethodNotAllowed status code
      MethodNotAllowed,
      ///NotAcceptable status code
      NotAcceptable,
      ///ProxyAuthenticationRequired status code
      ProxyAuthenticationRequired,
      ///RequestTimeout status code
      RequestTimeout,
      ///Conflict status code
      Conflict,
      ///Gone status code
      Gone,
      ///LengthRequired status code
      LengthRequired,
      ///PreconditionFailed status code
      PreconditionFailed,
      ///PayloadTooLarge status code
      PayloadTooLarge,
      ///URITooLong status code
      UriTooLong,
      ///UnsupportedMediaType status code
      UnsupportedMediaType,
      ///RangeNotSatisfiable status code
      RangeNotSatisfiable,
      ///ExpectationFailed status code
      ExpectationFailed,
      ///ImATeapot status code
      ImATeapot,
      ///UnprocessableEntity status code
      UnprocessableEntity,
      ///Locked status code
      Locked,
      ///FailedDependency status code
      FailedDependency,
      ///TooManyRequests status code
      TooManyRequests,
      ///InternalServerError status code
      InternalServerError,
      ///NotImplemented status code
      NotImplemented,
      ///BadGateway status code
      BadGateway,
      ///ServiceUnavailable status code
      ServiceUnavailable,
      ///GatewayTimeout status code
      GatewayTimeout,
      ///HTTPVersionNotSupported status code
      HttpVersionNotSupported,
      ///Indicates an unknown status code
      Unknown,
    }
    impl TryFrom<wick_component::serde_util::enum_repr::StringOrNum> for StatusCode {
      type Error = String;
      fn try_from(value: wick_component::serde_util::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde_util::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde_util::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde_util::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
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
      ///Returns the value of the enum variant as a string.
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
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP method enum
    #[serde(into = "String", try_from = "wick_component::serde_util::enum_repr::StringOrNum")]
    #[allow(clippy::exhaustive_enums)]
    pub enum HttpMethod {
      ///HTTP GET method
      Get,
      ///HTTP POST method
      Post,
      ///HTTP PUT method
      Put,
      ///HTTP DELETE method
      Delete,
      ///HTTP PATCH method
      Patch,
      ///HTTP HEAD method
      Head,
      ///HTTP OPTIONS method
      Options,
      ///HTTP TRACE method
      Trace,
    }
    impl TryFrom<wick_component::serde_util::enum_repr::StringOrNum> for HttpMethod {
      type Error = String;
      fn try_from(value: wick_component::serde_util::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde_util::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde_util::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde_util::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
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
      ///Returns the value of the enum variant as a string.
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
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP request
    #[allow(clippy::exhaustive_structs)]
    pub struct HttpRequest {
      ///method from request line enum
      #[serde(rename = "method")]
      pub method: HttpMethod,
      ///scheme from request line enum
      #[serde(rename = "scheme")]
      pub scheme: HttpScheme,
      ///domain/port and any authentication from request line. optional
      #[serde(rename = "authority")]
      pub authority: String,
      ///query parameters from request line. optional
      #[serde(rename = "query_parameters")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub query_parameters: std::collections::HashMap<String, Vec<String>>,
      ///path from request line (not including query parameters)
      #[serde(rename = "path")]
      pub path: String,
      ///full URI from request line
      #[serde(rename = "uri")]
      pub uri: String,
      ///HTTP version enum
      #[serde(rename = "version")]
      pub version: HttpVersion,
      ///All request headers. Duplicates are comma separated
      #[serde(rename = "headers")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub headers: std::collections::HashMap<String, Vec<String>>,
      ///The remote address of the connected client
      #[serde(rename = "remote_addr")]
      pub remote_addr: String,
    }
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP response
    #[allow(clippy::exhaustive_structs)]
    pub struct HttpResponse {
      ///HTTP version enum
      #[serde(rename = "version")]
      pub version: HttpVersion,
      ///status code enum
      #[serde(rename = "status")]
      pub status: StatusCode,
      ///All response headers. Supports duplicates.
      #[serde(rename = "headers")]
      #[serde(default)]
      #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
      pub headers: std::collections::HashMap<String, Vec<String>>,
    }
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP scheme
    #[serde(into = "String", try_from = "wick_component::serde_util::enum_repr::StringOrNum")]
    #[allow(clippy::exhaustive_enums)]
    pub enum HttpScheme {
      ///HTTP scheme
      Http,
      ///HTTPS scheme
      Https,
    }
    impl TryFrom<wick_component::serde_util::enum_repr::StringOrNum> for HttpScheme {
      type Error = String;
      fn try_from(value: wick_component::serde_util::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde_util::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde_util::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde_util::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
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
      ///Returns the value of the enum variant as a string.
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
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP version
    #[serde(into = "String", try_from = "wick_component::serde_util::enum_repr::StringOrNum")]
    #[allow(clippy::exhaustive_enums)]
    pub enum HttpVersion {
      ///HTTP 1.0 version
      Http10,
      ///HTTP 1.1 version
      Http11,
      ///HTTP 2.0 version
      Http20,
    }
    impl TryFrom<wick_component::serde_util::enum_repr::StringOrNum> for HttpVersion {
      type Error = String;
      fn try_from(value: wick_component::serde_util::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde_util::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde_util::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde_util::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
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
      ///Returns the value of the enum variant as a string.
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
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///A response from pre-request middleware
    #[serde(untagged)]
    pub enum RequestMiddlewareResponse {
      ///A HttpRequest value.
      HttpRequest(HttpRequest),
      ///A HttpResponse value.
      HttpResponse(HttpResponse),
    }
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
    ///HTTP status code
    #[serde(into = "String", try_from = "wick_component::serde_util::enum_repr::StringOrNum")]
    #[allow(clippy::exhaustive_enums)]
    pub enum StatusCode {
      ///Continue status code
      Continue,
      ///SwitchingProtocols status code
      SwitchingProtocols,
      ///HTTP OK status code
      Ok,
      ///Created status code
      Created,
      ///Accepted status code
      Accepted,
      ///NonAuthoritativeInformation status code
      NonAuthoritativeInformation,
      ///NoContent status code
      NoContent,
      ///ResetContent status code
      ResetContent,
      ///PartialContent status code
      PartialContent,
      ///MultipleChoices status code
      MultipleChoices,
      ///MovedPermanently status code
      MovedPermanently,
      ///Found status code
      Found,
      ///SeeOther status code
      SeeOther,
      ///NotModified status code
      NotModified,
      ///TemporaryRedirect status code
      TemporaryRedirect,
      ///PermanentRedirect status code
      PermanentRedirect,
      ///BadRequest status code
      BadRequest,
      ///Unauthorized status code
      Unauthorized,
      ///PaymentRequired status code
      PaymentRequired,
      ///Forbidden status code
      Forbidden,
      ///NotFound status code
      NotFound,
      ///MethodNotAllowed status code
      MethodNotAllowed,
      ///NotAcceptable status code
      NotAcceptable,
      ///ProxyAuthenticationRequired status code
      ProxyAuthenticationRequired,
      ///RequestTimeout status code
      RequestTimeout,
      ///Conflict status code
      Conflict,
      ///Gone status code
      Gone,
      ///LengthRequired status code
      LengthRequired,
      ///PreconditionFailed status code
      PreconditionFailed,
      ///PayloadTooLarge status code
      PayloadTooLarge,
      ///URITooLong status code
      UriTooLong,
      ///UnsupportedMediaType status code
      UnsupportedMediaType,
      ///RangeNotSatisfiable status code
      RangeNotSatisfiable,
      ///ExpectationFailed status code
      ExpectationFailed,
      ///ImATeapot status code
      ImATeapot,
      ///UnprocessableEntity status code
      UnprocessableEntity,
      ///Locked status code
      Locked,
      ///FailedDependency status code
      FailedDependency,
      ///TooManyRequests status code
      TooManyRequests,
      ///InternalServerError status code
      InternalServerError,
      ///NotImplemented status code
      NotImplemented,
      ///BadGateway status code
      BadGateway,
      ///ServiceUnavailable status code
      ServiceUnavailable,
      ///GatewayTimeout status code
      GatewayTimeout,
      ///HTTPVersionNotSupported status code
      HttpVersionNotSupported,
      ///Indicates an unknown status code
      Unknown,
    }
    impl TryFrom<wick_component::serde_util::enum_repr::StringOrNum> for StatusCode {
      type Error = String;
      fn try_from(value: wick_component::serde_util::enum_repr::StringOrNum) -> std::result::Result<Self, String> {
        use std::str::FromStr;
        match value {
          wick_component::serde_util::enum_repr::StringOrNum::String(v) => Self::from_str(&v),
          wick_component::serde_util::enum_repr::StringOrNum::Int(v) => Self::from_str(&v.to_string()),
          wick_component::serde_util::enum_repr::StringOrNum::Float(v) => Self::from_str(&v.to_string()),
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
      ///Returns the value of the enum variant as a string.
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
///Types associated with the `echo` operation
pub mod echo {
  #[allow(unused)]
  use super::*;
  #[derive(Debug, Clone, Default, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
  #[allow(clippy::exhaustive_structs)]
  pub struct Config {}
  pub struct Outputs {
    #[allow(unused)]
    pub(crate) output: wick_packet::OutgoingPort<types::http::HttpRequest>,
    pub(crate) time: wick_packet::OutgoingPort<wick_component::datetime::DateTime>,
  }
  impl wick_component::Broadcast for Outputs {
    fn outputs_mut(&mut self) -> wick_packet::OutputIterator<'_> {
      wick_packet::OutputIterator::new(vec![&mut self.output, &mut self.time])
    }
  }
  impl Outputs {
    pub fn new(channel: wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError>) -> Self {
      Self {
        output: wick_packet::OutgoingPort::new("output", channel.clone()),
        time: wick_packet::OutgoingPort::new("time", channel),
      }
    }
  }
  #[async_trait::async_trait(?Send)]
  #[cfg(target_family = "wasm")]
  pub trait Operation {
    type Error;
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
  pub trait Operation {
    type Error: Send;
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
}
///Types associated with the `testop` operation
pub mod testop {
  #[allow(unused)]
  use super::*;
  #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, PartialEq)]
  #[allow(clippy::exhaustive_structs)]
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
    pub(crate) output: wick_packet::OutgoingPort<String>,
  }
  impl wick_component::Broadcast for Outputs {
    fn outputs_mut(&mut self) -> wick_packet::OutputIterator<'_> {
      wick_packet::OutputIterator::new(vec![&mut self.output])
    }
  }
  impl wick_component::SingleOutput for Outputs {
    fn single_output(&mut self) -> &mut dyn wick_packet::Port {
      &mut self.output
    }
  }
  impl Outputs {
    pub fn new(channel: wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError>) -> Self {
      Self {
        output: wick_packet::OutgoingPort::new("output", channel),
      }
    }
  }
  #[async_trait::async_trait(?Send)]
  #[cfg(target_family = "wasm")]
  pub trait Operation {
    type Error;
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
  pub trait Operation {
    type Error: Send;
    type Outputs: Send;
    type Config: std::fmt::Debug + Send;
    #[allow(unused)]
    async fn testop(
      message: WickStream<types::http::HttpResponse>,
      outputs: Self::Outputs,
      ctx: wick_component::flow_component::Context<Self::Config>,
    ) -> std::result::Result<(), Self::Error>;
  }
}
#[derive(Default, Clone)]
///The struct that the component implementation hinges around
pub struct Component;
impl Component {
  fn echo_wrapper(
    mut input: wasmrs_rx::BoxFlux<wasmrs::Payload, wasmrs::PayloadError>,
  ) -> std::result::Result<wasmrs_rx::BoxFlux<wasmrs::RawPayload, wasmrs::PayloadError>, wick_component::BoxError> {
    let (channel, rx) = wasmrs_rx::FluxChannel::<wasmrs::RawPayload, wasmrs::PayloadError>::new_parts();
    let outputs = echo::Outputs::new(channel.clone());
    runtime::spawn("echo_wrapper", async move {
      #[allow(unused_parens)]
      let (config, (input, time)) = wick_component::payload_fan_out!(
          input, raw : false, wick_component::AnyError, echo::Config,
          [("input", types::http::HttpRequest), ("time",
          wick_component::datetime::DateTime)]
      );
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
      use echo::Operation;
      if let Err(e) = Component::echo(Box::pin(input), Box::pin(time), outputs, config).await {
        let _ = channel.send_result(wick_packet::Packet::component_error(e.to_string()).into());
      }
    });
    Ok(Box::pin(rx))
  }
  fn testop_wrapper(
    mut input: wasmrs_rx::BoxFlux<wasmrs::Payload, wasmrs::PayloadError>,
  ) -> std::result::Result<wasmrs_rx::BoxFlux<wasmrs::RawPayload, wasmrs::PayloadError>, wick_component::BoxError> {
    let (channel, rx) = wasmrs_rx::FluxChannel::<wasmrs::RawPayload, wasmrs::PayloadError>::new_parts();
    let outputs = testop::Outputs::new(channel.clone());
    runtime::spawn("testop_wrapper", async move {
      #[allow(unused_parens)]
      let (config, (message)) = wick_component::payload_fan_out!(
          input, raw : false, wick_component::AnyError, testop::Config,
          [("message", types::http::HttpResponse)]
      );
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
      use testop::Operation;
      if let Err(e) = Component::testop(Box::pin(message), outputs, config).await {
        let _ = channel.send_result(wick_packet::Packet::component_error(e.to_string()).into());
      }
    });
    Ok(Box::pin(rx))
  }
}
