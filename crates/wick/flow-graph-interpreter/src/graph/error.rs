use flow_expression_parser::ast::InstanceTarget;
use flow_graph::NodePort;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Error {
  #[error(transparent)]
  CoreError(#[from] flow_graph::error::Error),
  #[error("Missing downstream '{0}'")]
  MissingDownstream(String),
  #[error("Could not find node named '{0}'")]
  NodeNotFound(String),
  #[error("Could not infer what downstream port '{from}->{to }' should connect to, known ports are {}", .known_ports.join(", "))]
  PortInferenceDown {
    from: String,
    to: InstanceTarget,
    known_ports: Vec<String>,
  },
  #[error("Could not infer what upstream port '{from }->{to}' should connect to, known ports are {}", .known_ports.join(", "))]
  PortInferenceUp {
    to: String,
    from: InstanceTarget,
    known_ports: Vec<String>,
  },
  #[error("Could not find signature for operation '{operation}' on component '{component}', available operations are: {}", .available.join(", "))]
  MissingOperation {
    component: String,
    operation: String,
    available: Vec<String>,
  },
  #[error("Could not render operation config for '{0}': {1}")]
  Config(String, String),
  #[error("Invalid config for core operation '{0}': {1}")]
  CoreOperation(String, String),
}

impl Error {
  pub(crate) fn missing_downstream<T: AsRef<str>>(port: T) -> Self {
    Error::MissingDownstream(port.as_ref().to_owned())
  }
  pub(crate) fn node_not_found(node: impl std::fmt::Display) -> Self {
    Error::NodeNotFound(node.to_string())
  }
  pub(crate) fn port_inference_down<T: AsRef<str>>(
    from: &InstanceTarget,
    from_port: T,
    to: InstanceTarget,
    known_ports: &[NodePort],
  ) -> Self {
    Error::PortInferenceDown {
      from: format!("{}.{}", from, from_port.as_ref()),
      to,
      known_ports: known_ports.iter().map(|p| p.name().to_owned()).collect(),
    }
  }
  pub(crate) fn port_inference_up<T: AsRef<str>>(
    to: &InstanceTarget,
    to_port: T,
    from: InstanceTarget,
    known_ports: &[NodePort],
  ) -> Self {
    Error::PortInferenceUp {
      to: format!("{}.{}", to, to_port.as_ref()),
      from,
      known_ports: known_ports.iter().map(|p| p.name().to_owned()).collect(),
    }
  }

  pub(crate) fn missing_operation<T: AsRef<str>, J: AsRef<str>>(
    component: T,
    operation: J,
    available: &[&str],
  ) -> Self {
    Error::MissingOperation {
      component: component.as_ref().to_owned(),
      operation: operation.as_ref().to_owned(),
      available: available.iter().map(|s| (*s).to_owned()).collect(),
    }
  }

  pub(crate) fn config<T: AsRef<str>, J: AsRef<str>>(id: T, err: J) -> Self {
    Error::Config(id.as_ref().to_owned(), err.as_ref().to_owned())
  }
  pub(crate) fn core_operation<T: AsRef<str>, J: AsRef<str>>(id: T, err: J) -> Self {
    Error::CoreOperation(id.as_ref().to_owned(), err.as_ref().to_owned())
  }
}
