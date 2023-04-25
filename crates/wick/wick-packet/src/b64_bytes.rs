use std::cmp;
use std::ops::Deref;

use base64_serde::base64_serde_type;
use bytes::buf::IntoIter;
base64_serde_type!(Base64UrlSafe, base64::engine::general_purpose::URL_SAFE);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Base64Bytes(#[serde(with = "Base64UrlSafe")] pub bytes::Bytes);

impl Base64Bytes {
  pub fn new<T>(value: T) -> Self
  where
    T: Into<bytes::Bytes>,
  {
    Self(value.into())
  }
}

impl AsRef<[u8]> for Base64Bytes {
  fn as_ref(&self) -> &[u8] {
    self.0.as_ref()
  }
}

impl From<bytes::Bytes> for Base64Bytes {
  fn from(value: bytes::Bytes) -> Self {
    Self(value)
  }
}

impl From<Base64Bytes> for bytes::Bytes {
  fn from(value: Base64Bytes) -> Self {
    value.0
  }
}

impl Deref for Base64Bytes {
  type Target = [u8];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl std::borrow::Borrow<[u8]> for Base64Bytes {
  fn borrow(&self) -> &[u8] {
    &self.0
  }
}

impl From<Base64Bytes> for Vec<u8> {
  fn from(bytes: Base64Bytes) -> Vec<u8> {
    bytes.0.into()
  }
}
impl From<Vec<u8>> for Base64Bytes {
  fn from(vec: Vec<u8>) -> Base64Bytes {
    Base64Bytes::new(vec)
  }
}

impl Default for Base64Bytes {
  #[inline]
  fn default() -> Base64Bytes {
    Base64Bytes::new(bytes::Bytes::default())
  }
}

impl PartialEq for Base64Bytes {
  fn eq(&self, other: &Base64Bytes) -> bool {
    self.0 == other.0
  }
}

impl PartialOrd for Base64Bytes {
  fn partial_cmp(&self, other: &Base64Bytes) -> Option<cmp::Ordering> {
    self.0.partial_cmp(&other.0)
  }
}

impl Ord for Base64Bytes {
  fn cmp(&self, other: &Base64Bytes) -> cmp::Ordering {
    self.0.cmp(&other.0)
  }
}

impl Eq for Base64Bytes {}

impl PartialEq<[u8]> for Base64Bytes {
  fn eq(&self, other: &[u8]) -> bool {
    self.0 == other
  }
}

impl PartialOrd<[u8]> for Base64Bytes {
  fn partial_cmp(&self, other: &[u8]) -> Option<cmp::Ordering> {
    self.0.partial_cmp(other)
  }
}

impl PartialEq<Base64Bytes> for [u8] {
  fn eq(&self, other: &Base64Bytes) -> bool {
    *other == *self
  }
}

impl PartialOrd<Base64Bytes> for [u8] {
  fn partial_cmp(&self, other: &Base64Bytes) -> Option<cmp::Ordering> {
    <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
  }
}

impl PartialEq<str> for Base64Bytes {
  fn eq(&self, other: &str) -> bool {
    self.0 == other.as_bytes()
  }
}

impl PartialOrd<str> for Base64Bytes {
  fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
    self.0.partial_cmp(other.as_bytes())
  }
}

impl PartialEq<Base64Bytes> for str {
  fn eq(&self, other: &Base64Bytes) -> bool {
    *other == *self
  }
}

impl PartialOrd<Base64Bytes> for str {
  fn partial_cmp(&self, other: &Base64Bytes) -> Option<cmp::Ordering> {
    <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
  }
}

impl PartialEq<Vec<u8>> for Base64Bytes {
  fn eq(&self, other: &Vec<u8>) -> bool {
    *self == other[..]
  }
}

impl PartialOrd<Vec<u8>> for Base64Bytes {
  fn partial_cmp(&self, other: &Vec<u8>) -> Option<cmp::Ordering> {
    self.0.partial_cmp(&other[..])
  }
}

impl PartialEq<Base64Bytes> for Vec<u8> {
  fn eq(&self, other: &Base64Bytes) -> bool {
    *other == *self
  }
}

impl PartialOrd<Base64Bytes> for Vec<u8> {
  fn partial_cmp(&self, other: &Base64Bytes) -> Option<cmp::Ordering> {
    <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
  }
}

impl PartialEq<String> for Base64Bytes {
  fn eq(&self, other: &String) -> bool {
    *self == other[..]
  }
}

impl PartialOrd<String> for Base64Bytes {
  fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
    self.0.partial_cmp(other.as_bytes())
  }
}

impl PartialEq<Base64Bytes> for String {
  fn eq(&self, other: &Base64Bytes) -> bool {
    *other == *self
  }
}

impl PartialOrd<Base64Bytes> for String {
  fn partial_cmp(&self, other: &Base64Bytes) -> Option<cmp::Ordering> {
    <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
  }
}

impl PartialEq<Base64Bytes> for &[u8] {
  fn eq(&self, other: &Base64Bytes) -> bool {
    other.0 == *self
  }
}

impl PartialEq<&[u8]> for Base64Bytes {
  fn eq(&self, other: &&[u8]) -> bool {
    *other == self.0
  }
}

impl PartialOrd<Base64Bytes> for &[u8] {
  fn partial_cmp(&self, other: &Base64Bytes) -> Option<cmp::Ordering> {
    <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
  }
}

impl PartialEq<Base64Bytes> for &str {
  fn eq(&self, other: &Base64Bytes) -> bool {
    other.0 == *self
  }
}

impl PartialOrd<Base64Bytes> for &str {
  fn partial_cmp(&self, other: &Base64Bytes) -> Option<cmp::Ordering> {
    <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
  }
}

impl IntoIterator for Base64Bytes {
  type Item = u8;
  type IntoIter = IntoIter<Base64Bytes>;

  fn into_iter(self) -> Self::IntoIter {
    IntoIter::new(self)
  }
}

impl<'a> IntoIterator for &'a Base64Bytes {
  type Item = &'a u8;
  type IntoIter = core::slice::Iter<'a, u8>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.iter()
  }
}

impl FromIterator<u8> for Base64Bytes {
  fn from_iter<T: IntoIterator<Item = u8>>(into_iter: T) -> Self {
    Vec::from_iter(into_iter).into()
  }
}

impl bytes::Buf for Base64Bytes {
  fn remaining(&self) -> usize {
    self.0.remaining()
  }

  fn chunk(&self) -> &[u8] {
    self.0.chunk()
  }

  fn advance(&mut self, cnt: usize) {
    self.0.advance(cnt);
  }
}
