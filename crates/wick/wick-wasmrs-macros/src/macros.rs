#[macro_export]
macro_rules! payloads {
    ($(($port:expr, $value:expr)),*) => {
      {
        let mut msgs = std::vec::Vec::new();
        let mut ports = std::collections::HashSet::new();
        $(
          ports.insert($port.to_owned());
          let md = wasmrs::Metadata::new_extra(0, $crate::WickMetadata::new($port));
          msgs.push(wasmrs::Payload::new_data(Some(md), Some(serialize(&output).unwrap().into())));
        )*
        for port in ports {
          let md = wasmrs::Metadata::new_extra(0, $crate::WickMetadata::new_done($port));
          msgs.push(wasmrs::Payload::new_data(Some(md), None));
        }
        msgs
      }
    };
}

#[macro_export]
macro_rules! payload_stream {
  ($(($port:expr, $value:expr)),*) => {{
    let packets = $crate::packets!($(($port, $value)),*);
    let (tx,rx) = wasmrs::FluxChannel::new_partss();
    for p in packets {
      tx.send(p).unwrap();
    }
    rx
  }};
}

#[macro_export]
macro_rules! payload_fan_out {
    ($stream:expr, $($port:expr),*) => {
      {
        let mut streams = $crate::wick_packet::StreamMap::default();
        let mut senders = std::collections::HashMap::new();
        $(
          senders.insert($port, streams.init($port));
        )*
        $crate::wasmrs_runtime::spawn(async move {
          while let Some(Ok(mut payload)) = $stream.next().await {
            let packet: Packet = payload.into();
            let sender = senders.get_mut(packet.extra.stream()).unwrap();
            if packet.extra.is_done() {
              sender.complete();
              continue;
            }
            sender.send(packet).unwrap();
          }
        });
        ($(streams.take($port).unwrap()),*)
        }
    };
}
