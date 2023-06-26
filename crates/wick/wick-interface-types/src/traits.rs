/// A trait for objects that contain or can generate operation signatures.
pub trait OperationSignatures {
  /// Get a list of operations hosted by the implementer.
  fn operation_signatures(&self) -> Vec<crate::OperationSignature>;

  /// Get an operation signature by name.
  fn get_operation_signature(&self, name: &str) -> Option<crate::OperationSignature> {
    self.operation_signatures().into_iter().find(|o| o.name == name)
  }
}
