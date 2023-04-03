use std::collections::HashMap;

/// Wick package version annotation string.
pub const VERSION: &str = "org.opencontainers.image.version";
/// Wick package icon annotation string.
pub const ICON: &str = "dev.candle.wick.package.icon";
/// Wick package type annotation string.
pub const _TYPE: &str = "dev.candle.wick.type";
/// Wick package authors annotation string.
pub const AUTHORS: &str = "org.opencontainers.image.authors";
/// Wick package vendors annotation string.
pub const VENDORS: &str = "org.opencontainers.image.vendors";
/// Wick package description annotation string.
pub const DESCRIPTION: &str = "org.opencontainers.image.description";
/// Wick package documentation annotation string.
pub const DOCUMENTATION: &str = "org.opencontainers.image.documentation";
/// Wick package licenses annotation string.
pub const LICENSES: &str = "org.opencontainers.image.licenses";
/// Wick package title annotation string.
pub const TITLE: &str = "org.opencontainers.image.title";

#[derive(Debug, Clone)]
/// Annotation object for Wick packages.
#[must_use]
pub struct Annotations(HashMap<String, String>);

impl Annotations {
  #[allow(unused)]
  pub(crate) fn inner(&self) -> &HashMap<String, String> {
    &self.0
  }

  /// Create a new annotation object from a map.
  pub fn new(map: HashMap<String, String>) -> Self {
    Self(map)
  }
}
