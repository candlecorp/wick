#![allow(dead_code)]
use std::collections::HashSet;

use wildmatch::WildMatch;

use super::{FailureKind, LockdownError};
use crate::audit::AuditedPort;
use crate::config::resources::ResourceKind;
use crate::config::PortRestriction;

pub(crate) fn validate<'a>(
  component_id: &str,
  resource_id: &str,
  resource: &AuditedPort,
  restrictions: impl Iterator<Item = &'a PortRestriction>,
) -> Result<(), LockdownError> {
  let mut failures = HashSet::new();
  for restriction in restrictions {
    match is_allowed(component_id, resource_id, resource, restriction) {
      Ok(_) => return Ok(()),
      Err(e) => {
        failures.insert(e);
      }
    }
  }

  if failures.is_empty() {
    Ok(())
  } else {
    Err(LockdownError::new(failures.into_iter().collect()))
  }
}

pub(crate) fn is_allowed(
  component_id: &str,
  _resource_id: &str,
  resource: &AuditedPort,
  restriction: &PortRestriction,
) -> Result<(), FailureKind> {
  // Does this restriction include this component ID? If not, then we don't have access.
  if !restriction
    .components()
    .iter()
    .any(|c| WildMatch::new(c).matches(component_id))
  {
    return Err(FailureKind::NotExpresslyAllowed(
      component_id.to_owned(),
      ResourceKind::TcpPort,
    ));
  }

  // If our template configuration is unrendered, there's a bug. Panic.
  let (Some(port_restriction), Some(host_restriction)) = (restriction.port().value(), restriction.address().value())
  else {
    panic!("port restriction's allow template is unrendered");
  };

  let port_str = resource.port.to_string();

  if !WildMatch::new(port_restriction).matches(&port_str) {
    return Err(FailureKind::Port(component_id.to_owned(), resource.port));
  }

  if !WildMatch::new(host_restriction).matches(&resource.address) {
    return Err(FailureKind::Address(component_id.to_owned(), resource.address.clone()));
  }

  Ok(())
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use wick_logger::LoggingOptions;

  use super::*;
  use crate::config::template_config::Renderable;

  #[rstest::rstest]
  #[case("test", "0.0.0.0", 80,(["test"], "*","*"), )]
  #[case("test", "0.0.0.0", 80,(["test"], "*","80"), )]
  #[case("test", "0.0.0.0", 80,(["test"], "0.0.0.0","*"), )]
  fn test_allowed<const K: usize>(
    #[case] component_id: &str,
    #[case] desired_address: &str,
    #[case] desired_port: u16,
    #[case] restriction: ([&str; K], &str, &str),
  ) -> Result<()> {
    let _ = wick_logger::init_test(&LoggingOptions::with_level(wick_logger::LogLevel::Trace));
    let volume = AuditedPort {
      port: desired_port,
      address: desired_address.into(),
    };
    let mut restriction = PortRestriction::new_from_template(
      restriction.0.into_iter().map(Into::into).collect::<Vec<_>>(),
      restriction.1,
      restriction.2,
    );
    restriction.render_config(None, None)?;
    assert_eq!(
      is_allowed(component_id, "ID", &volume, &restriction),
      Ok(()),
      "component {} should have access to {}:{}",
      component_id,
      desired_address,
      desired_port,
    );

    Ok(())
  }

  #[rstest::rstest]
  #[case("test", "0.0.0.0", 80, (["test"], "0.0.0.0","8080"), FailureKind::Port("test".into(), 80))]
  #[case("test", "0.0.0.0", 80, (["test"], "127.0.0.1","80"), FailureKind::Address("test".into(), "0.0.0.0".into()))]
  fn test_restricted<const K: usize>(
    #[case] component_id: &str,
    #[case] desired_address: &str,
    #[case] desired_port: u16,
    #[case] restriction: ([&str; K], &str, &str),
    #[case] failure: FailureKind,
  ) -> Result<()> {
    let _ = wick_logger::init_test(&LoggingOptions::with_level(wick_logger::LogLevel::Trace));
    let volume = AuditedPort {
      port: desired_port,
      address: desired_address.into(),
    };
    let mut restriction = PortRestriction::new_from_template(
      restriction.0.into_iter().map(Into::into).collect::<Vec<_>>(),
      restriction.1,
      restriction.2,
    );
    restriction.render_config(None, None)?;
    assert_eq!(
      is_allowed(component_id, "ID", &volume, &restriction),
      Err(failure),
      "component {} should not have access to {}:{}",
      component_id,
      desired_address,
      desired_port
    );

    Ok(())
  }
}
