use console::set_colors_enabled;
use wasmrs_guest::StreamExt;
use wick_component::packet::{self as wick_packet, packet_stream};
mod wick {
  wick_component::wick_import!();
}
use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl OpMain for Component {
  async fn main(
    mut args: WickStream<Vec<String>>,
    mut is_interactive: WickStream<types::cli::Interactive>,
    mut outputs: OpMainOutputs,
  ) -> Result<()> {
    set_colors_enabled(false);
    println!("\nIn WebAssembly CLI component");

    while let (Some(Ok(args)), Some(Ok(_tty))) = (args.next().await, is_interactive.next().await) {
      let id: u32 = args.get(0).unwrap_or(&"0".to_string()).parse().unwrap_or(1);

      println!("Looking up user with id: {}.", console::style(id).green());

      let packets = packet_stream!(("id", id));

      let provided = get_provided();
      println!(
        "Calling provided component operation at URL: {}",
        console::style(format!("{}get_user", provided.db.component())).green()
      );
      let mut response = provided.db.get_user(packets)?;

      println!("Call succeeded, waiting for response...");

      while let Some(packet) = response.next().await {
        match packet {
          Ok(packet) => {
            println!("Row data: {}", console::style(packet).green());
          }
          Err(e) => {
            println!("Got error! {}", console::style(e).red());
          }
        }
      }
    }

    let _ = outputs.code.send(&0);
    let _ = outputs.code.done();
    Ok(())
  }
}
