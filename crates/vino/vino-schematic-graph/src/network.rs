use crate::Schematic;

#[derive(Debug)]
pub struct Network {
  pub name: String,
  schematics: Vec<Schematic>,
}

impl Network {
  pub fn new<T: AsRef<str>>(name: T) -> Self {
    Self {
      name: name.as_ref().to_owned(),
      schematics: Default::default(),
    }
  }
  pub fn add_schematic(&mut self, schematic: Schematic) {
    self.schematics.push(schematic);
  }
  #[must_use]
  pub fn schematic(&self, name: &str) -> Option<&Schematic> {
    self.schematics.iter().find(|s| s.name() == name)
  }
  #[must_use]
  pub fn schematics(&self) -> &[Schematic] {
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

  #[test_logger::test]
  fn test_sync_send() -> Result<()> {
    sync_send::<Network>();
    Ok(())
  }
}
