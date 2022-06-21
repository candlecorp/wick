![Maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)

# wasmflow-transport

Wasmflow Transport contains the structures and methods for communicating
across entity boundaries. It handles abstracting payload versions and
implementations so they can be used easily.

The `MessageTransport` struct normalizes `wasmflow_packet::Packet`s for
the Wasmflow tools.

The `TransportWrapper` wraps a `MessageTransport` along with the port name
it originated from.

A `TransportStream` is a stream of `TransportWrapper`s.
