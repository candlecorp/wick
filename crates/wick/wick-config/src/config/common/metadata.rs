use crate::config::AssetReference;

#[derive(Debug, Default, Clone, PartialEq, derive_assets::AssetManager)]
#[asset(AssetReference)]
/// Metadata for the component or application.
pub struct Metadata {
  /// The version of the component or application.
  #[asset(skip)]
  pub version: String,
  /// The authors of the component or application.
  #[asset(skip)]
  pub authors: Vec<String>,
  /// Any vendors associated with the component or application.
  #[asset(skip)]
  pub vendors: Vec<String>,
  /// A short description of the component or application.
  #[asset(skip)]
  pub description: Option<String>,
  /// Where to find documentation for the component or application.
  #[asset(skip)]
  pub documentation: Option<String>,
  /// The license(s) for the component or application.
  #[asset(skip)]
  pub licenses: Vec<String>,
  /// The icon for the component or application.
  pub icon: Option<AssetReference>,
}
