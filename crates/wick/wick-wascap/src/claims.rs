mod validate;

use std::borrow::Cow;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use nkeys::KeyPair;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::debug;
pub use validate::validate_token;
use wick_interface_types::ComponentSignature;

use crate::component::WickComponent;
use crate::parser::{CustomSection, ParsedModule};
use crate::{base64, error, v0, v1, Error};
const HEADER_TYPE: &str = "jwt";
const HEADER_ALGORITHM: &str = "Ed25519";

type Result<T> = std::result::Result<T, Error>;

/// A structure containing a JWT and its associated decoded claims
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[non_exhaustive]
pub struct Token<T> {
  /// The JWT itself
  pub jwt: String,
  /// The decoded claims
  pub claims: Claims<T>,
}

/// Represents a set of [RFC 7519](https://tools.ietf.org/html/rfc7519) compliant JSON Web Token
/// claims.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
#[non_exhaustive]
pub struct Claims<T> {
  /// All timestamps in JWTs are stored in _seconds since the epoch_ format
  /// as described as `NumericDate` in the RFC. Corresponds to the `exp` field in a JWT.
  #[serde(rename = "exp", skip_serializing_if = "Option::is_none")]
  pub expires: Option<u64>,

  /// Corresponds to the `jti` field in a JWT.
  #[serde(rename = "jti")]
  pub id: String,

  /// The `iat` field, stored in _seconds since the epoch_
  #[serde(rename = "iat")]
  pub issued_at: u64,

  /// Issuer of the token, by convention usually the public key of the _account_ that
  /// signed the token
  #[serde(rename = "iss")]
  pub issuer: String,

  /// Subject of the token, usually the public key of the _module_ corresponding to the WebAssembly file
  /// being signed
  #[serde(rename = "sub")]
  pub subject: String,

  /// The `nbf` JWT field, indicates the time when the token becomes valid. If `None` token is valid immediately
  #[serde(rename = "nbf", skip_serializing_if = "Option::is_none")]
  pub not_before: Option<u64>,

  /// Custom jwt claims in the `wascap` namespace
  #[serde(rename = "wascap", skip_serializing_if = "Option::is_none")]
  pub metadata: Option<T>,
}

impl<T> Claims<T>
where
  T: Serialize + DeserializeOwned + Named,
{
  pub(crate) fn encode(&self, kp: &KeyPair) -> Result<String> {
    let header = ClaimsHeader {
      header_type: HEADER_TYPE.to_owned(),
      algorithm: HEADER_ALGORITHM.to_owned(),
    };
    let jheader = to_jwt_segment(&header)?;
    let jclaims = to_jwt_segment(self)?;

    let head_and_claims = format!("{}.{}", jheader, jclaims);
    let sig = kp.sign(head_and_claims.as_bytes()).map_err(Error::Sign)?;
    let sig64 = base64.encode(sig);
    Ok(format!("{}.{}", head_and_claims, sig64))
  }

  pub(crate) fn decode(input: &str) -> Result<Claims<T>> {
    let segments: Vec<&str> = input.split('.').collect();
    if segments.len() != 3 {
      return Err(Error::Token);
    }
    let claims: Claims<T> = from_jwt_segment(segments[1])?;

    Ok(claims)
  }

  /// The name of the module described by the claims
  pub fn name(&self) -> String {
    self.metadata.as_ref().map_or("Anonymous".to_owned(), |md| md.name())
  }
}

fn to_jwt_segment<T: Serialize>(input: &T) -> Result<String> {
  let encoded = serde_json::to_string(input)?;
  Ok(base64.encode(encoded.as_bytes()))
}

fn from_jwt_segment<B: AsRef<str>, T: DeserializeOwned>(encoded: B) -> Result<T> {
  let decoded = base64.decode(encoded.as_ref())?;
  let s = String::from_utf8(decoded).map_err(|_| Error::Utf8("jwt segment".to_owned()))?;

  Ok(serde_json::from_str(&s)?)
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaimsHeader {
  #[serde(rename = "typ")]
  header_type: String,

  #[serde(rename = "alg")]
  algorithm: String,
}

pub trait Named: Clone {
  fn name(&self) -> String;
}

/// A common struct to group related options together.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ClaimsOptions {
  /// Version 0 Claims
  V0(v0::ClaimsOptions),
  /// Version 1 Claims
  V1(v1::ClaimsOptions),
}

impl ClaimsOptions {
  /// Create a new v0 claims.
  #[must_use]
  pub const fn v0(
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
  pub const fn v1(version: Option<String>, expires_in_days: Option<u64>, not_before_days: Option<u64>) -> Self {
    Self::V1(v1::ClaimsOptions {
      version,
      expires_in_days,
      not_before_days,
    })
  }

  /// Get when the claims expire in days
  #[must_use]
  pub const fn expires_in_days(&self) -> Option<u64> {
    match self {
      Self::V0(opts) => opts.expires_in_days,
      Self::V1(opts) => opts.expires_in_days,
    }
  }

  /// Get when the claims are valid (in days)
  #[must_use]
  pub const fn not_before_days(&self) -> Option<u64> {
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

/// The result of the validation process perform on a JWT
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TokenValidation {
  /// Indicates whether or not this token has expired, as determined by the current OS system clock.
  /// If `true`, you should treat the associated token as invalid
  pub expired: bool,
  /// Indicates whether this token is _not yet_ valid. If `true`, do not use this token
  pub cannot_use_yet: bool,
  /// A human-friendly (lowercase) description of the _relative_ expiration date (e.g. "in 3 hours").
  /// If the token never expires, the value will be "never"
  pub expires_human: String,
  /// A human-friendly description of the relative time when this token will become valid (e.g. "in 2 weeks").
  /// If the token has not had a "not before" date set, the value will be "immediately"
  pub not_before_human: String,
  /// Indicates whether the signature is valid according to a cryptographic comparison. If `false` you should
  /// reject this token.
  pub signature_valid: bool,
}

/// Extract the claims embedded in a WebAssembly module.
pub fn extract_claims<T: AsRef<[u8]>>(contents: T) -> Result<Option<Token<WickComponent>>> {
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
      module_hash: String::new(),
      tags: Some(Vec::new()),
      interface,
      ver: options.version(),
    }),
  }
}

#[allow(clippy::too_many_arguments)]
/// Sign WebAssembly bytes with the passed claims.
pub fn sign_buffer_with_claims<T: AsRef<[u8]>>(
  buf: T,
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

fn days_from_now_to_jwt_time(stamp: Option<u64>) -> Option<u64> {
  stamp.map(|e| since_the_epoch().as_secs() + e * 86400)
}
