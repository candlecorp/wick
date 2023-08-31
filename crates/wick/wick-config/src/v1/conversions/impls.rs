use crate::v1::ComponentDefinition;

impl ComponentDefinition {
  pub(crate) fn component_id(&self) -> Option<&str> {
    match self {
      ComponentDefinition::GrpcUrlComponent(_) => None,
      ComponentDefinition::ManifestComponent(_) => None,
      ComponentDefinition::ComponentReference(v) => Some(&v.id),
      ComponentDefinition::SqlComponent(_) => todo!(),
      ComponentDefinition::HttpClientComponent(_) => todo!(),
      ComponentDefinition::WebSocketClientComponent(_) => todo!(),
    }
  }
}
