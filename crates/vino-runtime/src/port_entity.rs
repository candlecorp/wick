use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct PortEntity {
    pub schematic: String,
    pub reference: String,
    pub name: String,
}

impl Display for PortEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}[{}]", self.schematic, self.reference, self.name)
    }
}
