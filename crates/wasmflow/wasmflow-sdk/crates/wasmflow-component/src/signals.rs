use std::str::FromStr;

#[allow(missing_debug_implementations, missing_copy_implementations)]
#[must_use]
/// The [OutputSignal] enum is a way of combining port output and messaging signal into one message. Used for WASM modules to reduce the number of calls between the host and guest.
pub enum OutputSignal {
  /// A single output.
  Output,
  /// An output and a done signal.
  OutputDone,
  /// A done signal.
  Done,
}

impl OutputSignal {
  #[must_use]
  #[doc(hidden)]
  pub fn as_str(&self) -> &'static str {
    match self {
      OutputSignal::Output => "1",
      OutputSignal::OutputDone => "2",
      OutputSignal::Done => "3",
    }
  }
}

impl FromStr for OutputSignal {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let result = match s {
      "1" => OutputSignal::Output,
      "2" => OutputSignal::OutputDone,
      "3" => OutputSignal::Done,
      _ => return Err(()),
    };
    Ok(result)
  }
}

#[allow(missing_debug_implementations, missing_copy_implementations)]
#[must_use]
/// The [HostCommand] enum tells the host what to do for the host call.
pub enum HostCommand {
  /// Port output.
  Output,
  /// Make a call to a linked entity.
  LinkCall,
  /// Logging output.
  Log,
}

impl HostCommand {
  #[must_use]
  #[doc(hidden)]
  pub fn as_str(&self) -> &'static str {
    match self {
      HostCommand::Output => "0",
      HostCommand::LinkCall => "1",
      HostCommand::Log => "2",
    }
  }
}

impl FromStr for HostCommand {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let result = match s {
      "0" => HostCommand::Output,
      "1" => HostCommand::LinkCall,
      "2" => HostCommand::Log,
      _ => return Err(()),
    };
    Ok(result)
  }
}

#[allow(missing_debug_implementations, missing_copy_implementations)]
#[must_use]
/// The [LogLevel] enum defines log levels for WASM modules to log appropriately to hosts.
pub enum LogLevel {
  /// Information-related messages
  Info,
  /// Error output
  Error,
  /// Non-fatal warnings
  Warn,
  /// Debug messages
  Debug,
  /// Trace-level messages
  Trace,
  /// Performance mark messages
  Mark,
}

impl LogLevel {
  #[must_use]
  #[doc(hidden)]
  pub fn as_str(&self) -> &'static str {
    match self {
      LogLevel::Info => "0",
      LogLevel::Error => "1",
      LogLevel::Warn => "2",
      LogLevel::Debug => "3",
      LogLevel::Trace => "4",
      LogLevel::Mark => "5",
    }
  }
}

impl FromStr for LogLevel {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let result = match s {
      "0" => LogLevel::Info,
      "1" => LogLevel::Error,
      "2" => LogLevel::Warn,
      "3" => LogLevel::Debug,
      "4" => LogLevel::Trace,
      "5" => LogLevel::Mark,
      _ => return Err(()),
    };
    Ok(result)
  }
}
