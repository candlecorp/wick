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

#[doc(hidden)]
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
            let packet: $crate::wick_packet::Packet = payload.into();
            match packet.port() {
              $(
                $port=> {
                  let tx = &$crate::paste::paste! {[<$port:snake _tx>]};
                  $crate::handle_port!(raw: $raw, packet, tx, $port, $($ty)*)
                },
              )*
              $crate::wick_packet::Packet::FATAL_ERROR => {
                let error = packet.unwrap_err();
                $crate::paste::paste! {
                  $(
                    [<$port:snake _tx>].send_result(Err(Box::new($crate::flow_component::ComponentError::message(error.msg())).into())).unwrap();
                  )*
                }
              }
              _ => {
                // TODO: add tracing to warn when we're sent packets we aren't expecting
              }
            }
          } else {
            break;
          }
        }

      });
      $crate::paste::paste! {($(Box::pin([<$port:snake _rx>])),*)}
      }

    };
    ($stream:expr, raw:$raw:tt, $error:ty, $config:ty, [ ]) => {
      {
        let (config_tx,config_rx) = $crate::runtime::oneshot();

        $crate::runtime::spawn("payload_fan_out",async move {
          #[allow(unused)]
          use $crate::StreamExt;
          let mut config_tx = Some(config_tx);
          loop {
            if let Some(Ok(payload)) = $stream.next().await {
              let mut packet: $crate::wick_packet::Packet = payload.into();
              if let Some(config_tx) = config_tx.take() {
                if let Some(context) = packet.context() {
                  let config: Result<$crate::wick_packet::ContextTransport<$config>, _> = $crate::wasmrs_codec::messagepack::deserialize(&context).map_err(|_e|$crate::flow_component::ComponentError::message("Cound not deserialize Context"));
                  let _ = config_tx.send(config.map($crate::flow_component::Context::from));
                } else {
                  packet = $crate::wick_packet::Packet::component_error("No context attached to first invocation packet");
                }
              }
            } else {
              break;
            }
          }
        });
        let config_mono = Box::pin(config_rx);
        config_mono
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
              let mut packet: $crate::wick_packet::Packet = payload.into();
              if let Some(config_tx) = config_tx.take() {
                if let Some(context) = packet.context() {
                  let config: Result<$crate::wick_packet::ContextTransport<$config>, _> = $crate::wasmrs_codec::messagepack::deserialize(&context).map_err(|e|$crate::flow_component::ComponentError::message(&format!("Cound not deserialize context: {}", e)));
                  let _ = config_tx.send(config.map($crate::flow_component::Context::from));
                } else {
                  packet = $crate::wick_packet::Packet::component_error("No context attached to first invocation packet");
                }
              }

              match packet.port() {
                $(
                  $port=> {
                    let tx = &$crate::paste::paste! {[<$port:snake _tx>]};
                    $crate::handle_port!(raw: $raw, packet, tx, $port, $($ty)*)
                  },
                )*
                $crate::wick_packet::Packet::FATAL_ERROR => {
                  use $crate::wasmrs_rx::Observer;
                  let error = packet.unwrap_err();
                  $crate::paste::paste! {
                    $(
                      [<$port:snake _tx>].send_result(Err(Box::new($crate::flow_component::ComponentError::message(error.msg())).into())).unwrap();
                    )*
                  }
                }
                _ => {
                  // TODO: add tracing to warn when we're sent packets we aren't expecting
                }
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
