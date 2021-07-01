use vino_rpc::PortSignature;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ComponentModel {
  /// The fully qualified name, including the namespace.
  pub(crate) id: String,
  /// The name of the component
  pub(crate) name: String,
  pub(crate) inputs: Vec<PortSignature>,
  pub(crate) outputs: Vec<PortSignature>,
}
