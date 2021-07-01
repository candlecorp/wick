// use serde::{
//   Deserialize,
//   Serialize,
// };

// /// The metadata that corresponds to a Vino component
// #[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
// pub struct ComponentClaims {
//   /// A descriptive name for this component
//   #[serde(skip_serializing_if = "Option::is_none")]
//   pub name: Option<String>,

//   /// A hash of the module's bytes as they exist without the embedded signature. This is stored to
//   /// determine if a WebAssembly module's bytecode has been altered after it was signed.
//   #[serde(rename = "hash")]
//   pub module_hash: String,

//   /// List of interfaces the component provides.
//   #[serde(rename = "caps", skip_serializing_if = "Option::is_none")]
//   pub provides: Option<Vec<String>>,

//   /// List of input ports and type strings.
//   #[serde(rename = "inputs")]
//   pub inputs: Option<Vec<PortSignature>>,

//   /// List of output ports and type strings
//   #[serde(rename = "outputs")]
//   pub outputs: Option<Vec<PortSignature>>,

//   /// A human-friendly semver string
//   #[serde(rename = "ver")]
//   pub ver: String,
// }
