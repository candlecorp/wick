use super::bindings::BoundIdentifier;

#[derive(Debug, Clone, PartialEq, derive_builder::Builder, property::Property, serde::Serialize)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[builder(setter(into))]

/// Volumes to expose to a component and the internal paths they map to.
pub struct ExposedVolume {
  /// The resource ID of the volume.
  pub(crate) resource: BoundIdentifier,
  /// The path to map it to in the component.
  pub(crate) path: String,
}
