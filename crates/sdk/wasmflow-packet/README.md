# wasmflow-packet

The Vino packet crate contains the consistent message structure for arbitrary output
from Vino components and providers.

Components output versioned payloads (e.g. a `v0::Payload`) which then get
wrapped into a `Packet` to normalize differences across versions.

`Packet`s are designed for backwards compatibility but that compatibility layer is
strictly between the component and `Packet`, not for consumers of the `Packet`.
`Packet`s are not meant to be long lived and you should have a compatibility layer
between `Packet`s and your system if you depend on this crate. For example, Vino
uses [vino-transport](https://crates.io/crates/vino-transport) to keep
a dependent platform insulated from `Packet` changes.

License: BSD-3-Clause
