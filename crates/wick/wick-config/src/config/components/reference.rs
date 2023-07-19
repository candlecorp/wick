#![allow(missing_docs)] // delete when we move away from the `property` crate.

#[derive(Debug, Clone, PartialEq, property::Property, serde::Serialize)]
#[property(get(public), set(private), mut(disable))]
/// A reference to a component by id (typically unused by user code)
pub struct ComponentReference {
  /// The id of the component.
  pub(crate) id: String,
}

impl ComponentReference {
  /// Create a new [ComponentReference] with specified id.
  pub fn new(id: impl AsRef<str>) -> Self {
    Self {
      id: id.as_ref().to_owned(),
    }
  }
}
