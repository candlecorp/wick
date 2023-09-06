use std::collections::HashMap;
use std::str::FromStr;

use liquid_json::LiquidJsonValue;
use once_cell::sync::Lazy;
use parking_lot::Mutex;

use crate::{parse, Error};

pub(crate) static RNG: Lazy<Mutex<seeded_random::Random>> = Lazy::new(|| Mutex::new(seeded_random::Random::new()));

/// Set the seed for the RNG.
///
/// The RNG is most commonly used for generating UUIDs for anonymous nodes.
pub fn set_seed(seed: u64) {
  *RNG.lock() = seeded_random::Random::from_seed(seeded_random::Seed::unsafe_new(seed));
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
#[must_use]
#[allow(clippy::exhaustive_enums)]
#[serde(rename_all = "kebab-case")]
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
  Path {
    /// The path to the operation, i.e. namespace::operation
    path: String,
    /// The optional ID to use for this instance.
    id: TargetId,
  },
}

impl InstanceTarget {
  /// Returns [self] unless self is [InstanceTarget::Default], in which case it returns `other`.
  #[allow(clippy::missing_const_for_fn)]
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
      InstanceTarget::Path { id, .. } => id.to_opt_str(),
    }
  }

  /// Create a new [InstanceTarget::Named] from a string.
  pub fn named<T: AsRef<str>>(name: T) -> Self {
    Self::Named(name.as_ref().to_owned())
  }

  /// Create a new [InstanceTarget::Path] from a path and id.
  pub(crate) fn path<T: AsRef<str>, I: AsRef<str>>(path: T, id: I) -> Self {
    Self::Path {
      path: path.as_ref().to_owned(),
      id: TargetId::Named(id.as_ref().to_owned()),
    }
  }

  /// Create a new [InstanceTarget::Path] from a path without an id.
  pub(crate) fn anonymous_path<T: AsRef<str>>(path: T) -> Self {
    Self::Path {
      path: path.as_ref().to_owned(),
      id: TargetId::None,
    }
  }

  /// Create a new [InstanceTarget::Path] from a path without an id.
  #[cfg(test)]
  pub(crate) fn generated_path<T: AsRef<str>>(path: T) -> Self {
    Self::Path {
      path: path.as_ref().to_owned(),
      id: TargetId::new_generated(&path.as_ref().replace("::", "_")),
    }
  }

  pub(crate) fn ensure_id(&mut self) {
    if let InstanceTarget::Path { id, path } = self {
      id.ensure_id(&path.replace("::", "_"));
    }
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
      InstanceTarget::Path { path, id } => write!(f, "{}{}", path, id.as_inline_id()),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
/// [TargetId] differentiates between user-provided IDs, generated IDs, and no IDs.
#[allow(clippy::exhaustive_enums)]
#[serde(into = "Option<String>")]
pub enum TargetId {
  /// An automatically generated ID
  Generated(String),
  /// No ID provided
  None,
  /// A user-provided ID
  Named(String),
}

fn generate_id(prefix: &str) -> String {
  format!(
    "{}_{}",
    prefix,
    RNG.lock().uuid().to_string().split_once('-').unwrap().0
  )
}

impl TargetId {
  #[cfg(test)]
  #[must_use]
  pub fn new_generated(prefix: &str) -> Self {
    TargetId::Generated(generate_id(prefix))
  }

  /// Convert the [TargetId] to an Option<String>.
  #[must_use]
  pub fn to_opt_str(&self) -> Option<&str> {
    match self {
      TargetId::Generated(id) => Some(id),
      TargetId::None => None,
      TargetId::Named(name) => Some(name),
    }
  }

  /// Turn the [TargetId] into something that can be appended as an inline ID.
  #[must_use]
  pub fn as_inline_id(&self) -> String {
    match self {
      TargetId::Generated(id) => format!("[{}]", id),
      TargetId::None => String::new(),
      TargetId::Named(name) => format!("[{}]", name),
    }
  }

  fn ensure_id(&mut self, prefix: &str) {
    if *self == TargetId::None {
      *self = TargetId::Generated(generate_id(prefix));
    }
  }

  /// Replace the [TargetId] with a [TargetId::Named] variant.
  pub fn replace<T: AsRef<str>>(&mut self, value: T) {
    *self = TargetId::Named(value.as_ref().to_owned());
  }
}

impl From<TargetId> for Option<String> {
  fn from(value: TargetId) -> Self {
    value.to_opt_str().map(ToOwned::to_owned)
  }
}

/// A connection between two targets.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
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
  #[allow(clippy::missing_const_for_fn)]
  pub fn into_parts(self) -> (ConnectionTargetExpression, ConnectionTargetExpression) {
    (self.from, self.to)
  }

  /// Get the from target.
  #[must_use]
  pub const fn from(&self) -> &ConnectionTargetExpression {
    &self.from
  }

  /// Get the from target.
  #[must_use]
  pub fn from_mut(&mut self) -> &mut ConnectionTargetExpression {
    &mut self.from
  }

  /// Get the to target.
  #[must_use]
  pub const fn to(&self) -> &ConnectionTargetExpression {
    &self.to
  }

  /// Get the to target.
  #[must_use]
  pub fn to_mut(&mut self) -> &mut ConnectionTargetExpression {
    &mut self.to
  }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
/// A flow expression.
#[allow(clippy::exhaustive_enums)]
#[serde(untagged)]
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
  /// Get the expression as a [ConnectionExpression].
  #[must_use]
  pub fn as_connection(&self) -> Option<&ConnectionExpression> {
    match self {
      FlowExpression::ConnectionExpression(expr) => Some(expr),
      _ => None,
    }
  }

  /// Get the expression as a [BlockExpression].
  #[must_use]
  pub const fn as_block(&self) -> Option<&BlockExpression> {
    match self {
      FlowExpression::BlockExpression(expr) => Some(expr),
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
  pub const fn block(expr: BlockExpression) -> Self {
    FlowExpression::BlockExpression(expr)
  }

  /// Convenience method to replace the expression with a new one.
  pub fn replace(&mut self, expr: Self) {
    *self = expr;
  }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
/// A block expression.
pub struct BlockExpression {
  #[serde(skip_serializing_if = "Vec::is_empty")]
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
  #[allow(clippy::missing_const_for_fn)]
  pub fn into_parts(self) -> Vec<FlowExpression> {
    self.expressions
  }

  /// Get a list of the inner expressions.
  #[must_use]
  pub fn inner(&self) -> &[FlowExpression] {
    &self.expressions
  }

  /// Get a mutable list of the inner expressions.
  #[must_use]
  pub fn inner_mut(&mut self) -> &mut Vec<FlowExpression> {
    &mut self.expressions
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

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
/// A flow program.
pub struct FlowProgram {
  #[serde(skip_serializing_if = "Vec::is_empty")]
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
  #[allow(clippy::missing_const_for_fn)]
  pub fn into_parts(self) -> Vec<FlowExpression> {
    self.expressions
  }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[allow(clippy::exhaustive_enums)]
#[serde(untagged)]
/// The port associated with an instance in a connection.
pub enum InstancePort {
  /// A simple, named port.
  Named(String),
  /// A named port with a path to an inner value.
  Path(String, Vec<String>),
  /// An unnamed port that must be inferred or it's an error.
  None,
}

impl From<&str> for InstancePort {
  fn from(s: &str) -> Self {
    match s {
      "" => Self::None,
      _ => Self::Named(s.to_owned()),
    }
  }
}

impl From<&String> for InstancePort {
  fn from(s: &String) -> Self {
    s.as_str().into()
  }
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
      InstancePort::None => f.write_str("<none>"),
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
  pub fn named<T: AsRef<str>>(name: T) -> Self {
    Self::Named(name.as_ref().to_owned())
  }

  /// Quickly create a [InstancePort::Path] variant.
  #[must_use]
  pub fn path<T:AsRef<str>>(name: T, path: Vec<String>) -> Self {
    Self::Path(name.as_ref().to_owned(), path)
  }

  /// Get the name of the port.
  #[must_use]
  pub fn name(&self) -> Option<&str> {
    match self {
      InstancePort::Named(name) => Some(name),
      InstancePort::Path(name, _) => Some(name),
      InstancePort::None => None,
    }
  }

  /// Convert the [InstancePort] to an Option<String> representing the (optional) parseable value.
  #[must_use]
  pub fn to_option_string(&self) -> Option<String> {
    match self {
      InstancePort::Named(_) => Some(self.to_string()),
      InstancePort::Path(_, _) => Some(self.to_string()),
      InstancePort::None => None,
    }
  }
}

/// A connection target, specified by an instance and a port.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ConnectionTargetExpression {
  instance: InstanceTarget,
  port: InstancePort,
  #[serde(skip_serializing_if = "Option::is_none")]
  data: Option<HashMap<String, LiquidJsonValue>>,
}

impl std::fmt::Display for ConnectionTargetExpression {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    #[allow(clippy::option_if_let_else)]
    if let Some(_data) = &self.data {
      // TODO: Implement data syntax. There's no way of stringifying this yet.
      Err(std::fmt::Error)
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
  pub fn new_data(
    instance: InstanceTarget,
    port: impl Into<InstancePort>,
    data: Option<HashMap<String, LiquidJsonValue>>,
  ) -> Self {
    Self {
      instance,
      port: port.into(),
      data,
    }
  }

  /// Get the instance target.
  pub const fn instance(&self) -> &InstanceTarget {
    &self.instance
  }

  /// Get the instance target.
  pub fn instance_mut(&mut self) -> &mut InstanceTarget {
    &mut self.instance
  }

  /// Get the port.
  #[must_use]
  pub const fn port(&self) -> &InstancePort {
    &self.port
  }

  /// Get the data.
  #[must_use]
  pub const fn data(&self) -> Option<&HashMap<String, LiquidJsonValue>> {
    self.data.as_ref()
  }

  /// Get the owned parts of the connection target.
  #[allow(clippy::missing_const_for_fn)]
  pub fn into_parts(self) -> (InstanceTarget, InstancePort, Option<HashMap<String, LiquidJsonValue>>) {
    (self.instance, self.port, self.data)
  }
}
