use thiserror::Error;

// type BoxedSyncSendError = Box<dyn std::error::Error + Sync + std::marker::Send>;

/// Error type for the flow expression parser.
#[derive(Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ParserError {
  /// Component id is not a fully qualified name with a namespace.
  #[error("Component id '{0}' is not a fully qualified name with a namespace")]
  ComponentIdError(String),

  /// Default was requested when none present.
  #[error("Invalid connection target syntax: '{0}': {1}")]
  ConnectionTargetSyntax(String, String),

  /// Default was requested when none present.
  #[error("Invalid connection definition syntax: '{0}'")]
  ConnectionDefinitionSyntax(String),

  /// Whatever was passed in as an operation's port isn't valid.
  #[error("Invalid input/output port syntax: '{0}'")]
  PortSyntax(String),

  /// Ambiguous reference in connection shorthand.
  #[error("No suitable default found for port in : {0}")]
  NoDefaultPort(String),

  /// Ambiguous port in connection shorthand.
  #[error("No suitable default found for reference in : {0}")]
  NoDefaultReference(String),

  /// Error parsing or serializing Sender data.
  #[error("Error parsing or serializing Sender data: {0}")]
  InvalidSenderData(String),

  /// Error occurred parsing a flow expression.
  #[error("Could not parse string into a FlowExpression '{0}'")]
  FlowExpressionParse(String),
}
