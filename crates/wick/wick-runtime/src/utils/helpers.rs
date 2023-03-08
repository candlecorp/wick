use seeded_random::{Random, Seed};

use crate::dev::prelude::*;

#[must_use]
pub(crate) fn new_seed() -> Seed {
  Random::new().seed()
}
