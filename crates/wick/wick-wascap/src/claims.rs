use std::borrow::Cow;
use std::time::{SystemTime, UNIX_EPOCH};

use tracing::debug;
use wascap::jwt::Token;
use wascap::prelude::{Claims, KeyPair};
use wascap::wasm::days_from_now_to_jwt_time;
use wick_interface_types::ComponentSignature;

use crate::component::WickComponent;
use crate::parser::{CustomSection, ParsedModule};
use crate::{error, v0, v1};

type Result<T> = std::result::Result<T, error::Error>;

/// A common struct to group related options together.
#[derive(Debug, Clone)]
pub enum ClaimsOptions {
  /// Version 0 Claims
  V0(v0::ClaimsOptions),
  /// Version 1 Claims
  V1(v1::ClaimsOptions),
}

impl ClaimsOptions {
  /// Create a new v0 claims.
  #[must_use]
  pub fn v0(
    revision: Option<u32>,
    version: Option<String>,
    expires_in_days: Option<u64>,
    not_before_days: Option<u64>,
  ) -> Self {
    Self::V0(v0::ClaimsOptions {
      revision,
      version,
      expires_in_days,
      not_before_days,
    })
  }

  /// Create a new v1 claims.
  #[must_use]
  pub fn v1(version: Option<String>, expires_in_days: Option<u64>, not_before_days: Option<u64>) -> Self {
    Self::V1(v1::ClaimsOptions {
      version,
      expires_in_days,
      not_before_days,
    })
  }

  /// Get when the claims expire in days
  #[must_use]
  pub fn expires_in_days(&self) -> Option<u64> {
    match self {
      Self::V0(opts) => opts.expires_in_days,
      Self::V1(opts) => opts.expires_in_days,
    }
  }

  /// Get when the claims are valid (in days)
  #[must_use]
  pub fn not_before_days(&self) -> Option<u64> {
    match self {
      Self::V0(opts) => opts.not_before_days,
      Self::V1(opts) => opts.not_before_days,
    }
  }

  /// Get the version described by the claims
  #[must_use]
  pub fn version(&self) -> Option<String> {
    match self {
      Self::V0(opts) => opts.version.clone(),
      Self::V1(opts) => opts.version.clone(),
    }
  }
}

/// Extract the claims embedded in a WebAssembly module.
pub fn extract_claims(contents: impl AsRef<[u8]>) -> Result<Option<Token<WickComponent>>> {
  let module = ParsedModule::new(contents.as_ref())?;
  let v0_section = module.get_custom_section(v0::SECTION_NAME);
  let v1_section = module.get_custom_section(v1::SECTION_NAME);

  let (token, target_hash) = if let Some(section) = v0_section {
    debug!(section= %v0::SECTION_NAME,"wasm:claims: decoding v0 token");
    (
      v0::decode(section)?,
      v0::hash(&module, &[v0::SECTION_NAME, v1::SECTION_NAME])?,
    )
  } else if let Some(section) = v1_section {
    debug!(section= %v1::SECTION_NAME,"wasm:claims: decoding v1 token");
    (
      v1::decode(section)?,
      v1::hash(&module, &[v0::SECTION_NAME, v1::SECTION_NAME])?,
    )
  } else {
    return Err(error::Error::InvalidModuleFormat);
  };

  debug!(?token, %target_hash, "wasm:claims");

  if let Some(ref meta) = token.claims.metadata {
    if meta.module_hash != target_hash {
      Err(error::Error::InvalidModuleHash)
    } else {
      Ok(Some(token))
    }
  } else {
    Err(error::Error::InvalidModuleFormat)
  }
}

/// This function will embed a set of claims inside the bytecode of a WebAssembly module. The claims.
/// are converted into a JWT and signed using the provided `KeyPair`.
pub(crate) fn embed_claims(orig_bytecode: &[u8], mut claims: Claims<WickComponent>, kp: &KeyPair) -> Result<Vec<u8>> {
  let module = ParsedModule::new(orig_bytecode)?;
  let module = module
    .remove_custom_section(v0::SECTION_NAME)
    .remove_custom_section(v1::SECTION_NAME);
  let hash = module.hash(&[])?;

  let meta = claims.metadata.map(|md| WickComponent {
    module_hash: hash,
    ..md
  });
  claims.metadata = meta;

  let encoded = claims.encode(kp)?;
  let encvec = encoded.as_bytes().to_vec();
  let custom_section = CustomSection::new(v1::SECTION_NAME.to_owned(), Cow::Owned(encvec));
  Ok(module.emit_wasm([custom_section]))
}

/// Build collection claims from passed values
#[must_use]
pub(crate) fn build_collection_claims(
  interface: ComponentSignature,
  subject_kp: &KeyPair,
  issuer_kp: &KeyPair,
  options: &ClaimsOptions,
) -> Claims<WickComponent> {
  Claims::<WickComponent> {
    expires: options.expires_in_days(),
    id: nuid::next(),
    issued_at: since_the_epoch().as_secs(),
    issuer: issuer_kp.public_key(),
    subject: subject_kp.public_key(),
    not_before: days_from_now_to_jwt_time(options.not_before_days()),
    metadata: Some(WickComponent {
      module_hash: "".to_owned(),
      tags: Some(Vec::new()),
      interface,
      ver: options.version(),
    }),
  }
}

#[allow(clippy::too_many_arguments)]
/// Sign WebAssembly bytes with the passed claims.
pub fn sign_buffer_with_claims(
  buf: impl AsRef<[u8]>,
  interface: ComponentSignature,
  mod_kp: &KeyPair,
  acct_kp: &KeyPair,
  options: &ClaimsOptions,
) -> Result<Vec<u8>> {
  let claims = build_collection_claims(interface, mod_kp, acct_kp, options);

  embed_claims(buf.as_ref(), claims, acct_kp)
}

fn since_the_epoch() -> std::time::Duration {
  let start = SystemTime::now();
  start.duration_since(UNIX_EPOCH).unwrap()
}
