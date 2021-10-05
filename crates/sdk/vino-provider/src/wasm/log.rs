/// WASM analog for log::info! macro
#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => (
      $crate::__log!("0", $($arg)+)
    )
}

/// WASM analog for log::error! macro
#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => (
      $crate::__log!("1", $($arg)+)
    )
}

/// WASM analog for log::warn! macro
#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => (
      $crate::__log!("2", $($arg)+)
    )
}

/// WASM analog for log::debug! macro
#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => (
      $crate::__log!("3", $($arg)+)
    )
}

/// WASM analog for log::trace! macro
#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => (
      $crate::__log!("4", $($arg)+)
    )
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! __log {
  ($lvl:expr, $($args:tt)*) => ({
      let lvl = $lvl;
      $crate::wasm::host_call("2", lvl, &std::format!($($args)*), &[])
  });
}
