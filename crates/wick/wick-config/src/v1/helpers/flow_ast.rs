use std::str::FromStr;

use flow_expression_parser::ast::{self, InstancePort, InstanceTarget};

use crate::error::ManifestError;
use crate::utils::VecTryMapInto;
use crate::v1;

type Result<T> = std::result::Result<T, ManifestError>;

impl TryFrom<ast::FlowExpression> for v1::FlowExpression {
  type Error = ManifestError;

  fn try_from(value: ast::FlowExpression) -> Result<Self> {
    match value {
      ast::FlowExpression::ConnectionExpression(c) => Ok(Self::ConnectionDefinition((*c).try_into()?)),
      ast::FlowExpression::BlockExpression(c) => Ok(Self::BlockExpression(c.try_into()?)),
    }
  }
}

impl TryFrom<ast::BlockExpression> for v1::BlockExpression {
  type Error = ManifestError;

  fn try_from(value: ast::BlockExpression) -> Result<Self> {
    let expressions = value.into_parts();
    Ok(Self {
      expressions: expressions.try_map_into()?,
    })
  }
}

impl TryFrom<ast::ConnectionExpression> for v1::ConnectionDefinition {
  type Error = ManifestError;

  fn try_from(value: ast::ConnectionExpression) -> Result<Self> {
    let (from, to) = value.into_parts();
    Ok(Self {
      from: from.try_into()?,
      to: to.try_into()?,
    })
  }
}

impl TryFrom<ast::ConnectionTargetExpression> for v1::ConnectionTargetDefinition {
  type Error = ManifestError;

  fn try_from(value: ast::ConnectionTargetExpression) -> Result<Self> {
    let (instance, port, data) = value.into_parts();
    Ok(Self {
      data,
      instance: instance.to_string(),
      port: port.to_option_string(),
    })
  }
}

impl TryFrom<v1::FlowExpression> for ast::FlowExpression {
  type Error = ManifestError;

  fn try_from(expr: v1::FlowExpression) -> Result<Self> {
    Ok(match expr {
      v1::FlowExpression::ConnectionDefinition(v) => ast::FlowExpression::connection(v.try_into()?),
      v1::FlowExpression::BlockExpression(v) => ast::FlowExpression::block(v.try_into()?),
    })
  }
}

impl TryFrom<v1::ConnectionDefinition> for ast::ConnectionExpression {
  type Error = ManifestError;

  fn try_from(expr: v1::ConnectionDefinition) -> Result<Self> {
    Ok(Self::new(expr.from.try_into()?, expr.to.try_into()?))
  }
}

impl TryFrom<v1::BlockExpression> for ast::BlockExpression {
  type Error = ManifestError;

  fn try_from(value: v1::BlockExpression) -> Result<Self> {
    Ok(Self::new(value.expressions.try_map_into()?))
  }
}

impl TryFrom<&crate::v1::ConnectionDefinition> for ast::ConnectionExpression {
  type Error = ManifestError;

  fn try_from(def: &crate::v1::ConnectionDefinition) -> Result<Self> {
    let from: ast::ConnectionTargetExpression = def.from.clone().try_into()?;
    let to: ast::ConnectionTargetExpression = def.to.clone().try_into()?;
    Ok(ast::ConnectionExpression::new(from, to))
  }
}

impl TryFrom<crate::v1::ConnectionTargetDefinition> for ast::ConnectionTargetExpression {
  type Error = ManifestError;

  fn try_from(def: crate::v1::ConnectionTargetDefinition) -> Result<Self> {
    Ok(ast::ConnectionTargetExpression::new_data(
      InstanceTarget::from_str(&def.instance)?,
      def
        .port
        .map_or(Ok(InstancePort::None), |p| InstancePort::from_str(&p))?,
      def.data,
    ))
  }
}
