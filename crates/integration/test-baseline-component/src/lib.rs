use std::fmt::Display;

use guest::*;
use wasmrs_guest as guest;
use wick_wasmrs_macros::{payload_fan_out, wasmflow_packet_stream::Packet};

#[no_mangle]
extern "C" fn __wasmrs_init(guest_buffer_size: u32, host_buffer_size: u32, max_host_frame_len: u32) {
  guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);

  guest::register_request_channel("wick", "add", add);
  guest::register_request_channel("wick", "validate", validate);
  guest::register_request_channel("wick", "error", error);
}

fn add(
  mut input: FluxReceiver<ParsedPayload, PayloadError>,
) -> Result<FluxReceiver<Payload, PayloadError>, GenericError> {
  let (channel, rx) = FluxChannel::<Payload, PayloadError>::new_parts();

  spawn(async move {
    let (mut left, mut right) = payload_fan_out!(input, "left", "right");
    while let (Some(Ok(left)), Some(Ok(right))) = (left.next().await, right.next().await) {
      let left: u64 = left.deserialize().unwrap();
      let right: u64 = right.deserialize().unwrap();
      if let Err(e) = channel.send_result(Packet::encode("output", left + right).into()) {
        println!("{}", e);
      }
    }
    let _ = channel.send_result(Packet::done("output").into());
  });

  Ok(rx)
}

#[derive(Debug)]
enum LengthError {
  TooShort,
  TooLong,
}

impl Display for LengthError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        LengthError::TooShort => format!("Needs to be longer than {} characters", MINIMUM_LENGTH),
        LengthError::TooLong => format!("Needs to be shorter than {} characters", MAXIMUM_LENGTH),
      }
    )
  }
}

impl std::error::Error for LengthError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    Some(self)
  }
}

static MINIMUM_LENGTH: usize = 8;
static MAXIMUM_LENGTH: usize = 512;

fn validate(
  mut input: FluxReceiver<ParsedPayload, PayloadError>,
) -> Result<FluxReceiver<Payload, PayloadError>, GenericError> {
  let (channel, rx) = FluxChannel::<Payload, PayloadError>::new_parts();

  spawn(async move {
    let (mut input) = payload_fan_out!(input, "input");
    while let (Some(Ok(input))) = (input.next().await) {
      let password: String = input.deserialize().unwrap();
      if password.len() < MINIMUM_LENGTH {
        let _ = channel.send_result(Packet::err("output", LengthError::TooShort.to_string()).into());
      } else if password.len() > MAXIMUM_LENGTH {
        let _ = channel.send_result(Packet::err("output", LengthError::TooLong.to_string()).into());
      } else {
        let _ = channel.send_result(Packet::encode("output", password).into());
      }
    }
    let _ = channel.send_result(Packet::done("output").into());
  });

  Ok(rx)
}

fn error(
  mut input: FluxReceiver<ParsedPayload, PayloadError>,
) -> Result<FluxReceiver<Payload, PayloadError>, GenericError> {
  let (channel, rx) = FluxChannel::<Payload, PayloadError>::new_parts();

  spawn(async move {
    let (mut input) = payload_fan_out!(input, "input");
    while let (Some(Ok(input))) = (input.next().await) {
      panic!("This component always panics");
    }
    let _ = channel.send_result(Packet::done("output").into());
  });

  Ok(rx)
}
