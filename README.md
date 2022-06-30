<div align="center">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/wasmflow/.github/blob/main/assets/wasmflow-logo-white-color@.5.png?raw=true">
  <img alt="wasmflow logo" width="50%" src="https://github.com/wasmflow/.github/blob/main/assets/wasmflow-logo-color@.5.png?raw=true">
</picture>
</div>

# Wasmflow

Wasmflow is a free and open platform for building applications out of WebAssembly code containers.

Wasmflow lets you write code and reuse it easily, everywhere. It's built with Rust and Wasmtime and takes ideas from Docker, Kubernetes, Erlang, Rx, FBP, and more.

## Getting Started

Visit the docs on [wasmflow.com](https://wasmflow.com) for getting started guides and API documentation.

## Installation

Head over to the [releases page](https://github.com/wasmflow/wasmflow/releases) to download precompiled binaries for your platform.

Alternately, install from source with the command:

```
$ make install
```

or for more optimized builds:

```sh
$ make install-release
```

## Prerequisites for building from source

Aside from rust & cargo, you'll need node.js & npm to run and build the code generator.

You can install all the prerequisites for tools that can be installed via rust and npm with the command:

```sh
$ make deps
```

### jq

The Makefiles also make use of `jq`. You'll find installation instructions on the `jq` homepage: https://stedolan.github.io/jq/

### cmake

The generated protobuf code uses prost which depends on a number of developer tools. Many developer systems will probably have these installed already, but recent macs no longer include `cmake` with the XCode CLI tools. You'll need either the full xcode installation or homebrew to install it.

You can install xcode from the app store or you can install `cmake` via brew with the command:

```sh
$ brew install cmake
```

### Rust nightly

You'll also need the rust nightly toolchain in addition to stable for some of the dependent tools and code formatting.

```
$ rustup toolchain add nightly
```

## Building Wasmflow

Build Wasmflow with `make` alone.

```console
make
```

## Running tests

Run unit and self-contained integration tests via `make`

```console
make test
```

To run the full suite of tests, you'll need a local NATS server, redis instance, and OCI registry. You can run these with the `docker-compose.yml` file in the `/scripts/` directory.

```console
$ cd scripts && docker-compose up
```

## Need a Makefile primer?

Check out these guides if you need help with `Makefile`s:

- [Isaacs's tutorial](https://gist.github.com/isaacs/62a2d1825d04437c6f08)
- [Your makefiles are wrong](https://tech.davis-hansson.com/p/make/)
