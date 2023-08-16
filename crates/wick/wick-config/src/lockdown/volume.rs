#![allow(dead_code)]
use std::collections::HashSet;
use std::path::PathBuf;

use normpath::PathExt;
use wildmatch::WildMatch;

use super::{FailureKind, LockdownError};
use crate::audit::AuditedVolume;
use crate::config::resources::ResourceKind;
use crate::config::VolumeRestriction;

pub(crate) fn validate<'a>(
  component_id: &str,
  resource_id: &str,
  resource: &AuditedVolume,
  restrictions: impl Iterator<Item = &'a VolumeRestriction>,
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
  resource_id: &str,
  resource: &AuditedVolume,
  restriction: &VolumeRestriction,
) -> Result<(), FailureKind> {
  // Does this restriction include this component ID? If not, then we don't have access.
  if !restriction
    .components()
    .iter()
    .any(|c| WildMatch::new(c).matches(component_id))
  {
    return Err(FailureKind::NotExpresslyAllowed(
      component_id.to_owned(),
      ResourceKind::Volume,
    ));
  }

  // Can we reconcile the volume path? If not, then we don't have access.
  let path = match resource
    .path
    .normalize()
    .map_err(|_| crate::error::ManifestError::FileNotFound(resource.path.to_string_lossy().to_string()))
  {
    Ok(path) => path,
    Err(e) => {
      tracing::error!("error reconciling volume path: {}", e);
      return Err(FailureKind::VolumeInvalid(
        resource_id.to_owned(),
        resource.path.to_string_lossy().to_string(),
      ));
    }
  };

  // If our template configuration is unrendered, there's a bug. Panic.
  let Some(restriction_value) = restriction.allow().value() else {
    panic!("volume restriction's allow template is unrendered");
  };

  // Is the restriction value a single '*'? If so, then we have access.
  if restriction_value == "*" {
    return Ok(());
  }

  let restriction_value = match PathBuf::from(restriction_value).normalize() {
    Ok(path) => path.into_path_buf(),
    Err(e) => {
      tracing::error!("error normalizing volume restriction path: {}", e);
      return Err(FailureKind::VolumeRestrictionInvalid(restriction_value.clone()));
    }
  };

  // Otherwise, we need to check if the volume path matches the restriction path.
  // The path has been normalized at this point, so we can do a simple starts_with.
  if path.starts_with(restriction_value) {
    Ok(())
  } else {
    Err(FailureKind::Volume(
      component_id.to_owned(),
      path.into_path_buf().to_string_lossy().to_string(),
    ))
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use wick_logger::LoggingOptions;

  use super::*;
  use crate::config::template_config::Renderable;

  fn resolve(path: &str) -> String {
    PathBuf::from(path)
      .normalize()
      .unwrap()
      .into_path_buf()
      .to_string_lossy()
      .to_string()
  }

  fn ensure_temp_subdir(path: &str) -> String {
    let path = PathBuf::from("/tmp").join(path);
    std::fs::create_dir_all(&path).unwrap();
    path.to_string_lossy().to_string()
  }

  #[rstest::rstest]
  #[case("test", "/etc", (["test"], "*"), )]
  #[case("test", "/tmp", (["test"], "/tmp"), )]
  #[case("test", &ensure_temp_subdir("subdir"), (["test"], "/tmp"), )]
  fn test_allowed<const K: usize>(
    #[case] component_id: &str,
    #[case] desired_path: &str,
    #[case] restriction: ([&str; K], &str),
  ) -> Result<()> {
    let _ = wick_logger::init_test(&LoggingOptions::with_level(wick_logger::LogLevel::Trace));
    let volume = AuditedVolume {
      path: desired_path.into(),
    };
    let mut restriction = VolumeRestriction::new_from_template(
      restriction.0.into_iter().map(Into::into).collect::<Vec<_>>(),
      restriction.1,
    );
    restriction.render_config(None, None)?;
    assert_eq!(
      is_allowed(component_id, "VOLUME_ID", &volume, &restriction),
      Ok(()),
      "component {} should have access to {}",
      component_id,
      desired_path
    );

    Ok(())
  }

  #[rstest::rstest]
  #[case("test", "/etc", (["test"], "/home/user/app"), FailureKind::VolumeRestrictionInvalid("/home/user/app".into()))]
  #[case("test", "/etc", (["test"], "/DOES NOT EXIST"), FailureKind::VolumeRestrictionInvalid("/DOES NOT EXIST".into()))]
  #[case("test", "/tmp/../etc", ([], "/tmp"), FailureKind::NotExpresslyAllowed("test".into(), ResourceKind::Volume))]
  #[case("test", "/tmp/../etc", (["test"], "/tmp"), FailureKind::Volume("test".into(), resolve("/tmp/../etc")))]
  fn test_restricted<const K: usize>(
    #[case] component_id: &str,
    #[case] desired_path: &str,
    #[case] restriction: ([&str; K], &str),
    #[case] failure: FailureKind,
  ) -> Result<()> {
    let _ = wick_logger::init_test(&LoggingOptions::with_level(wick_logger::LogLevel::Trace));
    let env: std::collections::HashMap<String, String> = std::env::vars().collect();
    let volume = AuditedVolume {
      path: desired_path.into(),
    };
    let mut restriction = VolumeRestriction::new_from_template(
      restriction.0.into_iter().map(Into::into).collect::<Vec<_>>(),
      restriction.1,
    );
    restriction.render_config(None, Some(&env))?;
    assert_eq!(
      is_allowed(component_id, "VOLUME_ID", &volume, &restriction),
      Err(failure),
      "component {} should not have access to {}",
      component_id,
      desired_path
    );

    Ok(())
  }
}
