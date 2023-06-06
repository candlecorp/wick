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
      let packet: Result<$ty, _> = $packet.decode().map_err(|e| e.into());
      let _ = $tx.send_result(packet);
    }
  }};
}

#[macro_export]
macro_rules! payload_fan_out {
    ($stream:expr, raw:$raw:tt, $error:ty, [ $(($port:expr, $($ty:tt)+)),* $(,)? ]) => {
      {
        $crate::paste::paste! {
          $(
            #[allow(unused_parens)]
            let ([<$port:snake _tx>],[<$port:snake _rx>]) = $crate::wasmrs_rx::FluxChannel::<_,$error>::new_parts();
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
                    [<$port:snake _tx>].send_result(Err(Box::new($crate::flow_component::ComponentError::message(error.msg())).into())).unwrap();
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
    ($stream:expr, raw:$raw:tt, $error:ty, $config:ty, [ $(($port:expr, $($ty:tt)+)),* $(,)? ]) => {
      {
          $crate::paste::paste! {
            $(
              #[allow(unused_parens)]
              let ([<$port:snake _tx>],[<$port:snake _rx>]) = $crate::wasmrs_rx::FluxChannel::<_,$error>::new_parts();
            )*
          }
          let (config_tx,config_rx) = $crate::runtime::oneshot();

        $crate::runtime::spawn("payload_fan_out",async move {
          #[allow(unused)]
          use $crate::StreamExt;
          let mut config_tx = Some(config_tx);
          loop {
            if let Some(Ok(payload)) = $stream.next().await {
              let mut packet: $crate::packet::Packet = payload.into();
              if let Some(config_tx) = config_tx.take() {
                if let Some(context) = packet.context() {
                  let config: Result<$crate::packet::ContextTransport<$config>, _> = $crate::wasmrs_codec::messagepack::deserialize(&context).map_err(|_e|$crate::flow_component::ComponentError::message("Cound not deserialize Context"));
                  let _ = config_tx.send(config.map($crate::flow_component::Context::from));
                } else {
                  packet = $crate::packet::Packet::component_error("No context attached to first invocation packet");
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
                      [<$port:snake _tx>].send_result(Err(Box::new($crate::flow_component::ComponentError::message(error.msg())).into())).unwrap();
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

#[macro_export]
macro_rules! propagate_if_error {
  ($result:expr, $outputs:ident, continue) => {
    match $result {
      Ok(value) => value,
      Err(err) => {
        $outputs.broadcast_err(err.to_string());
        continue;
      }
    }
  };
  ($result:expr,$outputs:ident, break) => {
    match $result {
      Ok(value) => value,
      Err(err) => {
        $outputs.broadcast_err(err.to_string());
        continue;
      }
    }
  };
  ($result:expr,$outputs:ident, return $rv:expr) => {
    match $result {
      Ok(value) => value,
      Err(err) => {
        $outputs.broadcast_err(err.to_string());
        return $rv;
      }
    }
  };
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use tokio_stream::StreamExt;
  use wick_packet::{packet_stream, InherentData};
  #[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
  struct Config {}

  #[tokio::test]
  async fn test_basic() -> Result<()> {
    let mut stream = packet_stream!(("foo", 1), ("bar", 2), ("foo", 3), ("bar", 4), ("foo", 5), ("bar", 6));
    stream.set_context(Default::default(), InherentData::unsafe_default());
    let (_config, mut foo_rx, mut bar_rx) =
      payload_fan_out!(stream, raw: false, anyhow::Error, Config, [("foo", i32), ("bar", i32)]);
    assert_eq!(foo_rx.next().await.unwrap().unwrap(), 1);
    assert_eq!(bar_rx.next().await.unwrap().unwrap(), 2);
    assert_eq!(foo_rx.next().await.unwrap().unwrap(), 3);
    assert_eq!(bar_rx.next().await.unwrap().unwrap(), 4);
    assert_eq!(foo_rx.next().await.unwrap().unwrap(), 5);
    assert_eq!(bar_rx.next().await.unwrap().unwrap(), 6);

    Ok(())
  }
}
