use std::collections::HashMap;

#[derive(Debug, Default, Clone, PartialEq)]
/// Metadata for the component or application.
pub struct Metadata {
  /// The version of the component or application.
  pub version: String,
  /// The authors of the component or application.
  pub authors: Vec<String>,
  /// Any vendors associated with the component or application.
  pub vendors: Vec<String>,
  /// A short description of the component or application.
  pub description: Option<String>,
  /// Where to find documentation for the component or application.
  pub documentation: Option<String>,
  /// The license(s) for the component or application.
  pub licenses: Vec<String>,
  /// The icon for the component or application.
  pub icon: String,
}

impl From<&Metadata> for HashMap<String, String> {
  /// Return the metadata of the component.
  #[must_use]
  fn from(metadata: &Metadata) -> Self {
    let mut map = HashMap::new();

    map.insert("version".to_owned(), metadata.version.clone());

    if !metadata.authors.is_empty() {
      map.insert("authors".to_owned(), metadata.authors.join(", "));
    }

    if !metadata.vendors.is_empty() {
      map.insert("vendors".to_owned(), metadata.vendors.join(", "));
    }

    if let Some(description) = &metadata.description {
      map.insert("description".to_owned(), description.clone());
    }

    if let Some(documentation) = &metadata.documentation {
      map.insert("documentation".to_owned(), documentation.clone());
    }

    if !metadata.licenses.is_empty() {
      map.insert("licenses".to_owned(), metadata.licenses.join(", "));
    }

    map.insert("icon".to_owned(), metadata.icon.clone());

    map
  }
}
