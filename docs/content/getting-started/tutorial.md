---
title: Tutorial
---
# Start a new project

In this tutorial we'll be making a template renderer component in Rust.

The `wick` repository includes templates in the `templates/` folder with templates in the common `liquid` template format. Use `cargo generate` (`cargo install cargo-generate`) to easily pull, render, and setup new components.

```
$ cargo generate candlecorp/wick templates/rust --name jinja
```

The template comes with a sample `component.yaml` that defines two operations, `greet` and `add`.

```
---
name: 'jinja'
format: 1
metadata:
  version: '0.0.1'
operations:
  - name: greet
    inputs:
      - name: input
        type: string
    outputs:
      - name: output
        type: string
  - name: add
    inputs:
      - name: left
        type: u64
      - name: right
        type: u64
    outputs:
      - name: output
        type: u64
```

This file powers Wick's code generation and is embedded into your WebAssembly module at build time. Wick uses the configuration and its operations, types, descriptions, and other metadata to automatically configure and validate Wick applications.

Get rid of the example operations and add one called `render`. The `render` operation will take template source and arbitrary data to use as the context. The template will be a `string` type and – since template data can be anything – the data input can be a generic `struct`. The `struct` type represents any JSON-like object. Name our output stream  `output` and give it a type of `string`.

The `component.yaml` should now look like this:

```
---
name: 'jinja'
format: 1
metadata:
  version: '0.0.2'
operations:
  - name: render
    inputs:
      - name: template
        type: string
      - name: data
        type: struct
    outputs:
      - name: output
        type: string
```

# Add dependencies

Add the template renderer `minijinja` to our project with `cargo add` or by modifying the `Cargo.toml` file by hand. `minijinja` is a rust implementation of the [jinja](https://jinja.palletsprojects.com/en/3.1.x/) template library.

```
$ cargo add minijinja
```

# Update implementation

This template's build step will generate a `Component` struct and traits for every operation defined in the manifest. The operation traits take each input as a separate stream argument and one final argument for the output stream(s).

*Note: The generated code can be found in your `target` directory. Have a peek at the generated code by looking in the `target/wasm32-unknown-unknown/debug/build/jinja-*/out/` directory (after running a build at least once). The code generator runs every time the `component.yaml` file changes.*

Replace the contents of `src/lib.rs` with the following:

```rs
use wasmrs_guest::*;
mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[async_trait::async_trait(?Send)]
impl OpRender for Component {
    async fn render(
        mut input: WickStream<String>,
        mut data: WickStream<Value>,
        mut outputs: OpRenderOutputs,
    ) -> wick::Result<()> {
        while let (Some(Ok(input)), Some(Ok(data))) = (input.next().await, data.next().await) {
            let mut env = minijinja::Environment::new();
            env.add_template("root", &input).unwrap();

            let template = env.get_template("root").unwrap();
            let rendered = template.render(data).unwrap();
            outputs.output.send(&rendered);
        }
        outputs.output.done();
        Ok(())
    }
}
```

The body of the implementation is standard rust code that can use any crate you've added to your `Cargo.toml`. Wick uses streams everywhere so – even though we're just taking a single input – we're waiting for pairs of values to come in from both streams. This let's our components be used in  be reused easily in many different cases and let's us build more complex components that can adapt to more.

Inside the loop we follow the [guide for `minijinja`](https://crates.io/crates/minijinja) to compile a template and render it with our data.

Finally we send the rendered output to our `output` stream with `outputs.output.send()`.

Outside the loop we can do whatever cleanup we need and close the output streams with `outputs.output.done()`.

# Run just build

Run the `just build` task to compile our code to WebAssembly, embed our component definition and types, and sign the module with our private key. If you don't have any keys yet, `wick` will make them for you automatically and put them in `~/.wick/keys/`.

```
$ just build
```

# Run your component

Use `wick invoke` to invoke any operation in your signed component `.wasm` file. By default, the built artifacts get copied to a `build` directory in the project root.

`wick invoke` takes the path to your `.wasm` file, the name of the operation to invoke, and any arguments to pass to the operation. The `--` is required to separate the arguments to `wick invoke` from the arguments to the operation.

*Note: The data passed to each argument is treated as JSON. The `data` argument should contain valid JSON that will be processed before sending to your component. The `template` argument is a string and `wick` will automatically wrap unquoted, invalid JSON with a string before processing it.*

```
$ wick invoke ./build/jinja.signed.wasm render -- --template 'Hello {{ name }}!' --data '{"name": "Samuel Clemens"}'
{"payload":{"value":"Hello Samuel Clemens!"},"port":"output"}
```

Success! `wick` loaded our component, validated it's integrity, found our operation, and invoked it with the arguments we passed. The output is a JSON-ified representation of Wick packets, the pieces of data that flow through pipelines in the wick runtime.