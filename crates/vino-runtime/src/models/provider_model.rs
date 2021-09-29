use std::collections::HashMap;

use crate::dev::prelude::*;

#[derive(Debug, Clone, Default)]
pub(crate) struct ProviderModel {
  pub(crate) components: HashMap<String, ComponentModel>,
}
