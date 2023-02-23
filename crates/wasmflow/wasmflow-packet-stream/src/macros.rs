#[macro_export]
macro_rules! packet_stream {
  ($(($port:expr, $value:expr)),*) => {{
    let packets = $crate::packets!($(($port, $value)),*);
    wasmflow_packet_stream::PacketStream::new(Box::new(futures::stream::iter(packets.into_iter().map(Ok))))
  }};
}

#[macro_export]
macro_rules! packets {
    ($(($port:expr, $value:expr)),*) => {
      {
        let mut msgs = std::vec::Vec::new();
        let mut ports = std::collections::HashSet::new();
        $(
          ports.insert($port.to_owned());
          msgs.push(wasmflow_packet_stream::Packet::encode($port, $value));
        )*
        for port in ports {
          msgs.push(wasmflow_packet_stream::Packet::done(&port));
        }
        msgs
      }
    };
}

#[macro_export]
macro_rules! fan_out {
    ($stream:expr, $($port:expr),*) => {
      {
        let mut streams = wasmflow_packet_stream::StreamMap::default();
        let mut senders = std::collections::HashMap::new();
        $(
          senders.insert($port, streams.init($port));
        )*
        tokio::spawn(async move {
          while let Some(Ok(payload)) = $stream.next().await {
            let sender = senders.get_mut(payload.port_name()).unwrap();
            if matches!(payload.payload, wasmflow_packet_stream::PacketPayload::Done) {
              sender.complete();
              continue;
            }
            sender.send(payload).unwrap();
          }
        });
        ($(streams.take($port).unwrap()),*)
        }
    };
}
