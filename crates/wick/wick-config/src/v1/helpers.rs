use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct LocationReference(
  #[serde(deserialize_with = "serde_with_expand_env::with_expand_envs")] pub(super) String,
);
