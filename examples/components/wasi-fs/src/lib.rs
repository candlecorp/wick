mod wick {
  wick_component::wick_import!();
}
use std::io::Write;

use wick::*;

#[wick_component::operation(unary_simple)]
fn read_string(filename: String, _ctx: Context<read_string::Config>) -> Result<String, std::io::Error> {
  std::fs::read_to_string(&filename)
}

#[wick_component::operation(unary_simple)]
fn read_bytes(filename: String, _ctx: Context<read_bytes::Config>) -> Result<Bytes, std::io::Error> {
  std::fs::read(filename).map(Into::into)
}

#[wick_component::operation(binary_paired_right_stream)]
async fn write_string(
  filename: String,
  mut contents: WickStream<String>,
  _ctx: Context<write_string::Config>,
) -> Result<u64, anyhow::Error> {
  let mut file = std::fs::File::create(filename)?;
  let mut size = 0;
  while let Some(contents) = contents.next().await {
    match contents {
      Ok(string) => {
        size += file.write(string.as_bytes())?;
      }
      Err(e) => {
        anyhow::bail!(e);
      }
    }
  }

  Ok(size as _)
}
