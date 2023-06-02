use console::set_colors_enabled;
use wasmrs_guest::StreamExt;
use wick_component::once;
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
impl MainOperation for Component {
  type Error = Box<dyn std::error::Error>;
  type Outputs = main::Outputs;
  type Config = main::Config;

  async fn main(
    mut args: WickStream<Vec<String>>,
    mut is_interactive: WickStream<types::cli::Interactive>,
    mut outputs: Self::Outputs,
    ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    set_colors_enabled(false);
    println!("\ncli:db: in WebAssembly CLI component");

    while let (Some(Ok(args)), Some(Ok(_tty))) = (args.next().await, is_interactive.next().await) {
      let id: u32 = args.get(0).unwrap_or(&"0".to_string()).parse().unwrap_or(1);

      println!("cli:db: looking up user with id: {}.", console::style(id).green());

      let provided = get_provided();
      println!(
        "cli:db: calling provided component operation at URL: {}",
        console::style(format!("{}get_user", provided.db.component())).green()
      );
      let mut response = ctx.provided().db.get_user(once(id))?;

      println!("cli:db: call succeeded, waiting for response...");

      while let Some(packet) = response.next().await {
        match packet {
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
