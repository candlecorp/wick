#[macro_export]
macro_rules! wick_import {
  () => {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
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
    use $crate::wasmrs_rx::Observer;

    let packets = $crate::packet::packets!($(($port, $value)),*);
    let (tx,rx) = $crate::wasmrs_rx::FluxChannel::new_parts();
    for p in packets {
      tx.send(p).unwrap();
    }
    rx
  }};
}

#[macro_export]
macro_rules! handle_port {
  (raw: true, $packet:ident, $tx:ident, $port:expr, $ty:ty) => {{
    use $crate::wasmrs_rx::Observer;
    if $packet.is_done() {
      $tx.complete();
    } else {
      // let packet: Result<$ty, _> = $packet.deserialize().map_err(|e| e.into());
      let _ = $tx.send($packet);
    }
  }};
  (raw: false, $packet:ident, $tx:ident, $port:expr, $ty:ty) => {{
    use $crate::wasmrs_rx::Observer;
    if $packet.is_done() {
      $tx.complete();
    } else {
      let packet: Result<$ty, _> = $packet.deserialize().map_err(|e| e.into());
      let _ = $tx.send_result(packet);
    }
  }};
}

#[macro_export]
macro_rules! payload_fan_out {
    ($stream:expr, raw:$raw:tt, [ $(($port:expr, $($ty:tt)+)),* $(,)? ]) => {
      {
        $crate::paste::paste! {
          $(
            #[allow(unused_parens)]
            let ([<$port:snake _tx>],[<$port:snake _rx>]) = $crate::wasmrs_rx::FluxChannel::<_,$crate::anyhow::Error>::new_parts();
          )*
        }
      $crate::runtime::spawn("payload_fan_out", async move {
        use $crate::StreamExt;
        loop {
          if let Some(Ok(payload)) = $stream.next().await {
            let packet: $crate::packet::Packet = payload.into();
            match packet.port() {
              $(
                $port=> {
                  let tx = &$crate::paste::paste! {[<$port:snake _tx>]};
                  $crate::handle_port!(raw: $raw, packet, tx, $port, $($ty)*)
                },
              )*
              $crate::packet::Packet::FATAL_ERROR => {
                let error = packet.unwrap_err();
                $crate::paste::paste! {
                  $(
                    [<$port:snake _tx>].send_result(Err($crate::anyhow::Error::msg(error.msg().to_owned()))).unwrap();
                  )*
                }
              }
              _ => panic!("Unexpected port: {}", packet.port())
            }
          } else {
            break;
          }
        }

      });
      $crate::paste::paste! {($(Box::pin([<$port:snake _rx>])),*)}
      }

    };
    ($stream:expr, raw:$raw:tt, $config:ty, [ $(($port:expr, $($ty:tt)+)),* $(,)? ]) => {
      {
          $crate::paste::paste! {
            $(
              #[allow(unused_parens)]
              let ([<$port:snake _tx>],[<$port:snake _rx>]) = $crate::wasmrs_rx::FluxChannel::<_,$crate::anyhow::Error>::new_parts();
            )*
          }
          let (config_tx,config_rx) = $crate::runtime::oneshot();

        $crate::runtime::spawn("payload_fan_out",async move {
          use $crate::StreamExt;
          let mut config_tx = Some(config_tx);
          loop {
            if let Some(Ok( payload)) = $stream.next().await {
              let packet: $crate::packet::Packet = payload.into();
              if let Some(config_tx) = config_tx.take() {
                if let Some(context) = packet.context() {
                  let config: Result<$crate::packet::ContextTransport<$config>, _> = $crate::wasmrs_codec::messagepack::deserialize(&context).map_err(|_e|$crate::flow_component::ComponentError::message("Cound not deserialize Context"));
                  let _ = config_tx.send(config.map($crate::flow_component::Context::from));
                } else {
                  let _ = config_tx.send(Ok($crate::packet::ContextTransport::new(<$config>::default(),None).into()));
                }
              }

              match packet.port() {
                $(
                  $port=> {
                    let tx = &$crate::paste::paste! {[<$port:snake _tx>]};
                    $crate::handle_port!(raw: $raw, packet, tx, $port, $($ty)*)
                  },
                )*
                $crate::packet::Packet::FATAL_ERROR => {
                  use $crate::wasmrs_rx::Observer;
                  let error = packet.unwrap_err();
                  $crate::paste::paste! {
                    $(
                      [<$port:snake _tx>].send_result(Err($crate::anyhow::Error::msg(error.msg().to_owned()))).unwrap();
                    )*
                  }
                }
                _ => panic!("Unexpected port: {}", packet.port())
              }
            } else {
              break;
            }
          }
        });
        let config_mono = Box::pin(config_rx);
        $crate::paste::paste! {(config_mono, $(Box::pin([<$port:snake _rx>])),*)}
        }
    };
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use futures::StreamExt;
  use wasmrs::PayloadError;
  use wasmrs_rx::FluxReceiver;
  use wick_packet::Packet;
  #[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
  struct Config {}

  #[tokio::test]
  async fn test_basic() -> Result<()> {
    let mut stream: FluxReceiver<Packet, PayloadError> =
      payload_stream!(("foo", 1), ("bar", 2), ("foo", 3), ("bar", 4), ("foo", 5), ("bar", 6));
    let (_config, mut foo_rx, mut bar_rx) = payload_fan_out!(stream, raw: false, Config, [("foo", i32), ("bar", i32)]);
    assert_eq!(foo_rx.next().await.unwrap().unwrap(), 1);
    assert_eq!(bar_rx.next().await.unwrap().unwrap(), 2);
    assert_eq!(foo_rx.next().await.unwrap().unwrap(), 3);
    assert_eq!(bar_rx.next().await.unwrap().unwrap(), 4);
    assert_eq!(foo_rx.next().await.unwrap().unwrap(), 5);
    assert_eq!(bar_rx.next().await.unwrap().unwrap(), 6);

    Ok(())
  }
}
