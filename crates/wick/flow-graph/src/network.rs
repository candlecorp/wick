use crate::Schematic;

#[derive(Debug)]
pub struct Network<DATA> {
  pub name: String,
  schematics: Vec<Schematic<DATA>>,
  data: DATA,
}

impl<DATA> Network<DATA>
where
  DATA: Clone,
{
  pub fn new<T: Into<String>>(name: T, data: DATA) -> Self {
    Self {
      name: name.into(),
      schematics: Default::default(),
      data,
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

  pub const fn data(&self) -> &DATA {
    &self.data
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;

  const fn sync_send<T>()
  where
    T: Sync + Send,
  {
  }

  #[test]
  const fn test_sync_send() -> Result<()> {
    sync_send::<Network<Option<()>>>();
    Ok(())
  }
}
