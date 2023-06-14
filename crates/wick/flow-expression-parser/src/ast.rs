use std::collections::HashMap;
use std::str::FromStr;

use serde_json::Value;

use crate::{parse, Error};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[must_use]
/// A node instance
pub enum InstanceTarget {
  /// A flow input node.
  Input,
  /// A flow output node.
  Output,
  /// A black hole for inputs.
  Null(Option<String>),
  /// A reserved namespace for built-in nodes.
  Core,
  /// An unspecified node.
  Default,
  #[doc(hidden)]
  Link,
  /// A named node instance.
  Named(String),
  /// An instance created inline.
  Path(String, Option<String>),
}

impl InstanceTarget {
  /// Returns [self] unless self is [InstanceTarget::Default], in which case it returns [other].
  pub fn or(self, other: InstanceTarget) -> InstanceTarget {
    match self {
      InstanceTarget::Default => other,
      _ => self,
    }
  }

  /// Get the id of the instance target.
  #[must_use]
  pub fn id(&self) -> Option<&str> {
    match self {
      InstanceTarget::Input => Some(parse::SCHEMATIC_INPUT),
      InstanceTarget::Output => Some(parse::SCHEMATIC_OUTPUT),
      InstanceTarget::Null(id) => id.as_deref(),
      InstanceTarget::Core => Some(parse::CORE_ID),
      InstanceTarget::Default => panic!("Cannot get id of default instance"),
      InstanceTarget::Link => Some(parse::NS_LINK),
      InstanceTarget::Named(name) => Some(name),
      InstanceTarget::Path(_, id) => id.as_deref(),
    }
  }

  /// Create a new [InstanceTarget::Named] from a string.
  pub fn named(name: impl AsRef<str>) -> Self {
    Self::Named(name.as_ref().to_owned())
  }

  /// Create a new [InstanceTarget::Path] from a path and id.
  pub fn path(path: impl AsRef<str>, id: impl AsRef<str>) -> Self {
    Self::Path(path.as_ref().to_owned(), Some(id.as_ref().to_owned()))
  }

  /// Create a new [InstanceTarget::Path] from a path without an id.
  pub fn anonymous_path(path: impl AsRef<str>) -> Self {
    Self::Path(path.as_ref().to_owned(), None)
  }
}

impl FromStr for InstanceTarget {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    parse::v1::parse_instance(s)
  }
}

impl std::fmt::Display for InstanceTarget {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      InstanceTarget::Input => f.write_str(parse::SCHEMATIC_INPUT),
      InstanceTarget::Output => f.write_str(parse::SCHEMATIC_OUTPUT),
      InstanceTarget::Null(id) => f.write_str(id.as_deref().unwrap_or(parse::SCHEMATIC_NULL)),
      InstanceTarget::Core => f.write_str(parse::CORE_ID),
      InstanceTarget::Default => f.write_str("<>"),
      InstanceTarget::Link => f.write_str(parse::NS_LINK),
      InstanceTarget::Named(name) => f.write_str(name),
      InstanceTarget::Path(path, id) => match id {
        Some(id) => write!(f, "{}[{}]", path, id),
        None => write!(f, "{}", path),
      },
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[must_use]
/// A port on a node instance, used to connect node instances together.
pub struct ConnectionTarget {
  pub(crate) target: InstanceTarget,
  pub(crate) port: String,
}

impl ConnectionTarget {
  /// Create a new ConnectionTarget.
  pub fn new(target: InstanceTarget, port: impl AsRef<str>) -> Self {
    Self {
      target,
      port: port.as_ref().to_owned(),
    }
  }

  /// Get the target port
  #[must_use]
  pub fn port(&self) -> &str {
    &self.port
  }

  /// Get the target instance
  pub fn target(&self) -> &InstanceTarget {
    &self.target
  }
}

impl std::fmt::Display for ConnectionTarget {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}.{}", self.target, self.port)
  }
}

/// A connection between two targets.
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub struct ConnectionExpression {
  from: ConnectionTargetExpression,
  to: ConnectionTargetExpression,
}

impl std::fmt::Display for ConnectionExpression {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} -> {}", self.from, self.to)
  }
}

impl ConnectionExpression {
  /// Create a new [ConnectionExpression] from two [ConnectionTargetExpression]s.
  pub fn new(mut from: ConnectionTargetExpression, mut to: ConnectionTargetExpression) -> Self {
    from.instance = from.instance.or(InstanceTarget::Input);
    to.instance = to.instance.or(InstanceTarget::Output);

    Self { from, to }
  }

  /// Get the owned parts of the connection.
  #[must_use]
  pub fn into_parts(self) -> (ConnectionTargetExpression, ConnectionTargetExpression) {
    (self.from, self.to)
  }

  /// Get the from target.
  #[must_use]
  pub fn from(&self) -> &ConnectionTargetExpression {
    &self.from
  }

  /// Get the from target.
  #[must_use]
  pub fn from_mut(&mut self) -> &mut ConnectionTargetExpression {
    &mut self.from
  }

  /// Get the to target.
  #[must_use]
  pub fn to(&self) -> &ConnectionTargetExpression {
    &self.to
  }

  /// Get the to target.
  #[must_use]
  pub fn to_mut(&mut self) -> &mut ConnectionTargetExpression {
    &mut self.to
  }
}

#[derive(Debug, Clone, PartialEq)]
/// A flow expression.
pub enum FlowExpression {
  /// A [ConnectionExpression].
  ConnectionExpression(Box<ConnectionExpression>),
  /// A [BlockExpression].
  BlockExpression(BlockExpression),
}

impl FromStr for FlowExpression {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let (_, v) = crate::parse::v1::flow_expression(s).map_err(|e| Error::FlowExpressionParse(e.to_string()))?;
    Ok(v)
  }
}

impl FlowExpression {
  /// Get the expression as a ConnectionExpression.
  #[must_use]
  pub fn as_connection(&self) -> Option<&ConnectionExpression> {
    match self {
      FlowExpression::ConnectionExpression(expr) => Some(expr),
      _ => None,
    }
  }

  #[must_use]
  /// Make a new [FlowExpression::ConnectionExpression] from a [ConnectionExpression].
  pub fn connection(expr: ConnectionExpression) -> Self {
    FlowExpression::ConnectionExpression(Box::new(expr))
  }

  #[must_use]
  /// Make a new [FlowExpression::BlockExpression] from a [BlockExpression].
  pub fn block(expr: BlockExpression) -> Self {
    FlowExpression::BlockExpression(expr)
  }
}

#[derive(Debug, Clone, PartialEq)]
/// A block expression.
pub struct BlockExpression {
  expressions: Vec<FlowExpression>,
}

impl BlockExpression {
  /// Create a new [BlockExpression] from a vector of [FlowExpression]s.
  #[must_use]
  pub fn new(expressions: Vec<FlowExpression>) -> Self {
    Self { expressions }
  }

  /// Get the owned parts of the block expression.
  #[must_use]
  pub fn into_parts(self) -> Vec<FlowExpression> {
    self.expressions
  }

  /// Get the expressions in the block as a mutable iterator.
  pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut FlowExpression> {
    self.expressions.iter_mut()
  }

  /// Get the expressions in the block as an iterator.
  pub fn iter(&self) -> impl Iterator<Item = &FlowExpression> {
    self.expressions.iter()
  }
}

#[derive(Debug, Clone, PartialEq)]
/// A flow program.
pub struct FlowProgram {
  expressions: Vec<FlowExpression>,
}

impl FlowProgram {
  /// Create a new [FlowProgram] from a list of [FlowExpression]s.
  #[must_use]
  pub fn new(expressions: Vec<FlowExpression>) -> Self {
    Self { expressions }
  }

  /// Get the owned parts of the flow program.
  #[must_use]
  pub fn into_parts(self) -> Vec<FlowExpression> {
    self.expressions
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The port associated with an instance in a connection.
pub enum InstancePort {
  /// A simple, named port.
  Named(String),
  /// A named port with a path to an inner value.
  Path(String, Vec<String>),
}

impl FromStr for InstancePort {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let (_, v) = crate::parse::v1::instance_port(s).map_err(|_e| Error::PortSyntax(s.to_owned()))?;
    Ok(v)
  }
}

impl std::fmt::Display for InstancePort {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      InstancePort::Named(name) => write!(f, "{}", name),
      InstancePort::Path(name, path) => write!(
        f,
        "{}.{}",
        name,
        path.iter().map(|v| format!("\"{}\"", v)).collect::<Vec<_>>().join(".")
      ),
    }
  }
}

impl InstancePort {
  /// Quickly create a [InstancePort::Named] variant.
  #[must_use]
  pub fn named(name: impl AsRef<str>) -> Self {
    Self::Named(name.as_ref().to_owned())
  }

  /// Quickly create a [InstancePort::Path] variant.
  #[must_use]
  pub fn path(name: impl AsRef<str>, path: Vec<String>) -> Self {
    Self::Path(name.as_ref().to_owned(), path)
  }

  /// Get the name of the port.
  #[must_use]
  pub fn name(&self) -> &String {
    match self {
      InstancePort::Named(name) => name,
      InstancePort::Path(name, _) => name,
    }
  }
}

/// A connection target, specified by an instance and a port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionTargetExpression {
  instance: InstanceTarget,
  port: InstancePort,
  data: Option<HashMap<String, Value>>,
}

impl std::fmt::Display for ConnectionTargetExpression {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    #[allow(clippy::option_if_let_else)]
    if let Some(_data) = &self.data {
      // TODO: Implement data syntax. There's no way of stringifying this yet.
      Err(std::fmt::Error::default())
    } else {
      write!(f, "{}.{}", self.instance, self.port)
    }
  }
}

impl ConnectionTargetExpression {
  /// Create a new [ConnectionTargetExpression]
  pub fn new(instance: InstanceTarget, port: impl Into<InstancePort>) -> Self {
    Self {
      instance,
      port: port.into(),
      data: None,
    }
  }

  /// Create a new [ConnectionTargetExpression] with default data
  pub fn new_default(
    instance: InstanceTarget,
    port: impl Into<InstancePort>,
    data: Option<HashMap<String, Value>>,
  ) -> Self {
    Self {
      instance,
      port: port.into(),
      data,
    }
  }

  /// Get the instance target.
  pub fn instance(&self) -> &InstanceTarget {
    &self.instance
  }

  /// Get the instance target.
  pub fn instance_mut(&mut self) -> &mut InstanceTarget {
    &mut self.instance
  }

  /// Get the port.
  #[must_use]
  pub fn port(&self) -> &InstancePort {
    &self.port
  }

  /// Get the data.
  #[must_use]
  pub fn data(&self) -> Option<&HashMap<String, Value>> {
    self.data.as_ref()
  }

  /// Get the owned parts of the connection target.
  pub fn into_parts(self) -> (InstanceTarget, InstancePort, Option<HashMap<String, Value>>) {
    (self.instance, self.port, self.data)
  }
}
