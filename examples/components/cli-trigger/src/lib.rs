#[cfg(feature = "localgen")]
mod generated;
#[cfg(feature = "localgen")]
use generated as wick;
#[cfg(not(feature = "localgen"))]
mod wick {
  #![allow(unused_imports, missing_debug_implementations, clippy::needless_pass_by_value)]
  wick_component::wick_import!();
}
use wick::*;

#[wick_component::operation(binary_interleaved_pairs)]
fn main(_args: Vec<String>, _interactive: types::cli::Interactive, _ctx: Context<main::Config>) -> anyhow::Result<u32> {
  println!("Hello world!");
  return Ok(0);
}
