use std::collections::HashMap;

/// The set of validated privileges and permissions for a component.
#[derive(Debug, Default, Clone, derive_builder::Builder, property::Property, PartialEq)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[builder(setter(into))]

pub struct Permissions {
  /// A map of directories (TO -> FROM) to expose to the component.
  #[builder(default)]
  pub(crate) dirs: HashMap<String, String>,
}

impl Permissions {
  /// Create a new permissions object
  #[must_use]
  pub fn new(dirs: HashMap<String, String>) -> Self {
    Self { dirs }
  }
}
