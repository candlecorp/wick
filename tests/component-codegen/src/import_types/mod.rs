#[allow(unused)]
use guest::*;
use wasmrs_guest as guest;
#[allow(unused)]
pub(crate) type WickStream<T> = BoxFlux<T, wick_component::anyhow::Error>;
pub use wick_component::anyhow::Result;
#[no_mangle]
#[cfg(target_family = "wasm")]
extern "C" fn __wasmrs_init(guest_buffer_size: u32, host_buffer_size: u32, max_host_frame_len: u32) {
  guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);
  guest::register_request_response("wick", "__setup", Box::new(__setup));
  guest::register_request_channel("wick", "testop", Box::new(Component::testop_wrapper));
}
#[cfg(target_family = "wasm")]
thread_local! { static __CONFIG : std :: cell :: UnsafeCell < Option < SetupPayload >> = std :: cell :: UnsafeCell :: new (None) ; }
#[cfg(target_family = "wasm")]
#[derive(Debug, serde :: Deserialize)]
pub(crate) struct SetupPayload {
  #[allow(unused)]
  pub(crate) provided: std::collections::HashMap<String, wick_component::packet::ComponentReference>,
}
#[cfg(target_family = "wasm")]
fn __setup(input: BoxMono<Payload, PayloadError>) -> Result<BoxMono<RawPayload, PayloadError>, GenericError> {
  Ok(
    Mono::from_future(async move {
      match input.await {
        Ok(payload) => {
          let input = wasmrs_guest::deserialize::<SetupPayload>(&payload.data).unwrap();
          __CONFIG.with(|cell| {
            #[allow(unsafe_code)]
            unsafe { &mut *cell.get() }.replace(input);
          });
          Ok(RawPayload::new_data(None, None))
        }
        Err(e) => {
          return Err(e);
        }
      }
    })
    .boxed(),
  )
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
  use super::*;
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
    use super::*;
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
      Ok,
      Created,
    }
    impl StatusCode {
      #[allow(unused)]
      pub fn value(&self) -> Option<&'static str> {
        match self {
          Self::Ok => Some("200"),
          Self::Created => Some("201"),
        }
      }
    }
    impl std::str::FromStr for StatusCode {
      type Err = String;
      fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
          "OK" => Ok(Self::Ok),
          "Created" => Ok(Self::Created),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for StatusCode {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    use super::*;
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
      Ok,
      Created,
    }
    impl StatusCode {
      #[allow(unused)]
      pub fn value(&self) -> Option<&'static str> {
        match self {
          Self::Ok => Some("200"),
          Self::Created => Some("201"),
        }
      }
    }
    impl std::str::FromStr for StatusCode {
      type Err = String;
      fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
          "OK" => Ok(Self::Ok),
          "Created" => Ok(Self::Created),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for StatusCode {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    use super::*;
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
      Ok,
      Created,
    }
    impl StatusCode {
      #[allow(unused)]
      pub fn value(&self) -> Option<&'static str> {
        match self {
          Self::Ok => Some("200"),
          Self::Created => Some("201"),
        }
      }
    }
    impl std::str::FromStr for StatusCode {
      type Err = String;
      fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
          "OK" => Ok(Self::Ok),
          "Created" => Ok(Self::Created),
          _ => Err(s.to_owned()),
        }
      }
    }
    impl std::fmt::Display for StatusCode {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
pub struct OpTestopOutputs {
  #[allow(unused)]
  pub(crate) output: wick_component::packet::Output<String>,
}
impl OpTestopOutputs {
  pub fn new(channel: FluxChannel<RawPayload, PayloadError>) -> Self {
    Self {
      output: wick_component::packet::Output::new("output", channel.clone()),
    }
  }
}
# [cfg_attr (target_family = "wasm" , async_trait :: async_trait (? Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
pub trait OpTestop {
  #[allow(unused)]
  async fn testop(message: WickStream<types::http::HttpResponse>, outputs: OpTestopOutputs) -> Result<()> {
    unimplemented!()
  }
}
#[derive(Default, Clone)]
pub struct Component;
impl Component {
  fn testop_wrapper(
    mut input: BoxFlux<Payload, PayloadError>,
  ) -> std::result::Result<BoxFlux<RawPayload, PayloadError>, Box<dyn std::error::Error + Send + Sync>> {
    let (channel, rx) = FluxChannel::<RawPayload, PayloadError>::new_parts();
    let outputs = OpTestopOutputs::new(channel.clone());
    spawn(async move {
      let message =
        wick_component :: payload_fan_out ! (input , raw : false , [("message" , types :: http :: HttpResponse) ,]);
      if let Err(e) = Component::testop(Box::pin(message), outputs).await {
        let _ = channel.send_result(wick_component::packet::Packet::component_error(e.to_string()).into());
      }
    });
    Ok(Box::pin(rx))
  }
}
