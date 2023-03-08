# Wick project structure

The `candlecorp/wick` monorepo houses four main projects as well as their dependencies:

- The `wick` binary project at `./crates/bins/wick`.

The first

## Directories and their purpose

`crates` - Wick source crates.

`docs/` - Developer documentation.

`etc/` - Miscellaneous scripts and files to aid development.

`Justfile` - The Justfile for the wick project.

## Crates

The wick crates are broken down into several child directories:

- `bins/` - contains the `wick` crate that builds the wick binary
- `collections/` - contains implementations of native (non-WASM) collections.
- `interfaces/` - contains well-known interface definitions.
- `integration/` - contains projects used for integration testing.
- `misc/` - support crates that are not specific to wick but are managed within the project.
- `wick/` - wick libraries.

### Wick crates

`logger/` - The logger used by Wick libraries.

`wick-component-cli` - Common CLI interface, options, and utilities for native based collections and persistent processes.

`wick-component-grpc` - Implementation of a wick collection over GRPC.

`wick-component-grpctar` - Implementation of a GRPC wick collection served as a tarball with an architecture-specific binary.

`wick-component-nats` - Implementation of a wick collection over NATS message queue.

`wick-component-wasm` - Implementation of a wick collection as a WASM module.

`wick-grpctar` - Library for archiving/extracting multi-architecture binaries.

`wick-host` - Library that encompasses the host logic used by the `wick` binary.

`flow-graph-interpreter` - Library that interprets a wick graph and how data flows across the connections.

`wick-invocation-server` - Library that contains an implementation of the wick invocation service that spins up a GRPC server.

`wick-loader-utils` - Utility library for loading wick assets via filepath or OCI reference.

`wick-config` - Library for loading and normalizing wick configuration files.

`wick-oci-utils` - Library for pushing/pulling from OCI registries.

`flow-expression-parser` - Library for parsing the mini-DSL in manifests.

`wick-rpc` - Library that houses the generated protobuf code for the GRPC service and the translation from protobuf types to wick types.

`wick-runtime` - The core runtime library for wick.

`flow-graph` - Library that represents the flow graph data structure. Used mainly by the interpreter.

`wick-stdlib` - The standard components implemented as native code within wick.

`wick-test` - The TAP test harness and implementation.

`wick-wascap` - Wick-specific WasCap implementations.
