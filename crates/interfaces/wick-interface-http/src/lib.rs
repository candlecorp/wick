#[cfg(feature = "localgen")]
#[rustfmt::skip]
mod generated;
#[cfg(feature = "localgen")]
use generated as wick;
#[cfg(not(feature = "localgen"))]
mod wick {
  #![allow(unused_imports, missing_debug_implementations, clippy::needless_pass_by_value)]
  #[rustfmt::skip]
  wick_component::wick_import!();
}
pub use wick::*;
