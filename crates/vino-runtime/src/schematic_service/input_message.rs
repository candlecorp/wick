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
        MessageTransport::Exception(msg) => make_default_transport(default, &msg),
        MessageTransport::Error(msg) => make_default_transport(default, &msg),
        MessageTransport::Invalid => make_default_transport(default, "Internal Error: 7801"),
        MessageTransport::Test(_) => make_default_transport(default, "Internal Error: 7804"),
        MessageTransport::Signal(_) => make_default_transport(default, "Internal Error: 7805"),
        MessageTransport::MessagePack(_) => self.payload,
        MessageTransport::Success(_) => self.payload,
        MessageTransport::Json(_) => self.payload,
      },
      None => self.payload,
    };

    InputMessage { payload, ..self }
  }
}
