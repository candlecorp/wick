use std::collections::HashSet;

use wildmatch::WildMatch;

use super::{FailureKind, LockdownError};
use crate::audit::AuditedUrl;
use crate::config::resources::ResourceKind;
use crate::config::UrlRestriction;

pub(crate) fn validate<'a>(
  component_id: &str,
  resource_id: &str,
  resource: &AuditedUrl,
  restrictions: impl Iterator<Item = &'a UrlRestriction>,
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
  resource: &AuditedUrl,
  restriction: &UrlRestriction,
) -> Result<(), FailureKind> {
  // Does this restriction include this component ID? If not, then we don't have access.
  if !restriction
    .components()
    .iter()
    .any(|c| WildMatch::new(c).matches(component_id))
  {
    return Err(FailureKind::NotExpresslyAllowed(
      component_id.to_owned(),
      ResourceKind::Url,
    ));
  }

  // If our template configuration is unrendered, there's a bug. Panic.
  let Some(url_restriction) = restriction.allow.value() else {
    panic!("url restriction's allow template is unrendered");
  };

  if !WildMatch::new(url_restriction).matches(resource.url.as_str()) {
    return Err(FailureKind::Url(component_id.to_owned(), resource.url.to_string()));
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
  #[case("test", "https://google.com",(["test"], "https://google.com/"), )]
  #[case("test", "https://google.com",(["test"], "https://*.com/"), )]
  #[case("test", "https://google.com",(["test"], "*"), )]
  #[case("test", "https://google.com",(["test"], "https://*.com/"), )]
  #[case("test", "https://user:foo@google.com",(["test"], "https://google.com/"), )]
  fn test_allowed<const K: usize>(
    #[case] component_id: &str,
    #[case] desired_url: &str,
    #[case] restriction: ([&str; K], &str),
  ) -> Result<()> {
    let _ = wick_logger::init_test(&LoggingOptions::with_level(wick_logger::LogLevel::Trace));
    let volume: AuditedUrl = desired_url.parse()?;
    let mut restriction = UrlRestriction::new_from_template(
      restriction.0.into_iter().map(Into::into).collect::<Vec<_>>(),
      restriction.1,
    );
    restriction.render_config(None, None)?;
    assert_eq!(
      is_allowed(component_id, "ID", &volume, &restriction),
      Ok(()),
      "component {} should have access to {}",
      component_id,
      desired_url,
    );

    Ok(())
  }

  #[rstest::rstest]
  #[case("test", "https://user:foo@google.com", (["test"], "http://google.com"), FailureKind::Url("test".into(), "https://google.com/".into()))]
  fn test_restricted<const K: usize>(
    #[case] component_id: &str,
    #[case] desired_url: &str,
    #[case] restriction: ([&str; K], &str),
    #[case] failure: FailureKind,
  ) -> Result<()> {
    let _ = wick_logger::init_test(&LoggingOptions::with_level(wick_logger::LogLevel::Trace));
    let volume: AuditedUrl = desired_url.parse()?;
    let mut restriction = UrlRestriction::new_from_template(
      restriction.0.into_iter().map(Into::into).collect::<Vec<_>>(),
      restriction.1,
    );
    restriction.render_config(None, None)?;
    assert_eq!(
      is_allowed(component_id, "ID", &volume, &restriction),
      Err(failure),
      "component {} should not have access to {}",
      component_id,
      desired_url,
    );

    Ok(())
  }
}
