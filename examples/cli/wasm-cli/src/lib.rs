mod wick {
  #![allow(unused_imports, missing_debug_implementations, clippy::needless_pass_by_value)]
  #[rustfmt::skip]
  wick_component::wick_import!();
}
use imported::app_config::static_site;
use wick::*;
use wick_component::wick_packet::{packet_stream, raw_packet_stream};

#[wick_component::operation(unary_simple)]
async fn main(_args: Vec<String>, ctx: Context<main::Config>) -> anyhow::Result<u64> {
  /*
   * This example highlights three different ways to make component calls from wasm,
   * each with increasing levels of control.
   *
   * This component imports a component we named "app_config" that has one operation
   * named "static_site"
   */

  /*
   * The StaticSiteConfig struct is the configuration for the static_site operation
   */
  let config = static_site::Config {
    app_name: "my new app".to_string(),
  };

  /*
   * The first way to call operations is to use the method on the imported component
   * directly. This is the simplest way to call a component, but does not give you
   * access to the underlying packet stream nor any signals that may be present.
   */
  let mut stream = ctx
    .imported()
    .app_config
    .static_site(config, static_site::Request { dir: "dist".to_owned() })?;
  println!("Printing packets received from app_config.static_site");
  while let Some(packet) = stream.yaml.next().await {
    let packet = packet.decode()?;

    println!("{}", packet);
  }

  let config = static_site::Config {
    app_name: "my new app".to_string(),
  };

  /*
   * If you need access to the underlying input or output stream, use the method
   * on the imported component named <operation>_raw. This method takes in a combined
   * stream of Packet values and returns a combined stream of packet values.
   */
  let mut stream = ctx
    .imported()
    .app_config
    .static_site_packets(config, packet_stream!(("dir", "dist")))?;

  println!("Printing packets received from app_config.static_site_raw()");
  while let Some(packet) = stream.next().await {
    let packet = packet?;
    if !packet.has_data() {
      continue;
    }
    let packet = packet.decode::<String>()?;

    println!("{}", packet);
  }

  /*
   * The third method bypasses all of the generated code and types for the imported operation
   * and lets you call an operation by name directly. This is the most flexible way to call
   * imported operations and is the only way to call operations dynamically.
   */
  let mut stream_direct = ctx.imported().app_config.component().call(
    "static_site",
    raw_packet_stream!(("dir", "dist")),
    Some(json!({"app_name":"my super app"}).try_into()?),
    ctx.inherent.clone().into(),
  )?;
  println!("Printing packets received from the app_config.component().call()");
  while let Some(packet) = stream_direct.next().await {
    let packet: Packet = packet?.try_into()?;
    if !packet.has_data() {
      continue;
    }
    let packet = packet.decode::<String>()?;

    println!("{}", packet);
  }

  Ok(0)
}
