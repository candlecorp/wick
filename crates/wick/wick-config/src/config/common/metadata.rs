#![allow(missing_docs)] // delete when we move away from the `property` crate.
use crate::config::AssetReference;

#[derive(
  Debug,
  Default,
  derive_builder::Builder,
  Clone,
  PartialEq,
  derive_asset_container::AssetManager,
  property::Property,
  serde::Serialize,
)]
#[property(get(public), set(private), mut(disable))]
#[builder(setter(into))]
#[asset(asset(AssetReference))]
/// Metadata for the component or application.
pub struct Metadata {
  /// The version of the component or application.
  #[asset(skip)]
  pub(crate) version: String,
  /// The authors of the component or application.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) authors: Vec<String>,
  /// Any vendors associated with the component or application.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) vendors: Vec<String>,
  /// A short description of the component or application.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) description: Option<String>,
  /// Where to find documentation for the component or application.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) documentation: Option<String>,
  /// The license(s) for the component or application.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) licenses: Vec<String>,
  /// The icon for the component or application.
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) icon: Option<AssetReference>,
}

impl Metadata {
  /// Create a new [Metadata] instance from a version string.
  pub fn new(version: impl AsRef<str>) -> Self {
    Self {
      version: version.as_ref().to_owned(),
      ..Default::default()
    }
  }
}
