pub(crate) mod v0;
pub(crate) mod v1;
/// The reserved reference name for schematic input. Used in schematic manifests to denote schematic input.
pub const SCHEMATIC_INPUT: &str = "<input>";
/// The reserved reference name for schematic output. Used in schematic manifests to denote schematic output.
pub const SCHEMATIC_OUTPUT: &str = "<output>";
/// The reserved reference name for a namespace link. Used in schematic manifests to pass a collection to a port by its namespace.
pub const NS_LINK: &str = "<link>";
/// The reserved port name to use when sending an asynchronous error from a component.
pub const COMPONENT_ERROR: &str = "<error>";
/// The reserved namespace for references to internal schematics.
pub const SELF_NAMESPACE: &str = "self";
/// The reserved name for components that send static data.
pub static SENDER_ID: &str = "core::sender";
/// The reserved name for data that Wasmflow injects itself.
pub static CORE_ID: &str = "<core>";
/// The name of SENDER's output port.
pub static SENDER_PORT: &str = "output";
