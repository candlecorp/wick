use std::borrow::Cow;

use wasm_encoder::{RawSection, Section};
use wasmparser::{Parser, Payload};

use crate::{v1, Error};

pub(crate) struct CustomSection<'a>(String, Cow<'a, [u8]>);

impl<'a> CustomSection<'a> {
  pub(crate) const fn new(name: String, data: Cow<'a, [u8]>) -> Self {
    Self(name, data)
  }
}

pub(crate) struct ParsedModule<'a>(pub(crate) &'a [u8], pub(crate) Vec<Payload<'a>>);

impl<'a> ParsedModule<'a> {
  pub(crate) fn new(buf: &'a [u8]) -> Result<Self, Error> {
    let parser = Parser::new(0);
    let mut payloads = Vec::new();

    for payload in parser.parse_all(buf) {
      let payload = payload?;
      payloads.push(payload);
    }

    Ok(Self(buf, payloads))
  }

  pub(crate) fn emit_wasm<'b, const K: usize>(&'a self, custom_sections: [CustomSection<'b>; K]) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::new();

    for payload in &self.1 {
      use wasmparser::Payload::*;
      match payload {
        Version { encoding, .. } => {
          output.extend(match encoding {
            wasmparser::Encoding::Component => unimplemented!("component model components not implemented yet"),
            wasmparser::Encoding::Module => wasm_encoder::Module::HEADER,
          });
        }
        CodeSectionEntry(_) => {
          //skip
        }
        _ => {
          if let Some(section) = payload.as_section() {
            RawSection {
              id: section.0,
              data: &self.0[section.1],
            }
            .append_to(&mut output);
          }
        }
      }
    }

    for section in custom_sections {
      wasm_encoder::CustomSection {
        name: Cow::Borrowed(&section.0),
        data: section.1,
      }
      .append_to(&mut output);
    }

    output
  }

  pub(crate) fn get_custom_section(&'a self, name: &str) -> Option<&'a [u8]> {
    for payload in &self.1 {
      use wasmparser::Payload::*;
      match payload {
        CustomSection(c) if (c.name() == name) => {
          return Some(c.data());
        }
        _ => {
          //skip
        }
      }
    }

    None
  }

  pub(crate) fn remove_custom_section(self, name: &str) -> Self {
    let mut output: Vec<Payload> = Vec::new();

    for payload in self.1 {
      use wasmparser::Payload::*;

      if matches!(payload, CustomSection(ref c) if (c.name() == name)) {
        continue;
      }
      output.push(payload);
    }

    Self(self.0, output)
  }

  pub(crate) fn hash(&self, filter: &[&str]) -> Result<String, Error> {
    v1::hash(self, filter)
  }
}
