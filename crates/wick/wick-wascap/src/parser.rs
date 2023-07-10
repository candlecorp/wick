use std::mem;

use wasm_encoder::{ComponentSectionId, Encode, Section};
use wasmparser::Parser;

use crate::Error;

pub(crate) fn strip_custom_section(buf: &[u8]) -> Result<Vec<u8>, Error> {
  let mut output: Vec<u8> = Vec::new();
  let mut stack = Vec::new();
  for payload in Parser::new(0).parse_all(buf) {
    let payload = payload?;
    use wasmparser::Payload::*;
    match payload {
      Version { encoding, .. } => {
        output.extend_from_slice(match encoding {
          wasmparser::Encoding::Component => &wasm_encoder::Component::HEADER,
          wasmparser::Encoding::Module => &wasm_encoder::Module::HEADER,
        });
      }
      ModuleSection { .. } | ComponentSection { .. } => {
        stack.push(mem::take(&mut output));
        continue;
      }
      End { .. } => {
        let mut parent = match stack.pop() {
          Some(c) => c,
          None => break,
        };
        if output.starts_with(&wasm_encoder::Component::HEADER) {
          parent.push(ComponentSectionId::Component as u8);
          output.encode(&mut parent);
        } else {
          parent.push(ComponentSectionId::CoreModule as u8);
          output.encode(&mut parent);
        }
        output = parent;
      }
      _ => {}
    }

    match payload {
      CustomSection(c) if (c.name() == "jwt") => {
        // skip
      }
      _ => {
        if let Some((id, range)) = payload.as_section() {
          wasm_encoder::RawSection { id, data: &buf[range] }.append_to(&mut output);
        }
      }
    }
  }

  Ok(output)
}
