<div align="center">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/candlecorp/.github/blob/main/assets/wick_logo.png?raw=true">
  <img alt="wick logo" width="50%" src="https://github.com/candlecorp/.github/blob/main/assets/wick_logo.png@.5.png?raw=true">
</picture>
</div>

# Wick

Wick is a free and open platform for building applications out of WebAssembly code containers.

Wick lets you write code and reuse it easily, everywhere. It's built with Rust and Wasmtime and takes ideas from Docker, Kubernetes, Erlang, Rx, FBP, and more.

## Getting Started

Visit the docs in the [Wick Wiki](https://github.com/candlecorp/wick/wiki) for the getting started guide and API documentation.

## Quick Install - Mac and Linux
Mac and Linux users can install the latest stable version of Wick with the following command:
```sh
curl -sSL sh.wick.run | bash
```

To download and install the nightly version, or other releases of wick, pass a single argument of the desired release.
```sh
curl -sSL sh.wick.run | bash -s -- nightly
```

## Quick Install - Windows
Windows users can install the latest stable version of Wick with the following command:
```sh
curl https://ps.wick.run -UseBasicParsing | Invoke-Expression
```

To download and install the nightly version, or other releases of wick, pass a single argument of the desired release.
```sh
curl https://ps.wick.run -OutFile setup-wick.ps1; .\setup-wick.ps1 -ReleaseVersion "nightly"; rm setup-wick.ps1;
```

## Manual Install
If you want to set it up manually, you can go to the [releases page](https://github.com/candlecorp/wick/releases) to download precompiled binaries for your platform.

Make sure you add the binary location to your path.

## Compile from Source
Clone the repository and run the install task with `just`, you will need to ensure the path is updated appropriately for your system:

```
$ just install
```

or for more optimized builds:

```sh
$ just install-release
```

## Prerequisites for building from source

Aside from rust & cargo, you'll need node.js & npm to run and build the code generator.

You can install all the prerequisites for tools that can be installed via rust and npm with the command:

```sh
$ just deps
```

### jq

The tasks also make use of `jq`. Find installation instructions on the `jq` homepage: https://stedolan.github.io/jq/

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

To run the full suite of tests, you'll need a local NATS server, redis instance, and OCI registry. You can run these with the `docker-compose.yml` file in the `/etc/` directory.

```console
$ cd scripts && docker-compose up
```

