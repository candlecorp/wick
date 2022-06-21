# Wasmflow codegen

This is the official code generation utility for [Wasmflow](https://wasmflow.com) components and providers.

## Installation

```shell
$ npm install -g @candlecorp/codegen
```

## Install from Source

```shell
$ npm install
$ npm run build
$ npm install -g .
```

## Usage

Run `wasmflow-codegen --help` to get a list of languages available to generate. Use `--help` on any of the languages to dive further.

This executable is primarily used by the Makefiles in the [Wasmflow](https://github.com/wasmflow/wasmflow) project and its components. See those for usage.

### Testing and debugging

Run tests via

```
$ npm run tests
```

Tests for the generated code are accounted for in downstream consumers but this repository should have some baseline tests. THis would be a great first issue for anyone interested.
