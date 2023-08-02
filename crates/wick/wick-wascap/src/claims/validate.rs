use std::time::{Duration, SystemTime};

use base64::Engine;
use nkeys::KeyPair;
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::{Claims, ClaimsHeader, Named, TokenValidation, HEADER_ALGORITHM, HEADER_TYPE};
use crate::{base64, Error};

type Result<T> = std::result::Result<T, Error>;

/// Validates a signed JWT. This will check the signature, expiration time, and not-valid-before time
pub fn validate_token<T>(input: &str) -> Result<TokenValidation>
where
  T: Serialize + DeserializeOwned + Named,
{
  let segments: Vec<&str> = input.split('.').collect();
  let header_and_claims = format!("{}.{}", segments[0], segments[1]);
  let sig = base64.decode(segments[2])?;

  let header: ClaimsHeader = from_jwt_segment(segments[0])?;
  validate_header(&header)?;

  let claims = Claims::<T>::decode(input)?;
  validate_issuer(&claims.issuer)?;
  validate_subject(&claims.subject)?;

  let kp = KeyPair::from_public_key(&claims.issuer)?;
  let sigverify = kp.verify(header_and_claims.as_bytes(), &sig);

  let validation = TokenValidation {
    signature_valid: sigverify.is_ok(),
    expired: validate_expiration(claims.expires).is_err(),
    expires_human: stamp_to_human(claims.expires).unwrap_or_else(|| "never".to_owned()),
    not_before_human: stamp_to_human(claims.not_before).unwrap_or_else(|| "immediately".to_owned()),
    cannot_use_yet: validate_notbefore(claims.not_before).is_err(),
  };

  Ok(validation)
}

fn since_the_epoch() -> Duration {
  let start = SystemTime::now();
  start.duration_since(std::time::UNIX_EPOCH).unwrap()
}

fn validate_notbefore(nb: Option<u64>) -> Result<()> {
  nb.map_or_else(
    || Ok(()),
    |nbf| {
      let nbf_secs = Duration::from_secs(nbf);
      if since_the_epoch() < nbf_secs {
        Err(Error::TokenTooEarly)
      } else {
        Ok(())
      }
    },
  )
}

fn validate_expiration(exp: Option<u64>) -> Result<()> {
  exp.map_or_else(
    || Ok(()),
    |exp| {
      let exp_secs = Duration::from_secs(exp);
      if exp_secs < since_the_epoch() {
        Err(Error::ExpiredToken)
      } else {
        Ok(())
      }
    },
  )
}

fn validate_issuer(iss: &str) -> Result<()> {
  if iss.is_empty() {
    Err(Error::MissingIssuer)
  } else {
    Ok(())
  }
}

fn validate_subject(sub: &str) -> Result<()> {
  if sub.is_empty() {
    Err(Error::MissingSubject)
  } else {
    Ok(())
  }
}

fn validate_header(h: &ClaimsHeader) -> Result<()> {
  if h.algorithm != HEADER_ALGORITHM {
    Err(Error::InvalidAlgorithm)
  } else if h.header_type != HEADER_TYPE {
    Err(Error::Token)
  } else {
    Ok(())
  }
}

fn from_jwt_segment<B: AsRef<str>, T: DeserializeOwned>(encoded: B) -> Result<T> {
  let decoded = base64.decode(encoded.as_ref())?;
  let s = String::from_utf8(decoded).map_err(|_| Error::Utf8("jwt segment".to_owned()))?;

  Ok(serde_json::from_str(&s)?)
}

fn stamp_to_human(stamp: Option<u64>) -> Option<String> {
  stamp.map(|s| {
    let now = since_the_epoch().as_secs() as i64;
    let diff_sec = (now - (s as i64)).abs();

    // calculate roundoff
    let diff_sec = if diff_sec >= 86400 {
      // round to days
      diff_sec - (diff_sec % 86400)
    } else if diff_sec >= 3600 {
      // round to hours
      diff_sec - (diff_sec % 3600)
    } else if diff_sec >= 60 {
      // round to minutes
      diff_sec - (diff_sec % 60)
    } else {
      diff_sec
    };
    let ht = humantime::format_duration(Duration::from_secs(diff_sec as u64));

    if now as u64 > s {
      format!("{} ago", ht)
    } else {
      format!("in {}", ht)
    }
  })
}
