/// V0 parser logic
pub mod v0;
/// V1 parser logic
pub mod v1;

/// The reserved reference name for schematic input. Used in schematic manifests to denote schematic input.
pub const SCHEMATIC_INPUT: &str = "<input>";
/// The reserved reference name for schematic output. Used in schematic manifests to denote schematic output.
pub const SCHEMATIC_OUTPUT: &str = "<output>";
/// The reserved reference name for schematic output. Used in schematic manifests to denote schematic output.
pub const SCHEMATIC_NULL: &str = "__null__";
/// The reserved reference name for a namespace link. Used in schematic manifests to pass a collection to a port by its namespace.
pub const NS_LINK: &str = "<link>";
/// The reserved name for components that send static data.
pub const SENDER_ID: &str = "core::sender";
/// The reserved name for data that Wick injects itself.
pub const CORE_ID: &str = "core";
/// The name of SENDER's output port.
pub const SENDER_PORT: &str = "output";
