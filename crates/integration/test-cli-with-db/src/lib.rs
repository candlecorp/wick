use console::set_colors_enabled;

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

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl main::Operation for Component {
  type Error = anyhow::Error;
  type Inputs = main::Inputs;
  type Outputs = main::Outputs;
  type Config = main::Config;

  async fn main(
    inputs: Self::Inputs,
    mut outputs: Self::Outputs,
    ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    set_colors_enabled(false);
    println!("\ncli:db: in WebAssembly CLI component");
    let mut args = inputs.args;
    while let Some(args) = args.next().await {
      let args = args.decode()?;

      let id: u32 = args.get(0).unwrap_or(&"0".to_string()).parse().unwrap_or(1);

      println!("cli:db: looking up user with id: {}.", console::style(id).green());

      let provided = ctx.provided();
      println!(
        "cli:db: calling provided component operation at URL: {}",
        console::style(format!("{}get_user", provided.db.component())).green()
      );

      let mut get_user = provided::db::get_user::Inputs::new();
      get_user.id.send(&id);
      let mut response = ctx.provided().db.get_user(Default::default(), get_user)?;

      println!("cli:db: call succeeded, waiting for response...");

      while let Some(packet) = response.output.next().await {
        match packet.decode() {
          Ok(packet) => {
            println!("cli:db: row data: {}", console::style(packet).green());
          }
          Err(e) => {
            println!("cli:db: got error! {}", console::style(e).red());
          }
        }
      }
      println!("cli:db: response stream ended.");
    }
    println!("cli:db: sending output code.");

    let _ = outputs.code.send(&0);
    let _ = outputs.code.done();

    println!("cli:db: done.");
    Ok(())
  }
}
