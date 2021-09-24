use tonic::Status;
use vino_rpc::rpc::message_kind::{
  Data,
  Kind,
  OutputSignal,
};
use vino_rpc::rpc::{
  MessageKind,
  Output,
};
use vino_transport::message_transport::{
  Failure,
  MessageSignal,
  Success,
};
use vino_transport::MessageTransport;

pub(crate) fn make_output(
  port: &str,
  inv_id: &str,
  payload: MessageTransport,
) -> Result<Output, Status> {
  match payload {
    MessageTransport::Success(v) => match v {
      Success::MessagePack(bytes) => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(MessageKind {
          kind: Kind::MessagePack.into(),
          data: Some(Data::Messagepack(bytes)),
        }),
      }),
      Success::Serialized(v) => match serde_json::to_string(&v) {
        Ok(json) => Ok(Output {
          port: port.to_owned(),
          invocation_id: inv_id.to_owned(),
          payload: Some(MessageKind {
            kind: Kind::Json.into(),
            data: Some(Data::Json(json)),
          }),
        }),
        Err(e) => Ok(Output {
          port: port.to_owned(),
          invocation_id: inv_id.to_owned(),
          payload: Some(MessageKind {
            kind: Kind::Error.into(),
            data: Some(Data::Message(e.to_string())),
          }),
        }),
      },
      Success::Json(json) => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(MessageKind {
          kind: Kind::Json.into(),
          data: Some(Data::Json(json)),
        }),
      }),
    },
    MessageTransport::Failure(v) => match v {
      Failure::Invalid => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(MessageKind {
          kind: Kind::Invalid.into(),
          data: None,
        }),
      }),
      Failure::Exception(msg) => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(MessageKind {
          kind: Kind::Exception.into(),
          data: Some(Data::Message(msg)),
        }),
      }),
      Failure::Error(msg) => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(MessageKind {
          kind: Kind::Error.into(),
          data: Some(Data::Message(msg)),
        }),
      }),
    },

    MessageTransport::Signal(signal) => match signal {
      MessageSignal::Done => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(MessageKind {
          kind: Kind::Signal.into(),
          data: Some(Data::Signal(OutputSignal::Done.into())),
        }),
      }),
      MessageSignal::OpenBracket => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(MessageKind {
          kind: Kind::Signal.into(),
          data: Some(Data::Signal(OutputSignal::OpenBracket.into())),
        }),
      }),
      MessageSignal::CloseBracket => Ok(Output {
        port: port.to_owned(),
        invocation_id: inv_id.to_owned(),
        payload: Some(MessageKind {
          kind: Kind::Signal.into(),
          data: Some(Data::Signal(OutputSignal::CloseBracket.into())),
        }),
      }),
    },
  }
}
