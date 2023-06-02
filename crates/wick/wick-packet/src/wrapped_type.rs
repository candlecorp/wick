use serde_json::Value;
use wick_interface_types::Type;

#[derive(Debug, Clone)]
#[must_use]
pub struct TypeWrapper(Type, Value);

impl TypeWrapper {
  /// Create a new TypeWrapper.
  pub fn new(ty: Type, val: Value) -> Self {
    Self(ty, val)
  }

  pub fn type_signature(&self) -> &Type {
    &self.0
  }

  #[must_use]
  pub fn into_inner(self) -> Value {
    self.1
  }

  #[must_use]
  pub fn inner(&self) -> &Value {
    &self.1
  }
}
