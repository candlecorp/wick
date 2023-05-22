#[macro_export]
macro_rules! packet_stream {
  ($(($port:expr, $value:expr)),*) => {{
    let packets = $crate::packets!($(($port, $value)),*);
    wick_packet::PacketStream::new(Box::new(futures::stream::iter(packets.into_iter().map(Ok))))
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
          msgs.push($crate::Packet::encode($port, $value));
        )*
        for port in ports {
          msgs.push($crate::Packet::done(&port));
        }
        msgs
      }
    };
}

#[macro_export]
macro_rules! fan_out {
  ($stream:expr, $($port:expr),*) => {
  {
        use $crate::wasmrs_rx::Observer;
        let mut streams = wick_packet::StreamMap::default();
        let mut senders = std::collections::HashMap::new();
        $(
          senders.insert($port, streams.init($port));
        )*
        tokio::spawn(async move {
            while let Some(Ok(payload)) = $stream.next().await {
            let sender = senders.get_mut(payload.port()).unwrap();
            if payload.is_done() {
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
