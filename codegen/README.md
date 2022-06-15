# Vino codegen

This is the core code generator for [Vino](https://vino.dev) components and providers.

## Installation

```shell
$ npm install -g @vinodotdev/codegen
```

## Install from Source
```shell
$ npm install
$ npm run build
$ npm install -g .
```

## Usage

Run `vino-codegen --help` to get a list of languages available to generate. Use `--help` on any of the languages to dive further, e.g.:

```shell
$ vino-codegen rust --help
vino-codegen rust

Generate Rust code from a WIDL schema

Commands:
  vino-codegen rust interface <schema_dir> [options]             Generate source code for well-known interfaces
  vino-codegen rust provider-component <schema> [options]        Generate boilerplate for native provider components
  vino-codegen rust provider-integration <schema_dir> [options]  Generate the Vino integration code for all component schemas
  vino-codegen rust wapc-component <schema> [options]            Generate boilerplate for WaPC components
  vino-codegen rust wapc-integration <schema_dir> [options]      Generate the Vino & WaPC integration code for all component schemas
  vino-codegen rust wapc-lib                                     Generate the boilerplate lib.rs for WaPC components
  vino-codegen rust wellknown-implementer <interface> [options]  Generate the Vino integration code for well-known interface schemas

Options:
      --version  Show version number                                                                                                           [boolean]
  -h, --help     Show help                                                                                                                     [boolean]
```

### Testing and debugging

Run tests via

```
$ npm run tests
```

Tests for the generated code are accounted for in downstream consumers but this repository should have some baseline tests. THis would be a great first issue for anyone interested.
