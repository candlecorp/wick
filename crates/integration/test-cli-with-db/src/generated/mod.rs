#[allow(unused)]
use guest::*;
use wasmrs_guest as guest;
#[allow(unused)]
pub(crate) type WickStream<T> = FluxReceiver<T, wick_component::anyhow::Error>;
pub use wick_component::anyhow::Result;
#[no_mangle]
extern "C" fn __wasmrs_init(guest_buffer_size: u32, host_buffer_size: u32, max_host_frame_len: u32) {
  guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);
  guest::register_request_channel("wick", "main", Component::main_wrapper);
}
#[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize, PartialEq)]
pub struct Interactive {
  pub stdin: bool,
  pub stdout: bool,
  pub stderr: bool,
}
pub struct OpMainOutputs {
  #[allow(unused)]
  pub(crate) code: wick_component::packet::Output<u32>,
}
impl OpMainOutputs {
  pub fn new(channel: FluxChannel<RawPayload, PayloadError>) -> Self {
    Self {
      code: wick_component::packet::Output::new("code", channel.clone()),
    }
  }
}
# [cfg_attr (target_family = "wasm" , async_trait :: async_trait (? Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
pub trait OpMain {
  #[allow(unused)]
  async fn main(
    args: WickStream<Vec<String>>,
    is_interactive: WickStream<Interactive>,
    outputs: OpMainOutputs,
  ) -> Result<()> {
    unimplemented!()
  }
}
#[derive(Default, Clone)]
pub struct Component;
impl Component {
  fn main_wrapper(
    mut input: FluxReceiver<Payload, PayloadError>,
  ) -> std::result::Result<FluxReceiver<RawPayload, PayloadError>, Box<dyn std::error::Error + Send + Sync>> {
    let (channel, rx) = FluxChannel::<RawPayload, PayloadError>::new_parts();
    let outputs = OpMainOutputs::new(channel.clone());
    spawn(async move {
      let (args, is_interactive) = wick_component :: payload_fan_out ! (input , raw : false , [("args" , Vec < String >) , ("isInteractive" , Interactive) ,]);
      if let Err(e) = Component::main(args, is_interactive, outputs).await {
        let _ = channel.send_result(wick_component::packet::Packet::component_error(e.to_string()).into());
      }
    });
    Ok(rx)
  }
}
