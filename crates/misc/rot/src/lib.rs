mod rot;
pub use anyhow::{Error, Result};
pub extern crate anyhow;
pub extern crate k9;
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() -> Result<()> {
    assert_equal!(3, 3, "2 != 3");
    Ok(())
  }
}
