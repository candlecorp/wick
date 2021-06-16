use serde::{Deserialize, Serialize};
use vino_runtime::{Invocation, MessagePayload, PortEntity};

#[derive(Debug, Serialize, Deserialize)]
pub enum VinoRpcMessage {
    Invoke(Invocation),
    Output(OutputPayload),
    Close(ClosePayload),
    Ack(String),
    Error(String),
    Ping,
    Pong,
    Shutdown,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OutputPayload {
    pub entity: PortEntity,
    pub tx_id: String,
    pub payload: MessagePayload,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ClosePayload {
    pub entity: PortEntity,
    pub tx_id: String,
}

pub const OP_INVOKE: &str = "invoke";
pub const OP_OUTPUT: &str = "output";
pub const OP_CLOSE: &str = "close";
pub const OP_ERROR: &str = "error";
pub const OP_ACK: &str = "ack";
pub const OP_PING: &str = "ping";
pub const OP_PONG: &str = "pong";
pub const OP_SHUTDOWN: &str = "shutdown";

impl VinoRpcMessage {
    pub fn op_name(&self) -> &str {
        match self {
            VinoRpcMessage::Invoke(_) => OP_INVOKE,
            VinoRpcMessage::Output { .. } => OP_OUTPUT,
            VinoRpcMessage::Close { .. } => OP_CLOSE,
            VinoRpcMessage::Ack { .. } => OP_ACK,
            VinoRpcMessage::Error(_) => OP_ERROR,
            VinoRpcMessage::Ping => OP_PING,
            VinoRpcMessage::Pong => OP_PONG,
            VinoRpcMessage::Shutdown => OP_SHUTDOWN,
        }
    }
}

#[cfg(test)]
mod tests {
    use vino_runtime::VinoEntity;

    use super::*;

    #[test_env_log::test(tokio::test)]
    async fn enforce_names() {
        let close = VinoRpcMessage::Close(ClosePayload::default());
        assert_eq!(close.op_name(), "close");
        let output = VinoRpcMessage::Output(OutputPayload::default());
        assert_eq!(output.op_name(), "output");
        let invoke = VinoRpcMessage::Invoke(Invocation {
            origin: VinoEntity::Component("".to_string()),
            target: VinoEntity::Component("".to_string()),
            operation: "".to_string(),
            msg: MessagePayload::MessagePack(vec![]),
            id: "".to_string(),
            tx_id: "".to_string(),
            encoded_claims: "".to_string(),
            host_id: "".to_string(),
        });
        assert_eq!(invoke.op_name(), "invoke");
    }
}
