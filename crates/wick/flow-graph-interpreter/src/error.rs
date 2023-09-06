pub use crate::interpreter::error::Error as InterpreterError;
pub use crate::interpreter::program::validator::error::{OperationInvalid, ValidationError};

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;
  use crate::interpreter::error::StateError;
  use crate::interpreter::executor::error::ExecutionError;

  const fn sync_send<T>()
  where
    T: Sync + Send,
  {
  }

  #[test]
  const fn test_sync_send() -> Result<()> {
    sync_send::<InterpreterError>();
    sync_send::<OperationInvalid>();
    sync_send::<ValidationError>();
    sync_send::<ExecutionError>();
    sync_send::<InterpreterError>();
    sync_send::<StateError>();

    Ok(())
  }
}
