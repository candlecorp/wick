pub mod ephemeral;
#[cfg(not(target_arch = "wasm32"))]
pub mod native;
pub mod stateful;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

/// Utility type for a Box<dyn std::error::Error + Send + Sync>
pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;

#[macro_export]
#[doc(hidden)]
macro_rules! console_log {
  ($($args:tt)*) => ({
    #[cfg(target_arch = "wasm32")]
      $crate::guest::wasm::runtime::console_log(&std::format!($($args)*));
    #[cfg(not(target_arch = "wasm32"))]
      &std::println!($($args)*);
  });
}
