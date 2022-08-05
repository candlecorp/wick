# Wasmflow project structure

The `wasmflow/wasmflow` monorepo houses four main projects as well as their dependencies:

- The `wasmflow` binary project at `./crates/bins/wasmflow`.
- The `wafl` binary project at `./crates/bins/wafl`.
- The `wasmflow-sdk` crate at `./crates/wasmflow/wasmflow-sdk`.
- The `wasmflow-codegen` executable at `./codegen`.

The first

## Directories and their purpose

`codegen/` - Includes the node-based code generator.

`crates` - Wasmflow source crates.

`docs/` - Developer documentation.

`etc/` - Miscellaneous configuration

`makefiles/` - Core makefiles included by various sub-projects.

`scripts/` - Miscellaneous scripts and files to aid development.

`tests/` - Integration tests that couldn't fit in sub-crates.

`deny.toml` - Configuration for `cargo deny` checks

`Makefile` - The Makefile for the wasmflow project.

`package.json` - Dependencies for node-based support scripts.

## Crates

The wasmflow crates are broken down into several child directories:

- `bins/` - contains the `wasmflow` and `wafl` crates that build their respective binaries.
- `collections/` - contains implementations of native (non-WASM) collections.
- `interfaces/` - contains well-known interface definitions.
- `integration/` - contains projects used for integration testing.
- `misc/` - support crates that are not specific to wasmflow but are managed within the project.
- `wasmflow/` - wasmflow libraries.

### Wasmflow crates

`logger/` - The logger used by Wasmflow libraries.

`wasmflow-collection-cli` - Common CLI interface, options, and utilities for native based collections and persistent processes.

`wasmflow-collection-grpc` - Implementation of a wasmflow collection over GRPC.

`wasmflow-collection-grpctar` - Implementation of a GRPC wasmflow collection served as a tarball with an architecture-specific binary.

`wasmflow-collection-nats` - Implementation of a wasmflow collection over NATS message queue.

`wasmflow-collection-wasm` - Implementation of a wasmflow collection as a WASM module.

`wasmflow-grpctar` - Library for archiving/extracting multi-architecture binaries.

`wasmflow-host` - Library that encompasses the host logic used by the `wasmflow` binary.

`wasmflow-interpreter` - Library that interprets a wasmflow graph and how data flows across the connections.

`wasmflow-invocation-server` - Library that contains an implementation of the wasmflow invocation service that spins up a GRPC server.

`wasmflow-loader` - Utility library for loading wasmflow assets via filepath or OCI reference.

`wasmflow-manifest` - Library for loading and normalizing wasmflow manifest files.

`wasmflow-mesh` - Library that manages connecting to a wasmflow mesh over NATS.

`wasmflow-oci` - Library for pushing/pulling from OCI registries.

`wasmflow-parser` - Library for parsing the mini-DSL in manifests.

`wasmflow-rpc` - Library that houses the generated protobuf code for the GRPC service and the translation from protobuf types to wasmflow types.

`wasmflow-runtime` - The core runtime library for wasmflow.

`wasmflow-schematic-graph` - Library that represents the flow graph data structure. Used mainly by the interpreter.

`wasmflow-sdk` - Library used by Rust-based WASM components.

`wasmflow-stdlib` - The standard components implemented as native code within wasmflow.

`wasmflow-test` - The TAP test harness and implementation.

`wasmflow-wascap` - Wasmflow-specific WasCap implementations.
