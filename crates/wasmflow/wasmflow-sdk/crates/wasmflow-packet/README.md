# wasmflow-packet

The Wasmflow packet crate contains the consistent message structure for arbitrary output
from Wasmflow components and collections.

Components output versioned payloads (e.g. a `v1::Packet`) which then get
wrapped into a generic `Packet` to normalize differences across versions.

`Packet`s are designed for backwards compatibility but that compatibility layer is
strictly between the component and `Packet`, not for consumers of the `Packet`.
`Packet`s are not meant to be long lived and you should have a compatibility layer
between `Packet`s and your system if you depend on this crate.
