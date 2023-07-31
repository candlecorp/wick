mod wick {
  wick_component::wick_import!();
}
use wick::*;

#[wick_component::operation(binary_interleaved_pairs)]
fn main(_args: Vec<String>, _interactive: types::cli::Interactive, _ctx: Context<main::Config>) -> anyhow::Result<u32> {
  println!("Hello world!");
  return Ok(0);
}
