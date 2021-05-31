use std::collections::HashMap;

use crate::{
    deserialize, hlreg::HostLocalSystemService, network::Network, port_entity::PortEntity,
    schematic::OutputReady, Result,
};

use serde::{Deserialize, Serialize};

use vino_guest::OutputPayload;

use crate::MessagePayload;

#[derive(Debug, Serialize, Deserialize)]
struct PassedJobArgs {
    connection: ConnectionDownstream,
    input: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Default)]
pub struct ComponentMetadata {
    pub ports: super::ActorPorts,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ConnectionDownstream {
    pub host_id: String,
    pub namespace: String,
    pub tx_id: String,
    pub actor: String,
    pub reference: String,
}

impl ConnectionDownstream {
    pub fn new(
        host_id: String,
        namespace: String,
        tx_id: String,
        actor: String,
        reference: String,
    ) -> Self {
        ConnectionDownstream {
            host_id,
            namespace,
            tx_id,
            actor,
            reference,
        }
    }
    pub fn send(&self, port: String, data: Vec<u8>) -> Result<()> {
        let network = Network::from_hostlocal_registry(&self.host_id);
        trace!(
            "sending to output {}[{}] (txid: {})",
            self.reference,
            port,
            self.tx_id,
        );
        let payload = match deserialize::<OutputPayload>(&data)? {
            OutputPayload::Bytes(b) => MessagePayload::Bytes(b),
            OutputPayload::Exception(e) => MessagePayload::Exception(e),
            OutputPayload::Error(e) => MessagePayload::Error(e),
        };
        network.try_send(OutputReady {
            port: PortEntity {
                name: port,
                reference: self.reference.to_string(),
                schematic: self.namespace.to_string(),
            },
            tx_id: self.tx_id.to_string(),
            payload,
        })?;
        Ok(())
    }
}
