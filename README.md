<div align="center">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/candlecorp/.github/blob/main/assets/wick_logo.png?raw=true">
  <img alt="wick logo" width="50%" src="https://github.com/candlecorp/.github/blob/main/assets/wick_logo.png@.5.png?raw=true">
</picture>
</div>

# Wick

Wick is a reactive, flow-based framework for building applications out of WebAssembly components.

Wick is built with Rust and Wasmtime and takes ideas from Docker, Kubernetes, Erlang, Rx, FBP, and more.

## Getting Started

Visit the docs in the [Wick Wiki](https://github.com/candlecorp/wick/wiki) for the getting started guide and API documentation.

## Quick Install - Mac and Linux

Mac and Linux users can install the latest stable version of `wick` with the following command:

```sh
curl -sSL sh.wick.run | bash
```

To download and install the nightly version, or other releases of `wick`, pass the version or tag as the argument.

```sh
curl -sSL sh.wick.run | bash -s -- nightly
```

## Quick Install - Windows

Windows users can install the latest stable version of `wick` with the following command:

```sh
curl https://ps.wick.run -UseBasicParsing | Invoke-Expression
```

To download and install the nightly version, or other releases of `wick`, pass a tag or version to the downloaded PowerShell script:

```sh
curl https://ps.wick.run -OutFile setup-wick.ps1; .\setup-wick.ps1 -ReleaseVersion "nightly"; rm setup-wick.ps1;
```

### Install with Homebrew

```sh
brew install candlecorp/tap/wick
```

Or to install from source:

```sh
brew install candlecorp/tap/wick --head
```

## Manual Installation

Go to the [releases page](https://github.com/candlecorp/wick/releases) to download precompiled binaries for your platform.

## Compile from Source

1. Clone the repository

```sh
git clone https://github.com/candlecorp/wick.git && cd wick
```

2. Run the install task with `just install`. This will install `wick` to your local cargo bin directory, usually `~/.cargo/bin`.

```sh
just install
```

## Prerequisites for building from source

Aside from rust & cargo, you'll need node.js & npm to run and build the code generator.

You can install all the prerequisites for tools that can be installed via rust and npm with the command:

```sh
$ just deps
```

### jq

The tasks also make use of `jq`. Find installation instructions on the `jq` homepage: https://stedolan.github.io/jq/

### protoc

The generated protobuf code relies on protoc. Check the [protobuf](https://grpc.io/docs/protoc-installation/) homepage for installation instructions.

### Rust nightly

You will need the Rust nightly toolchain for some of the dependent tools and code formatting.

```
rustup toolchain add nightly
```

## Building Wick

Build Wick with `just` alone.

```console
just build
```

## Running tests

Run unit and self-contained integration tests via `just`

```console
just test
```

To run the full suite of tests, run the `integration-tests` task below. Integration tasks require a modern version of Docker to run.

```console
just integration-tests
```

