#[derive(Debug, Clone, PartialEq, property::Property)]
#[property(get(public), set(private), mut(disable))]
/// A reference to a component by id.
pub struct ComponentReference {
  pub(crate) id: String,
}
