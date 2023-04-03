use std::collections::HashMap;

use wick_config::config::Metadata;

pub(crate) const VERSION: &str = "org.opencontainers.image.version";
pub(crate) const ICON: &str = "dev.candle.wick.package.icon";
pub(crate) const _TYPE: &str = "dev.candle.wick.type";
pub(crate) const AUTHORS: &str = "org.opencontainers.image.authors";
pub(crate) const VENDORS: &str = "org.opencontainers.image.vendors";
pub(crate) const DESCRIPTION: &str = "org.opencontainers.image.description";
pub(crate) const DOCUMENTATION: &str = "org.opencontainers.image.documentation";
pub(crate) const LICENSES: &str = "org.opencontainers.image.licenses";
pub(crate) const TITLE: &str = "org.opencontainers.image.title";

#[derive(Debug, Clone)]
pub(crate) struct Annotations(HashMap<String, String>);

impl Annotations {
  pub(crate) fn inner(&self) -> &HashMap<String, String> {
    &self.0
  }
}

impl From<Metadata> for Annotations {
  fn from(metadata: Metadata) -> Self {
    let mut map = HashMap::new();

    map.insert(VERSION.to_owned(), metadata.version.clone());

    if !metadata.authors.is_empty() {
      map.insert(AUTHORS.to_owned(), metadata.authors.join(", "));
    }

    if !metadata.vendors.is_empty() {
      map.insert(VENDORS.to_owned(), metadata.vendors.join(", "));
    }

    if let Some(description) = &metadata.description {
      map.insert(DESCRIPTION.to_owned(), description.clone());
    }

    if let Some(documentation) = &metadata.documentation {
      map.insert(DOCUMENTATION.to_owned(), documentation.clone());
    }

    if !metadata.licenses.is_empty() {
      map.insert(LICENSES.to_owned(), metadata.licenses.join(", "));
    }

    map.insert(
      ICON.to_owned(),
      metadata
        .icon
        .map(|v| v.path().map(|v| v.to_string()).unwrap_or_default())
        .unwrap_or_default(),
    );

    Self(map)
  }
}
