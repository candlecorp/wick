use data_encoding::HEXUPPER;
use ring::digest::{Context, SHA256};
use wascap::jwt::Token;
use wascap::prelude::Claims;

use crate::parser::ParsedModule;
use crate::{Error, WickComponent};

pub(crate) const SECTION_NAME: &str = "wick/claims@v1";

/// A common struct to group related options together.
#[derive(Debug, Default, Clone)]
pub struct ClaimsOptions {
  /// The version of the claims target.
  pub version: Option<String>,
  /// When the target expires.
  pub expires_in_days: Option<u64>,
  /// When the target becomes valid.
  pub not_before_days: Option<u64>,
}

pub(crate) fn decode(bytes: &[u8]) -> Result<Token<WickComponent>, Error> {
  let jwt = std::str::from_utf8(bytes)?;
  let claims: Claims<WickComponent> = Claims::decode(jwt)?;
  Ok(Token {
    jwt: jwt.to_owned(),
    claims,
  })
}

// The v1::hash method is more resilient to different ways of generating WebAssembly modules.
//
// It deals with each section individually instead of hashing the entire module at once.
pub(crate) fn hash(module: &ParsedModule, filter: &[&str]) -> Result<String, Error> {
  let mut context = Context::new(&SHA256);

  for payload in &module.1 {
    use wasmparser::Payload::*;

    let section = payload.as_section();
    match payload {
      Version { encoding, .. } => {
        // encode the header manually
        context.update(match encoding {
          wasmparser::Encoding::Module => &wasm_encoder::Module::HEADER,
          wasmparser::Encoding::Component => todo!(),
        });
      }
      CustomSection(c) if (filter.contains(&c.name())) => {
        // skip if it's a custom section we're filtering
      }
      _ => {
        // otherwise encode the section if it's from the original module
        if let Some((id, range)) = section {
          context.update(&id.to_le_bytes());
          let bytes = &module.0[range];
          context.update(&bytes.len().to_le_bytes());
          context.update(bytes);
        }
      }
    }
  }

  Ok(HEXUPPER.encode(context.finish().as_ref()))
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;
  use crate::parser::ParsedModule;

  #[rstest::rstest]
  #[case(
    "./test/1.v0.signed.wasm",
    "90E5D03AF45BAE5EFC5841C196A2774BEB783E4E041E1D6D1421073765D47E50"
  )]
  #[case(
    "./test/2.v0.signed.wasm",
    "846CBC6E9D35321E0A81D150B1CCA2816EAD9E53DAF0AA12BD2FB44E19E7605C"
  )]
  #[case(
    "./test/3.v0.signed.wasm",
    "7A68971E61256D7B76FA580B2E17B173B943B1B737E65C5EB6AECA6D37312EEE"
  )]
  #[case(
    "./test/4.v0.signed.wasm",
    "8DD28458BE618E260A70390FAEB5E74160823F979A32F6167F3C3D3D1C2C08BB"
  )]
  fn test_signature(#[case] file: &str, #[case] expected_hash: &str) -> Result<()> {
    let bytes = std::fs::read(file)?;
    let module = ParsedModule::new(&bytes)?;
    let hash = hash(&module, &[SECTION_NAME, crate::v0::SECTION_NAME])?;
    assert_eq!(hash, expected_hash);

    Ok(())
  }
}
