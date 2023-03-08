use std::fmt::Display;
use std::io::{self, BufRead, BufReader};

use guest::*;
use wasmrs_guest as guest;
use wick_wasmrs_macros::payload_fan_out;
use wick_wasmrs_macros::wick_packet::{packet_stream, CollectionLink, Packet, PacketStream};

#[no_mangle]
extern "C" fn __wasmrs_init(guest_buffer_size: u32, host_buffer_size: u32, max_host_frame_len: u32) {
  guest::init(guest_buffer_size, host_buffer_size, max_host_frame_len);

  guest::register_request_channel("wick", "byteCount", byteCount);
  guest::register_request_channel("wick", "main", main);
  guest::add_import(0, OperationType::RequestChannel, "wick", "link_call");
}

fn byteCount(
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
#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub struct Interactive {
  #[serde(rename = "stdin")]
  pub stdin: bool,
  #[serde(rename = "stdout")]
  pub stdout: bool,
  #[serde(rename = "stderr")]
  pub stderr: bool,
}

fn main(
  mut input: FluxReceiver<ParsedPayload, PayloadError>,
) -> Result<FluxReceiver<Payload, PayloadError>, GenericError> {
  let (channel, rx) = FluxChannel::<Payload, PayloadError>::new_parts();

  spawn(async move {
    let (mut input, mut is_interactive, mut program) = payload_fan_out!(input, "args", "isInteractive", "program");
    while let (Some(Ok(input)), Some(Ok(is_interactive)), Some(Ok(program))) =
      (input.next().await, is_interactive.next().await, program.next().await)
    {
      let args: Vec<String> = input.deserialize().unwrap();
      let tty: Interactive = is_interactive.deserialize().unwrap();
      let app: CollectionLink = program.deserialize().unwrap();
      // let stream = app.call("hello", PacketStream::default()).await.unwrap();

      println!(
        "args: {:?}, interactive: {{ stdin: {}, stdout: {}, stderr: {} }}",
        args, tty.stdin, tty.stdout, tty.stderr
      );

      let isatty = tty.stdin;
      if !isatty {
        let reader = BufReader::new(io::stdin());
        let input = reader.lines().collect::<Result<Vec<String>, _>>().unwrap().join("\n");
        println!("{}", input);
      }
    }
    let _ = channel.send_result(Packet::encode("code", 0).into());
    let _ = channel.send_result(Packet::done("code").into());
  });

  Ok(rx)
}
