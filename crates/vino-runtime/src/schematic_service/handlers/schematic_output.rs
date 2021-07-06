use crate::dev::prelude::*;

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<(), SchematicError>")]
pub(crate) struct SchematicOutput {
  pub(crate) port: String,
  pub(crate) tx_id: String,
  pub(crate) payload: MessageTransport,
}

impl Handler<SchematicOutput> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: SchematicOutput, _ctx: &mut Context<Self>) -> Self::Result {
    trace!("Schematic port '{}' is ready", msg.port);

    let tx = actix_try!(self
      .tx_external
      .get(&msg.tx_id)
      .ok_or_else(|| SchematicError::TransactionNotFound(msg.tx_id.clone())));

    debug!("Sending output on transmitter");

    let err = Packet::V0(packet::v0::Payload::Error(
      "Invalid payload received as schematic output".to_owned(),
    ));

    let output_msg = OutputPacket {
      invocation_id: msg.tx_id,
      payload: match msg.payload {
        MessageTransport::Invalid => Packet::V0(packet::v0::Payload::Invalid),
        MessageTransport::Exception(v) => Packet::V0(packet::v0::Payload::Exception(v)),
        MessageTransport::Error(v) => Packet::V0(packet::v0::Payload::Error(v)),
        MessageTransport::MessagePack(v) => Packet::V0(packet::v0::Payload::MessagePack(v)),
        MessageTransport::MultiBytes(_) => err,
        MessageTransport::OutputMap(_) => err,
        MessageTransport::Test(_) => err,
        MessageTransport::Signal(v) => match v {
          MessageSignal::Close => Packet::V0(packet::v0::Payload::Close),
          MessageSignal::OpenBracket => Packet::V0(packet::v0::Payload::OpenBracket),
          MessageSignal::CloseBracket => Packet::V0(packet::v0::Payload::CloseBracket),
        },
      },
      port: msg.port,
    };

    meh!(tx.send(output_msg));

    ActorResult::reply(Ok(()))
  }
}
