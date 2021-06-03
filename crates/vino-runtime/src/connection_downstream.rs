use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
