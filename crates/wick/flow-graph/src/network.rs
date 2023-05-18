use crate::util::AsStr;
use crate::Schematic;

#[derive(Debug)]
pub struct Network<DATA> {
  pub name: String,
  schematics: Vec<Schematic<DATA>>,
}

impl<DATA> Network<DATA>
where
  DATA: Clone,
{
  pub fn new<T: AsStr>(name: T) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      schematics: Default::default(),
    }
  }
  pub fn add_schematic(&mut self, schematic: Schematic<DATA>) {
    self.schematics.push(schematic);
  }
  #[must_use]
  pub fn schematic(&self, name: &str) -> Option<&Schematic<DATA>> {
    self.schematics.iter().find(|s| s.name() == name)
  }
  #[must_use]
  pub fn schematics(&self) -> &[Schematic<DATA>] {
    &self.schematics
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;

  fn sync_send<T>()
  where
    T: Sync + Send,
  {
  }

  #[test]
  fn test_sync_send() -> Result<()> {
    sync_send::<Network<Option<()>>>();
    Ok(())
  }
}
