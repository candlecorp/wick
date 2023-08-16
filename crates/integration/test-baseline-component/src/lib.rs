use std::fmt::Display;

#[cfg(feature = "localgen")]
mod generated;
#[cfg(feature = "localgen")]
use generated as wick;
#[cfg(not(feature = "localgen"))]
mod wick {
  #![allow(unused_imports, missing_debug_implementations, clippy::needless_pass_by_value)]
  wick_component::wick_import!();
}
use wick::*;

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait(Send))]
impl add::Operation for Component {
  type Error = String;
  type Outputs = add::Outputs;
  type Config = add::Config;
  async fn add(
    mut left: WickStream<u64>,
    mut right: WickStream<u64>,
    mut outputs: Self::Outputs,
    _ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    println!("op:add: in add operation, waiting for inputs");
    while let (Some(left), Some(right)) = (left.next().await, right.next().await) {
      println!("op:add: received inputs");
      match (left, right) {
        (Ok(left), Ok(right)) => {
          let output = left + right;
          println!("op:add: sending output");
          outputs.output.send(&output);
        }
        (Err(err), _) | (_, Err(err)) => {
          println!("op:add: received error, propagating forward");
          outputs.output.error(&format!("Error adding numbers: {}", err));
        }
      }
    }
    println!("op:add: done");
    outputs.output.done();
    Ok(())
  }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait(Send))]
impl power::Operation for Component {
  type Error = String;
  type Outputs = power::Outputs;
  type Config = power::Config;

  async fn power(
    mut input: WickStream<u64>,
    mut outputs: Self::Outputs,
    ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    println!("op:power: received exponent {}", ctx.config.exponent);
    while let Some(Ok(input)) = input.next().await {
      let output = input.pow(ctx.config.exponent);
      outputs.output.send(&output);
    }
    outputs.output.done();
    Ok(())
  }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait(Send))]
impl error::Operation for Component {
  type Error = String;
  type Outputs = error::Outputs;
  type Config = error::Config;

  async fn error(
    mut input: WickStream<String>,
    _outputs: Self::Outputs,
    ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    let config = ctx.root_config();

    println!("In error operation");
    while let Some(Ok(_)) = input.next().await {
      println!("Going to panic! This is expected!");
      panic!("This component always panics: {}", config.default_err);
    }
    println!("Returning from error operation without panicking (this is unexpected)");
    Ok(())
  }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait(Send))]
impl uuid::Operation for Component {
  type Error = String;
  type Outputs = uuid::Outputs;
  type Config = uuid::Config;

  async fn uuid(mut outputs: Self::Outputs, ctx: Context<Self::Config>) -> Result<(), Self::Error> {
    let uuid = ctx.inherent.rng.uuid().to_string();
    outputs.output.send(&uuid);
    outputs.output.done();

    Ok(())
  }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait(Send))]
impl validate::Operation for Component {
  type Error = String;
  type Outputs = validate::Outputs;
  type Config = validate::Config;

  async fn validate(
    mut input: WickStream<String>,
    mut outputs: Self::Outputs,
    _ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    while let Some(password) = input.next().await {
      let password = propagate_if_error!(password, outputs, continue);
      println!("Checking password {}", password);

      if password.len() < MINIMUM_LENGTH {
        println!("Too short!");
        outputs.output.error(&LengthError::TooShort.to_string());
      } else if password.len() > MAXIMUM_LENGTH {
        println!("Too long!!");
        outputs.output.error(&LengthError::TooLong.to_string());
      } else {
        println!("Just right!");
        outputs.output.send(&password);
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

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait(Send))]
impl strftime::Operation for Component {
  type Error = String;
  type Outputs = strftime::Outputs;
  type Config = strftime::Config;

  async fn strftime(
    mut input: WickStream<datetime::DateTime>,
    mut outputs: Self::Outputs,
    ctx: Context<Self::Config>,
  ) -> Result<(), Self::Error> {
    let fmt = &ctx.config.format;
    while let Some(input) = input.next().await {
      let input = propagate_if_error!(input, outputs, continue);
      let output = input.format(fmt).to_string();
      outputs.output.send(&output);
    }
    outputs.output.done();
    Ok(())
  }
}
