![Maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)

# vino-provider

The Vino provider crate contains the necessary pieces for Native
or WebAssembly providers written in Rust.

This library is not meant to be integrated manually. Vino uses
code generators to automate most integration and — while backwards compatibility
is a top priority — the generated code is considered the primary consumer. If you
end up using this library to fit other use cases, please open an issue to let us know
so we can track that usage.

To use this library or learn more about code generation, check out the docs at
[docs.vino.dev](https://docs.vino.dev/docs/concepts/codegen/).

License: BSD-3-Clause
