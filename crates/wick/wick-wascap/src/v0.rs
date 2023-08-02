use data_encoding::HEXUPPER;
use ring::digest::{Context, SHA256};

use crate::claims::{Claims, Token};
use crate::parser::ParsedModule;
use crate::{Error, WickComponent};

pub(crate) const SECTION_NAME: &str = "jwt";

/// A common struct to group related options together.
#[derive(Debug, Default, Clone)]
pub struct ClaimsOptions {
  /// The revision of the claims target.
  pub revision: Option<u32>,
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
          context.update(&encode_old_u32(range.size_hint().0 as _));
          context.update(&module.0[range]);
        }
      }
    }
  }

  Ok(HEXUPPER.encode(context.finish().as_ref()))
}
// Commenting this out because I don't want it to get lost. Recreating the original hashes was kind of
// a PITA and this was a useful tool if I need to troubleshoot in the future.
//
// Feel free to delete if the v0 hash footprint is negligible - JSO 2023-07-11
//
// pub(crate) fn regen(module: &ParsedModule, filter: &[&str]) -> Result<Vec<u8>, Error> {
//   let mut context = Vec::new();

//   for payload in &module.1 {
//     use wasmparser::Payload::*;

//     let section = payload.as_section();
//     match payload {
//       Version { encoding, .. } => {
//         // encode the header manually
//         context.extend(match encoding {
//           wasmparser::Encoding::Module => &wasm_encoder::Module::HEADER,
//           wasmparser::Encoding::Component => todo!(),
//         });
//       }
//       CustomSection(c) if (filter.contains(&c.name())) => {
//         // skip if it's a custom section we're filtering
//       }
//       _ => {
//         // otherwise encode the section if it's from the original module
//         if let Some((id, range)) = section {
//           context.extend(&id.to_le_bytes());
//           context.extend(encode_old_u32(range.size_hint().0 as _));
//           context.extend(&module.0[range]);
//           // encode_section(range, module.0, &mut context);
//         }
//       }
//     }
//   }

//   Ok(context)
// }

// fn encode_section(range: std::ops::Range<usize>, buff: &[u8], sink: &mut Vec<u8>) {
//   let bytes = &buff[range];
//   let total_size = (bytes.len() as u32);
//   sink.extend(encode_old_u32(total_size));
//   // encode_old_u32(sink, total_size);
//   sink.extend(bytes);
// }

// This was taken from the old walrus encoder: https://github.com/rustwasm/walrus/commit/3af60e01e0e27f033c7424b09aecd0c863c5e415#diff-5bcac9d065ac0e857e3f34b17fe1e84b5849f674851fa77670b08600d87c768a
//
// This has been replaced in modern versions with leb128 implementations so keep it around until v0 hashes are gone.
const MAX_U32_LENGTH: usize = 5;

fn encode_old_u32(mut amt: u32) -> Vec<u8> {
  let mut bytes = Vec::new();
  #[allow(clippy::needless_range_loop)]
  for i in 0..MAX_U32_LENGTH {
    let flag = if i == MAX_U32_LENGTH - 1 { 0 } else { 0x80 };
    bytes.push((amt as u8) & 0x7f | flag);
    amt >>= 7;
  }
  bytes
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;
  use crate::parser::ParsedModule;

  #[rstest::rstest]
  #[case(
    "./test/1.v0.signed.wasm",
    "D3DFCF7F12B01A22025B2341871A46B5A4EE71387B32EE857EDBE69F2D1E1299"
  )]
  #[case(
    "./test/2.v0.signed.wasm",
    "2535F3568A2E0798AA376A6F836A65C81F1A258156F9E98E94B33A0E42EFC2C2"
  )]
  #[case(
    "./test/3.v0.signed.wasm",
    "8CF411C08AEEF40150E70E0210A4C5A67559871FDB43351664A42DC6F94B8DC5"
  )]
  #[case(
    "./test/4.v0.signed.wasm",
    "7E215B19354779A37A5C01740D8D129536C38E1A2659A916F440418129924A11"
  )]
  fn test_signature(#[case] file: &str, #[case] expected_hash: &str) -> Result<()> {
    let bytes = std::fs::read(file)?;
    let module = ParsedModule::new(&bytes)?;
    let hash = hash(&module, &[SECTION_NAME])?;
    assert_eq!(hash, expected_hash);

    Ok(())
  }
}
