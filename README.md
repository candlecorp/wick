# Vino monorepo

## Prerequisites

Vino depends on (at least) the tools below to build properly.

### System dependencies

These tools are necessary to build the majority of Vino projects, but you need to install them yourself.

- rust/cargo
- node
- make
- docker & docker-compose (to run integration tests)

### Necessary tools

The rest of the dependencies should be installable via:

```
make deps
```

If you find something that should be added, please submit a PR.

#### widl-template

Used to generated JSON schemas, documentation, and code for host manifests.

- `npm install -g widl-template`

#### vino-codegen

Used to generate provider scaffolding and new component templates

- `npm install -g @vinodotdev/codegen`

Or from the `dev` branch

- `npm install -g "https://github.com/vinodotdev/codegen#dev"`

#### tomlq

Used in WASM code generation and Makefiles.

- `cargo install tomlq`

#### `cargo-deny`

Used to assert dependency licenses

- `cargo install cargo-deny`

#### prettier

Used to format JSON

- `npm install -g prettier`

## Using make

Vino uses `Makefile`s extensively, make sure you explore them to understand what you can run as tasks.

Many makefiles have a `help` task to give you a rundown of the important tasks.

```console
make help
```

## Building vino

Build vino with `make` alone.

```console
make
```

## Install vino binaries from source

```console
make install
```

Alternately, to install optimized release builds:

```console
make install-release
```

## Running tests

To run the full suite of tests, you'll need a local NATS server, redis instance, and OCI registry. You can run these with the `docker-compose.yml` file in the `/scripts/` directory.

```console
$ cd scripts && docker-compose up
```

Run tests via `make`

```console
make test
```

## Doc links

- docs.vino.dev/codegen
- docs.vino.dev

## Need a Makefile primer?

- Check out isaacs's tutorial: https://gist.github.com/isaacs/62a2d1825d04437c6f08
- Your makefiles are wrong: https://tech.davis-hansson.com/p/make/

## Good first contributions

This is a list of nice-to-haves that would also make good contributions for people looking to get involved with Vino.

### Improving logging & the logger

Logging is an unstructured mix of debug and informational output. It would be better to have a structured logging solution.

### Opportunities for code generation

Vino uses code generation extensively and making it better or adding more opportunities to use generated code is usually welcome. Open an issue first to discuss it to be sure that someone isn't already working on it.

### Rustdoc examples

Rustdoc examples are always helpful. Examples should be written in a way that they can be copy-pasted and executed as-is without hidden context whenever possible.

### FAQ Documentation

As you go work with Vino, what issues pop up that you solve yourself? Those issues could make good FAQ items.
