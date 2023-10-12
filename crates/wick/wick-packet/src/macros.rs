#[macro_export]
macro_rules! raw_packet_stream {
  ($(($port:expr, $value:expr)),*) => {{
    let packets = $crate::packets!($(($port, $value)),*);
    let packets :$crate::PacketStream = packets.into();
    $crate::packetstream_to_wasmrs(0,packets)
  }};
}

#[macro_export]
macro_rules! packet_stream {
  ($(($port:expr, $value:expr)),*) => {{
    let packets = $crate::packets!($(($port, $value)),*);
    let packets :$crate::PacketStream = packets.into();
    packets
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
            use $crate::PacketExt;
            while let Some(Ok(payload)) = $stream.next().await {
            let sender = senders.get_mut(payload.port()).unwrap();
            if payload.is_done() {
              sender.send(payload).unwrap();
              sender.complete();
            } else {
              sender.send(payload).unwrap();
            }
          }
        });
        ($(streams.take($port).unwrap()),*)
        }
    };
}
