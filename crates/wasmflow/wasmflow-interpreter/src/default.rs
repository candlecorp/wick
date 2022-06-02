use std::borrow::Cow;

use wasmflow_manifest::process_default;
use wasmflow_transport::{MessageTransport, Serialized};

pub(crate) fn make_default_transport(json: &serde_json::Value, message: &str) -> MessageTransport {
  process_default(Cow::Borrowed(json), message).map_or(
    MessageTransport::error("Error processing default value"),
    |result| {
      wasmflow_codec::messagepack::serialize(&result)
        .map_or(MessageTransport::error("Error serializing default value"), |bytes| {
          MessageTransport::Success(Serialized::MessagePack(bytes))
        })
    },
  )
}
