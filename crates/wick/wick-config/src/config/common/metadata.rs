use crate::config::AssetReference;

#[derive(Debug, Default, Builder, Clone, PartialEq, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(AssetReference))]
/// Metadata for the component or application.
pub struct Metadata {
  /// The version of the component or application.
  #[asset(skip)]
  pub(crate) version: String,
  /// The authors of the component or application.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) authors: Vec<String>,
  /// Any vendors associated with the component or application.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) vendors: Vec<String>,
  /// A short description of the component or application.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) description: Option<String>,
  /// Where to find documentation for the component or application.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) documentation: Option<String>,
  /// The license(s) for the component or application.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) licenses: Vec<String>,
  /// The icon for the component or application.
  #[builder(default)]
  pub(crate) icon: Option<AssetReference>,
}

impl Metadata {
  pub fn new(version: impl AsRef<str>) -> Self {
    Self {
      version: version.as_ref().to_owned(),
      ..Default::default()
    }
  }
}
