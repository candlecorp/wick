use std::fmt::Display;

use wasmrs_guest::*;
mod wick {
  wick_component::wick_import!();
}
use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait(Send))]
impl OpAdd for TestComponent {
  async fn add(mut left: WickStream<u64>, mut right: WickStream<u64>, mut outputs: OpAddOutputs) -> wick::Result<()> {
    while let (Some(Ok(left)), Some(Ok(right))) = (left.next().await, right.next().await) {
      outputs.output.send(left + right);
    }
    outputs.output.done();
    Ok(())
  }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait(Send))]
impl OpError for TestComponent {
  async fn error(mut input: WickStream<String>, _outputs: OpErrorOutputs) -> wick::Result<()> {
    while let Some(Ok(_)) = input.next().await {
      panic!("This component always panics");
    }
    Ok(())
  }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait(Send))]
impl OpValidate for TestComponent {
  async fn validate(mut input: WickStream<String>, mut outputs: OpValidateOutputs) -> Result<()> {
    while let Some(Ok(password)) = input.next().await {
      println!("Checking password {}", password);

      if password.len() < MINIMUM_LENGTH {
        println!("Too short!");
        outputs.output.error(LengthError::TooShort.to_string());
      } else if password.len() > MAXIMUM_LENGTH {
        println!("Too long!!");
        outputs.output.error(LengthError::TooLong.to_string());
      } else {
        println!("Just right!");
        outputs.output.send(password);
      }
    }
    outputs.output.done();
    Ok(())
  }
}

#[derive(Debug)]
enum LengthError {
  TooShort,
  TooLong,
}

impl Display for LengthError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        LengthError::TooShort => format!("Needs to be longer than {} characters", MINIMUM_LENGTH),
        LengthError::TooLong => format!("Needs to be shorter than {} characters", MAXIMUM_LENGTH),
      }
    )
  }
}

impl std::error::Error for LengthError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    Some(self)
  }
}

static MINIMUM_LENGTH: usize = 8;
static MAXIMUM_LENGTH: usize = 512;
