use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct PortEntity {
    pub schematic: String,
    pub reference: String,
    pub name: String,
}
