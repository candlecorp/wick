mod wick {
  wick_component::wick_import!();
}
use std::io::Write;

use anyhow::bail;
use wick::*;
use wick_component::wick_packet::Packet;

// simple_unary_operation!(read_string);

#[wick_component::operation(unary::simple)]
fn read_string(filename: &String) -> Result<String, std::io::Error> {
  std::fs::read_to_string(filename)
}

// simple_unary_operation!(read_bytes);

#[wick_component::operation(unary::simple)]
fn read_bytes(filename: &String) -> Result<Bytes, std::io::Error> {
  std::fs::read(filename).map(Into::into)
}

// paired_right_stream!(write_string);

// fn write_string(
//   filename: String,
//   contents: WickStream<String>,
// ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64, anyhow::Error>> + 'static>> {
//   Box::pin(async move { real_write_string(filename, contents).await })
// }

#[wick_component::operation(binary::paired_right_stream)]
async fn real_write_string(filename: String, mut contents: WickStream<String>) -> Result<u64, anyhow::Error> {
  let mut file = std::fs::File::create(filename)?;
  let mut size = 0;
  while let Some(contents) = contents.next().await {
    match contents {
      Ok(string) => {
        size += file.write(string.as_bytes())?;
      }
      Err(e) => {
        bail!(e);
      }
    }
  }

  Ok(size as _)
}
