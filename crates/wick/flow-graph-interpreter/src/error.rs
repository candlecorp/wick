pub use crate::interpreter::error::Error as InterpreterError;
pub use crate::interpreter::program::validator::error::{SchematicInvalid, ValidationError};

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;
  use crate::interpreter::error::StateError;
  use crate::interpreter::executor::error::ExecutionError;

  fn sync_send<T>()
  where
    T: Sync + Send,
  {
  }

  #[test]
  fn test_sync_send() -> Result<()> {
    sync_send::<InterpreterError>();
    sync_send::<SchematicInvalid>();
    sync_send::<ValidationError>();
    sync_send::<ExecutionError>();
    sync_send::<InterpreterError>();
    sync_send::<StateError>();

    Ok(())
  }
}
