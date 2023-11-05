---
title: Writing a WebAssembly Component
weight: 1
---

# Start a new project

In this tutorial we'll be making a template renderer component in Rust.

# Prerequisites

Ensure you have [wick-cli](https://candle.dev/docs/wick/getting-started/installation/) and [`just`](https://github.com/casey/just#installation) installed.

# Starting

The `wick` repository includes templates in the `templates/` folder with templates in the common `liquid` template format. Use `cargo generate` (`cargo install cargo-generate`) to easily pull, render, and setup new components.

```console
$ cargo generate candlecorp/wick templates/rust --name jinja
```

The template comes with a sample `component.wick` that defines two operations, `greet` and `add`.

```yaml
---
name: 'jinja'
kind: wick/component@v1
metadata:
  version: '0.0.1'
component:
  kind: wick/component/wasmrs@v1
  ref: build/component.signed.wasm
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

Get rid of the example operations and add one called `render`. The `render` operation will need the raw template and arbitrary data to render the template with. The template will be a `string` type and – since template data can be anything – the data input can be a generic `object`. The `object` type represents any JSON-like object. Since it's common for templates to stay static while the data changes, we can define the `template` as part of the operation's configuration, rather than its input. An operation will receive configuration once while its input streams can have any number of elements.

As for our output, it will be a `string` and we'll name it `rendered`.

The `component.wick` should now look like this:

```yaml
---
name: 'jinja'
kind: wick/component@v1
metadata:
  version: '0.0.1'
component:
  kind: wick/component/wasmrs@v1
  ref: build/component.signed.wasm
  operations:
    - name: render
      with:
        - name: template
          type: string
      inputs:
        - name: data
          type: object
      outputs:
        - name: rendered
          type: string
```

# Add dependencies

Add the template renderer `minijinja` to our project with `cargo add` or by modifying the `Cargo.toml` file by hand. `minijinja` is a Rust implementation of the [jinja](https://jinja.palletsprojects.com/en/3.1.x/) template library.

```console
$ cargo add minijinja
```

# Update implementation

This template's build step will generate a `Component` struct and traits for every operation defined in the manifest. The operation traits take each input as a separate stream argument and one final argument for the output stream(s).

_Note: The generated code can be found in your `target` directory. Have a peek at the generated code by looking in the `target/wasm32-unknown-unknown/debug/build/jinja-_/out/`directory (after running a build at least once). The code generator runs every time the`component.wick` file changes.\*

Replace the contents of `src/lib.rs` with the following:

```rs
mod wick {
    wick_component::wick_import!();
}
use wick::*;

#[wick_component::operation(generic_raw)]
async fn render(
    mut inputs: render::Inputs,
    mut outputs: render::Outputs,
    ctx: Context<render::Config>,
) -> Result<(), anyhow::Error> {
    let mut templates = minijinja::Environment::new();
    templates.add_template("root", &ctx.config.template)?;
    let template = templates.get_template("root")?;

    while let Some(input) = inputs.data.next().await {
        let data = input.decode()?;
        let rendered = template.render(data)?;
        outputs.rendered.send(rendered);
    }

    Ok(())
}
```

The body of the implementation is standard Rust code that can use any crate you've added to your `Cargo.toml`. Wick uses streams everywhere, so many operations start with a loop awaiting for values. Expecting everything to be a stream and accounting for streaming cases up front makes components more flexible and reusable while still working perfectly fine for common cases.

Since we get the template as part of our `with` configuration block, we have access to it immediately and can pre-compile it with tips from the [`minijinja` guide](https://crates.io/crates/minijinja).

Finally we send the rendered template to our output stream named `rendered` with `outputs.rendered.send()`.

Outside the loop we can do whatever cleanup we need and close the output streams with `outputs.rendered.done()`. The streams will close automatically when the operation returns, but it's good practice to close them explicitly.

# Run `just build`

Run the `just build` task to compile our code to WebAssembly, embed our component definition and types, and sign the module with our private key. If you don't have any keys yet, they will be automatically generated for you by `wick` when you run `just build` and they will be put in `~/.wick/keys/`.

```console
$ just build
```

# Run your component

Use `wick invoke` to invoke any operation in your signed component `.wasm` file. By default, the built artifacts get copied to a `build` directory in the project root.

`wick invoke` takes the path to your `.wick` file, the name of the operation to invoke, and any arguments to pass to the operation. The `--` is required to separate the arguments to `wick invoke` from the arguments to the operation.

_Note: Wick expects Component and Operation configuration passed on the CLI to be valid JSON._

```console
$ wick invoke ./component.wick render --op-with '{ "template": "{%raw%}Hello {{ name }}!{%endraw%}" }' -- --data '{ "name": "Samuel Clemens" }'
{"payload":{"value":"Hello Samuel Clemens!"},"port":"rendered"}
```

_Note: Wick processes input configuration as a Liquid template which has similar syntax as Jinja. We're using Liquid's `{%raw%}...{%endraw%}` syntax to treat the inner template as raw text._

Writing valid JSON in CLI arguments is cumbersome. We can use Wick's `@file` syntax to take data from the filesystem. The command above is equivalent to the following, assuming the data has been written to files `config.json` and `data.json`.

```console
$ wick invoke ./component.wick render --op-with @config.json -- --data @data.json
{"payload":{"value":"Hello Samuel Clemens!"},"port":"rendered"}
```

Success! `wick` loaded our component, validated it's integrity, found our operation, and invoked it with the arguments we passed.

The output is a JSON-ified representation of Wick packets, the pieces of data that flow through pipelines in the wick runtime.

To get the raw output, we can use the `--values` flag:

```console
$ wick invoke ./component.wick render --values --op-with @config.json -- --data @data.json
Hello Samuel Clemens!
```

Now that you have a streaming WebAssembly component, there are a bunch of places to go next.

Do you want to:

- [Expose your component with a RestAPI](../rest-api)?
- [Run your component in a browser?](../../reference/sdks/browser)
- [Run your component in node.js?](../../reference/sdks/nodejs)
- [Run your component in a pipeline?](../../reference/components/composite)
- [Learn about other types of components?](../../reference/components/)
