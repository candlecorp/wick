use wick_packet::{OutputIterator, Port};

/// Trait for operation outputs to handle situations where packets need to be sent to all output streams.
pub trait Broadcast {
  /// Broadcast an open bracket to all output streams.
  fn broadcast_open(&mut self) {
    for output in self.outputs_mut() {
      output.open_bracket();
    }
  }

  /// Broadcast a close bracket to all output streams.
  fn broadcast_close(&mut self) {
    for output in self.outputs_mut() {
      output.close_bracket();
    }
  }

  /// Broadcast a done signal to all output streams.
  fn broadcast_done(&mut self) {
    for output in self.outputs_mut() {
      output.done();
    }
  }

  /// Broadcast an error to all output streams.
  fn broadcast_err(&mut self, err: impl Into<String>) {
    let err = err.into();
    for output in self.outputs_mut() {
      output.error(&err);
    }
  }

  /// Get all output streams.
  fn outputs_mut(&mut self) -> OutputIterator<'_>;
}

/// Trait implemented for output sets with a single output port.
pub trait SingleOutput: Broadcast {
  /// The single output port.
  fn single_output(&mut self) -> &mut dyn Port;
}

impl<T: Port> Broadcast for T {
  fn outputs_mut(&mut self) -> OutputIterator<'_> {
    OutputIterator::new(vec![self])
  }
}
