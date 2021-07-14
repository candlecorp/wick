# WASM Component Quick Start guide

## Installing Vino tools

Run `make install` from the the vino repository root to build vino dependencies and install vino binaries into your cargo install dir.

```shell
$ make install
```

## Install dependencies

### Node.js dependencies

```shell
$ npm install -g yo @vinodotdev/codegen
```

Clone the generator (temporary until released on npm)

```shell
$ git clone https://github.com/vinodotdev/yo-vino && cd yo-vino
$ npm install -g .
```

### Create a Vino WASM module

Create an empty directory

```shell
mkdir my-component && cd my-component
```

Run the generator and answer the prompts

```shell
$ yo vino
```

Build your component

```shell
$ make
```

Run your component

Use `vow` to load your module and execute a component on the command line

```shell
$ vow ./build/test_component_s.wasm my-component '{"input":"my_input"}'
```

To change your module name:

- Rename the `namespace` in the schema definition found in `./schemas/my-component.widl`

To make the filename reflect changes, change the filename of `./schemas/my-component.widl`

## Add sample logic

Add to `./src/components/my_component.rs` (or whatever you renamed your component to) so that it looks like this:

```rs
use wapc_guest::prelude::*;

use crate::generated::my_component::*;

pub(crate) fn job(input: Inputs, output: Outputs) -> HandlerResult<()> {
  output.output.send(input.input)?;
  Ok(())
}
```

Run your component now to see the output:

```sh
$ vow ./build/test_component_s.wasm my-component '{"input":"my_input"}'
{"output":{"error_kind":"None","value":"my_input"}}
```

### Connect your component

Docs TODO, but look at [vino-repo]/tests/manifests/wapc.yaml as an example
