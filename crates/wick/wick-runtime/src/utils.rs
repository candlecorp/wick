use seeded_random::{Random, Seed};

#[must_use]
pub(crate) fn new_seed() -> Seed {
  Random::new().seed()
}
