use regex::Regex;

use crate::v0;

lazy_static::lazy_static! {
    pub(crate) static ref CONNECTION_TARGET_REGEX_V0: Regex = Regex::new(&format!(r"^({}|{}|{}|\w*)(?:\[(\w*)\])?$", DEFAULT_ID, SCHEMATIC_INPUT, SCHEMATIC_OUTPUT)).unwrap();
}

pub(crate) static CONNECTION_SEPARATOR: &str = "=>";

/// The reserved identifier representing an as-of-yet-undetermined default value.
const DEFAULT_ID: &str = "<>";
/// The reserved reference name for schematic input. Used in schematic manifests to denote schematic input.
pub const SCHEMATIC_INPUT: &str = "<input>";
/// The reserved reference name for schematic output. Used in schematic manifests to denote schematic output.
pub const SCHEMATIC_OUTPUT: &str = "<output>";
/// The reserved port name to use when sending an asynchronous error from a component.
pub const COMPONENT_ERROR: &str = "<error>";
/// The reserved namespace for references to internal schematics.
pub const SELF_NAMESPACE: &str = "self";

pub(crate) fn parse_target_v0(s: &str) -> Result<(Option<&str>, Option<&str>), crate::Error> {
    CONNECTION_TARGET_REGEX_V0.captures(s.trim()).map_or_else(
        || Err(crate::Error::ConnectionTargetSyntax(s.to_owned())),
        |captures| {
            Ok((
                captures.get(1).map(|m| m.as_str()),
                captures.get(2).map(|m| m.as_str()),
            ))
        },
    )
}

pub(crate) fn parse_connection_target_v0(
    s: &str,
) -> Result<v0::ConnectionTargetDefinition, crate::Error> {
    let (t_ref, t_port) = parse_target_v0(s)?;
    Ok(v0::ConnectionTargetDefinition {
        reference: t_ref.unwrap_or(DEFAULT_ID).to_owned(),
        port: t_port.unwrap_or(DEFAULT_ID).to_owned(),
    })
}

pub(crate) fn parse_connection_v0(s: &str) -> Result<v0::ConnectionDefinition, crate::Error> {
    let s = s.trim();
    s.split_once(CONNECTION_SEPARATOR).map_or_else(
        || Err(crate::Error::ConnectionDefinitionSyntax(s.to_owned())),
        |(from, to)| {
            let (from_ref, from_port) = parse_target_v0(from)?;
            let (to_ref, to_port) = parse_target_v0(to)?;
            Ok(v0::ConnectionDefinition {
                from: Some(v0::ConnectionTargetDefinition {
                    port: from_port
                        .or(to_port)
                        .ok_or_else(|| crate::Error::NoDefaultPort(s.to_owned()))?
                        .to_owned(),
                    reference: match from_ref {
                        Some(DEFAULT_ID) => SCHEMATIC_INPUT,
                        Some(v) => v,
                        None => return Err(crate::Error::NoDefaultReference(s.to_owned())),
                    }
                    .to_owned(),
                }),
                to: Some(v0::ConnectionTargetDefinition {
                    port: to_port
                        .or(from_port)
                        .ok_or_else(|| crate::Error::NoDefaultPort(s.to_owned()))?
                        .to_owned(),
                    reference: match to_ref {
                        Some(DEFAULT_ID) => SCHEMATIC_OUTPUT,
                        Some(v) => v,
                        None => return Err(crate::Error::NoDefaultReference(s.to_owned())),
                    }
                    .to_owned(),
                }),
                default: None,
            })
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    use anyhow::Result;
    #[test_env_log::test]
    fn test_reserved() -> Result<()> {
        let parsed = parse_target_v0("<input>[foo]")?;
        assert_eq!(parsed, (Some("<input>"), Some("foo")));
        Ok(())
    }

    #[test_env_log::test]
    fn test_basic() -> Result<()> {
        let parsed = parse_target_v0("ref[foo]")?;
        assert_eq!(parsed, (Some("ref"), Some("foo")));
        Ok(())
    }

    #[test_env_log::test]
    fn test_default_with_port() -> Result<()> {
        let parsed = parse_target_v0("<>[foo]")?;
        assert_eq!(parsed, (Some(DEFAULT_ID), Some("foo")));
        Ok(())
    }

    #[test_env_log::test]
    fn test_default() -> Result<()> {
        let parsed = parse_target_v0("<>")?;
        assert_eq!(parsed, (Some(DEFAULT_ID), None));
        Ok(())
    }

    #[test_env_log::test]
    fn test_connection_basic() -> Result<()> {
        let parsed = parse_connection_v0("ref1[in]=>ref2[out]")?;
        assert_eq!(
            parsed,
            v0::ConnectionDefinition {
                from: Some(v0::ConnectionTargetDefinition {
                    reference: "ref1".to_owned(),
                    port: "in".to_owned()
                }),
                to: Some(v0::ConnectionTargetDefinition {
                    reference: "ref2".to_owned(),
                    port: "out".to_owned()
                }),
                default: None
            }
        );
        Ok(())
    }

    #[test_env_log::test]
    fn test_connection_default_input_named_port() -> Result<()> {
        let parsed = parse_connection_v0("<>[in]=>ref2[out]")?;
        assert_eq!(
            parsed,
            v0::ConnectionDefinition {
                from: Some(v0::ConnectionTargetDefinition {
                    reference: SCHEMATIC_INPUT.to_owned(),
                    port: "in".to_owned()
                }),
                to: Some(v0::ConnectionTargetDefinition {
                    reference: "ref2".to_owned(),
                    port: "out".to_owned()
                }),
                default: None
            }
        );
        Ok(())
    }

    #[test_env_log::test]
    fn test_connection_default_output_named_port() -> Result<()> {
        let parsed = parse_connection_v0("ref1[in]=><>[out]")?;
        assert_eq!(
            parsed,
            v0::ConnectionDefinition {
                from: Some(v0::ConnectionTargetDefinition {
                    reference: "ref1".to_owned(),
                    port: "in".to_owned()
                }),
                to: Some(v0::ConnectionTargetDefinition {
                    reference: SCHEMATIC_OUTPUT.to_owned(),
                    port: "out".to_owned()
                }),
                default: None
            }
        );
        Ok(())
    }

    #[test_env_log::test]
    fn test_connection_default_output() -> Result<()> {
        let parsed = parse_connection_v0("ref1[port]=><>")?;
        assert_eq!(
            parsed,
            v0::ConnectionDefinition {
                from: Some(v0::ConnectionTargetDefinition {
                    reference: "ref1".to_owned(),
                    port: "port".to_owned()
                }),
                to: Some(v0::ConnectionTargetDefinition {
                    reference: SCHEMATIC_OUTPUT.to_owned(),
                    port: "port".to_owned()
                }),
                default: None
            }
        );
        Ok(())
    }

    #[test_env_log::test]
    fn test_connection_default_input() -> Result<()> {
        let parsed = parse_connection_v0("<>=>ref1[port]")?;
        assert_eq!(
            parsed,
            v0::ConnectionDefinition {
                from: Some(v0::ConnectionTargetDefinition {
                    reference: SCHEMATIC_INPUT.to_owned(),
                    port: "port".to_owned()
                }),
                to: Some(v0::ConnectionTargetDefinition {
                    reference: "ref1".to_owned(),
                    port: "port".to_owned()
                }),
                default: None
            }
        );
        Ok(())
    }
}
