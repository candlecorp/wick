/// This macro is used to include the generated code from a `wick-component-codegen` build step.
///
/// # Example
///
/// ```
/// # use wick_component::prelude::*;
///
/// // Useful way of importing code and keeping it separate from your own code.
/// mod wick {
/// wick_import!();
/// }
///
#[macro_export]
macro_rules! wick_import {
  () => {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
  };
}

#[doc(hidden)]
#[macro_export]
macro_rules! handle_port {
  ($packet:ident, $tx:ident, $port:expr, $ty:ty) => {{
    use $crate::wasmrs_rx::Observer;
    use $crate::wick_packet::PacketExt;
    if $packet.is_done() {
      $tx.complete();
    } else {
      let _ = $tx.send($packet);
    }
  }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! stream_senders {
  ($error:ty, $name:ident, [ $(($port:expr, $($ty:tt)+)),* $(,)? ]) => {
    $crate::paste::paste! {
      struct $name {
        $(
          [<$port:snake>]: $crate::wasmrs_rx::FluxChannel<$crate::wick_packet::Packet,$error>
        ),*
      }
      impl $name {
        #[allow(clippy::missing_const_for_fn,unreachable_pub,unused,unused_parens)]
        pub fn receivers(&self) -> Option<($(
          $crate::BoxStream<$crate::wick_packet::VPacket<$($ty)*>>
        ),*)> {
          Some((
            $(Box::pin(self.[<$port:snake>].take_rx().ok()?.map($crate::wick_packet::VPacket::from_result))),*
          ))
        }
      }
      impl Default for $name {
        fn default() -> Self {
          $crate::paste::paste! {
            Self {
              $(
                [<$port:snake>]: $crate::wasmrs_rx::FluxChannel::new()
              ),*
            }
          }
        }
      }
    }
  };
}

#[doc(hidden)]
#[macro_export]
macro_rules! stream_receivers {
  ($error:ty, $name:ident, [ $(($port:expr, $($ty:tt)+)),* $(,)? ]) => {
    $crate::paste::paste! {
      struct $name {
        $(
          [<$port:snake>]: $crate::wasmrs_rx::FluxReceiver<$($ty)*,$error>
        ),*
      }
      impl $name {
        fn new(
          $(
            [<$port:snake>]: $crate::wasmrs_rx::FluxReceiver<$($ty)*,$error>
          ),*
        ) -> Self {
          $crate::paste::paste! {
            Self {
              $(
                [<$port:snake>]
              ),*
            }
          }
        }
      }
    }
  };
}

#[doc(hidden)]
#[macro_export]
macro_rules! payload_fan_out {
    (@handle_packet $payload: ident, $sender:ident, $config:ty) => {
      {
        let packet: $crate::wick_packet::Packet = $payload.into();

        if let Some(config_tx) = $sender.take() {
          if let Some(context) = packet.context() {
            let config: Result<$crate::wick_packet::ContextTransport<$config>, _> = $crate::wasmrs_codec::messagepack::deserialize(&context).map_err(|e|format!("Cound not deserialize context: {}", e));
            let _ = config_tx.send(config.map($crate::flow_component::Context::from));
          } else {
            // packet = $crate::wick_packet::Packet::component_error("No context attached to first invocation packet");
          }
        }
        packet
      }
    };
    (@handle_packet $payload: ident) => {
      {
        let packet: $crate::wick_packet::Packet = $payload.into();
        packet
      }
    };

    (@route_packet $packet:ident, $channels:ident, [ $(($port:expr, $($ty:tt)+)),* $(,)? ]) => {
      match $crate::wick_packet::PacketExt::port(&$packet) {
        $(
          $port => {
            let tx = &$crate::paste::paste! { $channels.[<$port:snake>] };
            $crate::handle_port!($packet, tx, $port, $($ty)*)
          }
        ),*
        $crate::wick_packet::Packet::FATAL_ERROR =>
        {
          #[allow(unused)]
          {
            use $crate::wasmrs_rx::Observer;
            let error = $packet.unwrap_err();
            $crate::paste::paste! {
              $(
                $channels.[<$port:snake>].send_result(Err($crate::anyhow::anyhow!(error.clone()))).unwrap();
              )*
            }
          }
        }
        _ => {
          // TODO: add tracing to warn when we're sent packets we aren't expecting
        }
      }
    };
    ($stream:expr, $error:ty, [ $(($port:expr, $($ty:tt)+)),* $(,)? ]) => {
      {
        $crate::stream_senders!($error, Channels, [ $(($port, $($ty)+)),* ]);
        #[allow(unused)]

        let channels = Channels::default();

        let output_streams = channels.receivers().unwrap();

        $crate::runtime::spawn("payload_fan_out", async move {
          #[allow(unused)]
          use $crate::StreamExt;
          loop {
            if let Some(Ok(payload)) = $stream.next().await {
              let packet = $crate::payload_fan_out!(@handle_packet payload);
              $crate::payload_fan_out!(@route_packet packet, channels, [ $(($port, $($ty)+)),* ]);
            } else {
              break;
            }
          }

        });
        output_streams
      }

    };
    ($stream:expr, $error:ty, $config:ty, [ $(($port:expr, $($ty:tt)+)),* $(,)? ]) => {
      {
        $crate::stream_senders!($error, Channels, [ $(($port, $($ty)+)),* ]);
        #[allow(unused)]
        let channels = Channels::default();

        let (config_tx,config_rx) = $crate::runtime::oneshot();
        let mut config_tx = Some(config_tx);
        let config_mono = Box::pin(async move {config_rx.await.unwrap()});
        let output_streams = (config_mono, channels.receivers().unwrap());

        $crate::runtime::spawn("payload_fan_out", async move {
          #[allow(unused)]
          use $crate::StreamExt;
          loop {
            if let Some(Ok(payload)) = $stream.next().await {
              let packet = $crate::payload_fan_out!(@handle_packet payload, config_tx, $config);
              $crate::payload_fan_out!(@route_packet packet, channels, [ $(($port, $($ty)+)),* ]);
            } else {
              break;
            }
          }
        });

        output_streams
      }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! propagate_if_error_then {
  (($($id:ident),*), $outputs:ident, $bail:expr) => {
    ($(
      match $id {
        Ok(value) => value,
        Err(err) => {
          $outputs.broadcast_err(err.to_string());
          $bail;
        }
      },
    )*)
  };
  ($result:expr, $outputs:ident, $bail:expr) => {
    match $result {
      Ok(value) => value,
      Err(err) => {
        $outputs.broadcast_err(err.to_string());
        $bail;
      }
    }
  };
}

/// Unwrap a [Result] value to its [Result::Ok] value or propagate the error to the downstream inputs and
/// short circuit the logic.
///
///
/// Takes a [Result] value, a generated component's Outputs, and an action to perform on error (`break`, `continue`, or `return expr`).
///
/// If the [Result] is [Ok], the [Result::Ok] value is returned.
///
/// If the [Result] is [Err], the error is propagated to the passed outputs,
/// and the logic is short circuited with the passed action.
///
/// # Example
///
/// ```
/// let results: Vec<Result<i32, ()>> = vec![Ok(1),Err(()),Ok(2)];
/// while let Some(result) = results.next() {
///   let value = propagate_if_error!(result, outputs, continue);
///   println!("{}", value);
/// }
/// ```
///
#[macro_export]
macro_rules! propagate_if_error {
  (($($id:ident),*), $outputs:ident, continue) => {
    $crate::propagate_if_error_then!(($($id),*), $outputs, continue)
  };
  ($result:expr, $outputs:ident, continue) => {
    $crate::propagate_if_error_then!($result, $outputs, continue)
  };
  ($result:expr,$outputs:ident, break) => {
    $crate::propagate_if_error_then!($result, $outputs, break)
  };
  ($result:expr,$outputs:ident, return $rv:expr) => {
    $crate::propagate_if_error_then!($result, $outputs, $rv)
  };
}

#[doc(hidden)]
#[macro_export]
macro_rules! if_done_close_then {
  ([$($id:ident),*], $do:expr) => {{
    $(
      if $id.is_done() || $id.is_close_bracket(){
        $do;
      }
    )*
  }};
}

#[allow(missing_docs)]
#[macro_export]
macro_rules! await_next_ok_or {
  ($stream:ident, $outputs:ident, continue) => {{
    let Some(next) = ($stream.next().await) else { break };
    let packet = propagate_if_error!(next, $outputs, continue);
    packet
  }};
  ($stream:ident, $outputs:ident, break) => {{
    let Some(next) = ($stream.next().await) else { break };
    let packet = propagate_if_error!(next, $outputs, break);
    packet
  }};
}

#[allow(missing_docs)]
#[macro_export]
macro_rules! make_substream_window {
  ($outputs:ident, $block:block) => {{
    $outputs.broadcast_open();
    $block;
    $outputs.broadcast_close();
  }};
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use tokio_stream::StreamExt;
  use wick_packet::{packet_stream, InherentData};
  #[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
  struct Config {}

  #[tokio::test]
  async fn test_fan_out() -> Result<()> {
    let mut stream = packet_stream!(("foo", 1), ("bar", 2), ("foo", 3), ("bar", 4), ("foo", 5), ("bar", 6));
    stream.set_context(Default::default(), InherentData::unsafe_default());
    let (_config, (mut foo_rx, mut bar_rx)) =
      payload_fan_out!(stream, anyhow::Error, Config, [("foo", i32), ("bar", i32)]);
    assert_eq!(foo_rx.next().await.unwrap().decode().unwrap(), 1);
    assert_eq!(bar_rx.next().await.unwrap().decode().unwrap(), 2);
    assert_eq!(foo_rx.next().await.unwrap().decode().unwrap(), 3);
    assert_eq!(bar_rx.next().await.unwrap().decode().unwrap(), 4);
    assert_eq!(foo_rx.next().await.unwrap().decode().unwrap(), 5);
    assert_eq!(bar_rx.next().await.unwrap().decode().unwrap(), 6);

    Ok(())
  }

  #[tokio::test]
  async fn test_fan_out_no_config() -> Result<()> {
    let mut stream = packet_stream!(("foo", 1), ("bar", 2), ("foo", 3), ("bar", 4), ("foo", 5), ("bar", 6));
    stream.set_context(Default::default(), InherentData::unsafe_default());
    let (mut foo_rx, mut bar_rx) = payload_fan_out!(stream, anyhow::Error, [("foo", i32), ("bar", i32)]);
    assert_eq!(foo_rx.next().await.unwrap().decode().unwrap(), 1);
    assert_eq!(bar_rx.next().await.unwrap().decode().unwrap(), 2);
    assert_eq!(foo_rx.next().await.unwrap().decode().unwrap(), 3);
    assert_eq!(bar_rx.next().await.unwrap().decode().unwrap(), 4);
    assert_eq!(foo_rx.next().await.unwrap().decode().unwrap(), 5);
    assert_eq!(bar_rx.next().await.unwrap().decode().unwrap(), 6);

    Ok(())
  }

  #[tokio::test]
  async fn test_fan_out_no_fields() -> Result<()> {
    let mut stream = packet_stream!(("foo", 1), ("bar", 2), ("foo", 3), ("bar", 4), ("foo", 5), ("bar", 6));
    stream.set_context(Default::default(), InherentData::unsafe_default());
    let _config = payload_fan_out!(stream, anyhow::Error, Config, []);

    Ok(())
  }
}
