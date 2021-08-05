![Maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)

# vino-transport

Vino Transport contains the structures and methods for communicating
across entity boundaries. It handles abstracting payload versions and
implementations so they can be used easily.

The [MessageTransport] struct normalizes [vino_packet::Packet]s for
the Vino tools.

The [TransportWrapper] wraps a [MessageTransport] along with the port name
it originated from.

A [TransportStream] is a stream of [TransportWrapper]s.


License: BSD-3-Clause
