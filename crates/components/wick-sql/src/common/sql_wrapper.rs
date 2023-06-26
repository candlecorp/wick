use wick_packet::TypeWrapper;

#[derive(Debug, Clone)]
pub(crate) struct SqlWrapper(pub(crate) TypeWrapper);

impl SqlWrapper {
  #[cfg(test)]
  pub(crate) fn decode<T: serde::de::DeserializeOwned>(self) -> Result<T, serde_json::Error> {
    serde_json::from_value(self.0.into_inner())
  }
}

impl From<TypeWrapper> for SqlWrapper {
  fn from(wrapper: TypeWrapper) -> Self {
    Self(wrapper)
  }
}
