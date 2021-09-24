use serde::{
  Deserialize,
  Serialize,
};

use crate::dev::prelude::*;
use crate::schematic_service::default::make_default_transport;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputMessage {
  pub tx_id: String,
  pub connection: ConnectionDefinition,
  pub payload: MessageTransport,
}

impl InputMessage {
  pub fn handle_default(self) -> Self {
    let payload = match &self.connection.default {
      Some(default) => match self.payload {
        MessageTransport::Success(_) => self.payload,
        MessageTransport::Failure(failure) => match failure {
          Failure::Invalid => make_default_transport(default, "Internal Error: 7801"),
          Failure::Exception(msg) => make_default_transport(default, &msg),
          Failure::Error(msg) => make_default_transport(default, &msg),
        },
        MessageTransport::Signal(_) => self.payload,
      },
      None => self.payload,
    };

    InputMessage { payload, ..self }
  }
}
