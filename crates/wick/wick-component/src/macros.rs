#[macro_export]
macro_rules! wick_import {
  () => {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
  };
}

#[macro_export]
macro_rules! operation {
    () => {
      #[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
      #[cfg_attr(not(target_family = "wasm"), async_trait::async_trait(Send))]
    };
}

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
    ($stream:expr, [ $(($port:expr, $ty:ty)),* $(,)? ]) => {
      {
          $crate::paste::paste! {
            $(
              #[allow(unused_parens)]
              let ([<$port:snake _tx>],[<$port:snake _rx>]) = FluxChannel::new_parts();
            )*
          }
        $crate::runtime::spawn(async move {
          while let Some(Ok(mut payload)) = $stream.next().await {
            let packet: Packet = payload.into();
            match packet.extra.stream() {
              $(
                $port=>if packet.extra.is_done() {
                  $crate::paste::paste! {[<$port:snake _tx>].complete();}
                } else {
                  let packet: Result<$ty,_> = packet.deserialize().map_err(|e|e.into());
                  $crate::paste::paste! {let _ = [<$port:snake _tx>].send_result(packet);}
                },
              )*
              _ => panic!("Unexpected port: {}", packet.extra.stream())
            }
          }
        });
        $crate::paste::paste! {($([<$port:snake _rx>]),*)}
        }
    };
}
