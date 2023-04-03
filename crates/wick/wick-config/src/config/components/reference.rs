#[derive(Debug, Clone, PartialEq)]
/// A reference to a component by id.
pub struct ComponentReference {
  pub(crate) id: String,
}

impl ComponentReference {
  /// Get the id of the referenced component.
  #[must_use]
  pub fn id(&self) -> &str {
    &self.id
  }
}
