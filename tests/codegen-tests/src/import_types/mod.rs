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
  wasmrs_guest::register_request_channel("wick", "testop", Box::new(Component::testop_wrapper));
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
  #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
  pub struct LocalStruct {
    pub field1: String,
    pub inner: types::LocalStructInner,
  }
  #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
  pub struct LocalStructInner {
    pub field1: String,
    pub field2: String,
  }
  pub mod ZZZ {
    #[allow(unused)]
    use super::ZZZ;
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
      Ok,
      Created,
    }
    impl StatusCode {
      #[allow(unused)]
      pub fn value(&self) -> Option<&'static str> {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Ok => Some("200"),
          Self::Created => Some("201"),
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
          "200" => Ok(Self::Ok),
          "201" => Ok(Self::Created),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for StatusCode {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Ok => f.write_str("OK"),
          Self::Created => f.write_str("Created"),
        }
      }
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    pub struct HttpResponse {
      pub version: HttpVersion,
      pub status: StatusCode,
      pub headers: std::collections::HashMap<String, Vec<String>>,
      pub body: bytes::Bytes,
    }
  }
  pub mod http {
    #[allow(unused)]
    use super::http;
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
      Ok,
      Created,
    }
    impl StatusCode {
      #[allow(unused)]
      pub fn value(&self) -> Option<&'static str> {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Ok => Some("200"),
          Self::Created => Some("201"),
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
          "200" => Ok(Self::Ok),
          "201" => Ok(Self::Created),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for StatusCode {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Ok => f.write_str("OK"),
          Self::Created => f.write_str("Created"),
        }
      }
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    pub struct HttpResponse {
      pub version: HttpVersion,
      pub status: StatusCode,
      pub headers: std::collections::HashMap<String, Vec<String>>,
      pub body: bytes::Bytes,
    }
  }
  pub mod AAA {
    #[allow(unused)]
    use super::AAA;
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
      Ok,
      Created,
    }
    impl StatusCode {
      #[allow(unused)]
      pub fn value(&self) -> Option<&'static str> {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Ok => Some("200"),
          Self::Created => Some("201"),
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
          "200" => Ok(Self::Ok),
          "201" => Ok(Self::Created),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for StatusCode {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
          Self::Ok => f.write_str("OK"),
          Self::Created => f.write_str("Created"),
        }
      }
    }
    #[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
    pub struct HttpResponse {
      pub version: HttpVersion,
      pub status: StatusCode,
      pub headers: std::collections::HashMap<String, Vec<String>>,
      pub body: bytes::Bytes,
    }
  }
}
#[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
pub struct OpTestopConfig {
  pub a: String,
  pub b: u32,
}
impl Default for OpTestopConfig {
  fn default() -> Self {
    Self {
      a: Default::default(),
      b: Default::default(),
    }
  }
}
pub struct OpTestopOutputs {
  #[allow(unused)]
  pub(crate) output: wick_packet::Output<String>,
}
impl OpTestopOutputs {
  pub fn new(channel: wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError>) -> Self {
    Self {
      output: wick_packet::Output::new("output", channel.clone()),
    }
  }
}
# [cfg_attr (target_family = "wasm" , async_trait :: async_trait (? Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
pub trait OpTestop {
  #[allow(unused)]
  async fn testop(
    message: WickStream<types::http::HttpResponse>,
    outputs: OpTestopOutputs,
    ctx: wick_component::flow_component::Context<OpTestopConfig>,
  ) -> Result<()> {
    unimplemented!()
  }
}
#[derive(Default, Clone)]
pub struct Component;
impl Component {
  fn testop_wrapper(
    mut input: wasmrs_rx::BoxFlux<wasmrs::Payload, wasmrs::PayloadError>,
  ) -> std::result::Result<
    wasmrs_rx::BoxFlux<wasmrs::RawPayload, wasmrs::PayloadError>,
    Box<dyn std::error::Error + Send + Sync>,
  > {
    let (channel, rx) = wasmrs_rx::FluxChannel::<wasmrs::RawPayload, wasmrs::PayloadError>::new_parts();
    let outputs = OpTestopOutputs::new(channel.clone());
    runtime::spawn("testop_wrapper", async move {
      let (config, message) = wick_component :: payload_fan_out ! (input , raw : false , OpTestopConfig , [("message" , types :: http :: HttpResponse) ,]);
      let config = match config.await {
        Ok(Ok(config)) => config,
        _ => {
          let _ = channel.send_result(wick_packet::Packet::component_error("Component sent invalid context").into());
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
