use crate::config::AssetReference;

#[derive(Debug, Default, Builder, Clone, PartialEq, derive_asset_container::AssetManager)]
#[asset(asset(AssetReference))]
/// Metadata for the component or application.
pub struct Metadata {
  /// The version of the component or application.
  #[asset(skip)]
  pub version: String,
  /// The authors of the component or application.
  #[asset(skip)]
  #[builder(default)]
  pub authors: Vec<String>,
  /// Any vendors associated with the component or application.
  #[asset(skip)]
  #[builder(default)]
  pub vendors: Vec<String>,
  /// A short description of the component or application.
  #[asset(skip)]
  #[builder(default)]
  pub description: Option<String>,
  /// Where to find documentation for the component or application.
  #[asset(skip)]
  #[builder(default)]
  pub documentation: Option<String>,
  /// The license(s) for the component or application.
  #[asset(skip)]
  #[builder(default)]
  pub licenses: Vec<String>,
  /// The icon for the component or application.
  #[builder(default)]
  pub icon: Option<AssetReference>,
}

impl Metadata {
  pub fn new(version: impl AsRef<str>) -> Self {
    Self {
      version: version.as_ref().to_owned(),
      ..Default::default()
    }
  }
}
