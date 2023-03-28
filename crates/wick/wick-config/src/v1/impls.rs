use super::ComponentDefinition;

impl ComponentDefinition {
  pub(crate) fn component_id(&self) -> Option<&str> {
    match self {
      ComponentDefinition::WasmRsComponent(_) => None,
      ComponentDefinition::GrpcUrlComponent(_) => None,
      ComponentDefinition::ManifestComponent(_) => None,
      ComponentDefinition::ComponentReference(v) => Some(&v.id),
    }
  }
}
