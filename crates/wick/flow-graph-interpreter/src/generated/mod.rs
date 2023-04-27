#[cfg(not(target_family = "wasm"))]
pub use serde_json::Value;
#[cfg(target_family = "wasm")]
pub use wasmrs_guest::Value;
pub(crate) use wick_component::wasmrs_rx::{Observable, Observer};
pub use wick_component::{packet as wick_packet, runtime, wasmrs, wasmrs_codec, wasmrs_rx};
#[allow(unused)]
pub(crate) type WickStream<T> = wick_component::wasmrs_rx::BoxFlux<T, wick_component::anyhow::Error>;
pub use wick_component::anyhow::Result;
#[no_mangle]
#[cfg(target_family = "wasm")]
extern "C" fn __wasmrs_init(guest_buffer_size: u32, host_buffer_size: u32, max_host_frame_len: u32) {
  wasmrs_guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);
  wasmrs_guest::register_request_response("wick", "__setup", Box::new(__setup));
  wasmrs_guest::register_request_channel("wick", "pluck", Box::new(Component::pluck_wrapper));
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
  Ok(
    wasmrs_rx::Mono::from_future(async move {
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
  #[allow(unused)]
  use super::types;
}
pub struct OpPluckOutputs {
  #[allow(unused)]
  pub(crate) output: wick_packet::Output<Value>,
}
impl OpPluckOutputs {
  pub fn new(channel: wasmrs_rx::FluxChannel<wasmrs::RawPayload, wasmrs::PayloadError>) -> Self {
    Self {
      output: wick_packet::Output::new("output", channel.clone()),
    }
  }
}
