# Building a Wick WebAssembly Component

*Quick links* [Protocol details](https://github.com/wasmrs/docs/blob/main/wasmrs.md) | [Rust source](https://github.com/wasmrs/wasmrs-rust/tree/main)

Wick uses the WasmRS protocol to communicate between WebAssembly components. WasmRS is an implementation of [RSocket](https://rsocket.io) in WebAssembly giving you reactive, async streams and Wick extends WasmRS with multiplexing so you can have requests that take in and receive any number of streams at once.

## How do I build a Wick component?

```sh
cargo generate candlecorp/wick -b init-templates templates/rust
```

## RequestResponse actions

This defines a service called `MyApi` that has one action – `greet` – that takes an argument and returns a string. The `name` argument is who to greet.

Run `just codegen` and watch the newly generated files get written.

> *Note: The [`just`](https://github.com/casey/just) tool is a task runner modeled after the good parts of `make`. It's not a requirement for wasmRS and you can inspect the commands you need to run in the `justfile`.*

```sh
$ just codegen
INFO Writing file ./src/actions/my_api/greet.rs (mode:644)
INFO Writing file ./src/lib.rs (mode:644)
INFO Writing file ./src/error.rs (mode:644)
INFO Writing file ./src/actions/mod.rs (mode:644)
INFO Formatting file ./src/error.rs
INFO Formatting file ./src/lib.rs
INFO Formatting file ./src/actions/mod.rs
INFO Formatting file ./src/actions/my_api/greet.rs
```

These new files include wasmRS boilerplate and scaffolding to get you started quickly.

The file `./src/actions/my_api/greet.rs` contains a stub for our `greet` action.

```rs
use crate::actions::my_api_service::greet::*;

pub(crate) async fn task(input: Inputs) -> Result<Outputs, crate::Error> {
    todo!("Add implementation");
}
```

Turn our greeter into an appropriate 'Hello World!" by returning a string like below:

```rs
use crate::actions::my_api_service::greet::*;

pub(crate) async fn task(input: Inputs) -> Result<Outputs, crate::Error> {
    Ok(format!("Hello, {}!", input.name))
}
```

Build your project with:

```sh
just build
```

`just build` runs `cargo build` and copies the resulting `.wasm` file into a `build/` directory for easy access.

To run the new `.wasm` file on the command line we can use [`NanoBus`](https://github.com/nanobus/nanobus) or the `wasmrs-request` binary.

### Running wasmRS modules with `wasmrs-request`

`wasmrs-request` is an implementation of a wasmRS host to serve as an example and a basic runner.

Install it with the command:

```sh
cargo install wasmrs-request
```

Then run:

```sh
wasmrs-request ./build/example.wasm example.MyApi greet '{"name":"World"}'
```

Output:

```sh
Hello, World!
```

### Running wasmRs modules with NanoBus

To use NanoBus we need a configuration that points to our `.wasm` file. Make an `iota.yaml` that looks like this:

```yaml
id: example
version: 0.0.1
main: build/example.wasm
```

Run `nanobus invoke` with a piped payload to see our greeting printed as output.

```sh
echo '{"name":"World"}' | nanobus invoke iota.yaml example.MyApi::greet
```

Output:

```sh
"Hello, World!"
```

## RequestStream/RequestChannel

RequestResponse actions are straightforward and similar to other ways of working with WebAssembly.

RequestStream actions take the same type of inputs as RequestResponse (i.e. anything) and return a stream rather than a future. RequestChannel actions take a single stream in and return a stream.

`wasmrs-request` takes input from STDIN when running in `--channel` mode. That's how we can demonstrate a RequestChannel action running meaningfully on the command line.

Add a `reverse` method to our API that takes a stream of `string` and outputs a stream of `string`. We'll pipe a file to our action and the output will be streamed, reversed, to STDOUT.

```graphql
namespace "example"

interface MyApi @service {
  greet(name: string): string
  reverse(input: stream string): stream string
}
```

Run `just codegen` to generate the new code:

```sh
$ just codegen
INFO Writing file ./src/actions/my_api/reverse.rs (mode:644)
INFO Writing file ./src/actions/mod.rs (mode:644)
INFO Formatting file ./src/actions/my_api/reverse.rs
INFO Formatting file ./src/actions/mod.rs
```

Our new stub looks a little different than the simple RequestResponse stub above:

```rs
use crate::actions::my_api_service::reverse::*;

pub(crate) async fn task(
    mut input: FluxReceiver<Inputs, PayloadError>,
    outputs: Flux<Outputs, PayloadError>,
) -> Result<Flux<Outputs, PayloadError>, crate::Error> {
    todo!("Add implementation");
}
```

> *WasmRS uses terminology from [RSocket](https://rsocket.io) and reactive-streams to stay consistent. A `Flux` is like a rust `Stream` mixed with a channel. You can push to it, pass it around, pipe one to another, and await values. A `FluxReceiver` is a `Flux` that you can only receive values from. It's like the receiving end of a channel implemented as a `Stream`.*

We can await from `Flux`es and `FluxReceiver`s the same as any Rust stream. Pushing to a `Flux` is done via `send()` & `error()` methods.

```rs
use crate::actions::my_api_service::reverse::*;

pub(crate) async fn task(
  mut input: FluxReceiver<Inputs, PayloadError>,
  outputs: Flux<Outputs, PayloadError>,
) -> Result<Flux<Outputs, PayloadError>, crate::Error> {
  while let Some(line) = input.next().await {
    match line {
      Ok(line) => {
        outputs.send(line.chars().rev().collect()).unwrap();
      }
      Err(e) => outputs.error(PayloadError::application_error(e.to_string())).unwrap(),
    }
  }
  outputs.complete();
  Ok(outputs)
}
```

Build with `just build`

```sh
just build
```

To run it with `wasmrs-request`, use the same path and action arguments as above with the addition of the `--channel` flag and piped input.

```sh
cat Cargo.toml |  wasmrs-request --channel ./build/example.wasm example.MyApi reverse
```

Now anything you pipe to our `reverse` action will come out reversed.

```console
]egakcap[
"elpmaxe" = eman
"0.1.0" = noisrev
"1202" = noitide

]bil[
]"bilydc"[ = epyt-etarc

]esaeler.eliforp[
"slobmys" = pirts
1 = stinu-negedoc
eslaf = gubed
eurt = otl
"z" = level-tpo
"troba" = cinap

]seicnedneped[
"2.0" = tseug-srmsaw
"0.1" = rorresiht
} ]"evired"[ = serutaef ,eslaf = serutaef-tluafed ,"1" = noisrev { = edres
"1.0" = tiart-cnysa
"0.82.0" = ajnijinim

]seicnedneped-ved[
```

RequestStream is used in a similar way except doesn't have a stream as input.

## Fire & Forget actions

Fire & Forget actions are not generated with the Apexlang code generators at this time.

## More links

- [wasmRS spec](https://github.com/nanobus/iota/blob/main/docs/wasmrs.md)
- [More on Iotas](https://github.com/nanobus/iota/blob/main/docs/iota-spec.md)
- [GitHub Repository](https://github.com/nanobus/iota/)
- [NanoBus](https://nanobus.io) ([github.com/nanobus/nanobus](https://github.com/nanobus/nanobus))
- [Apexlang](https://apexlang.io) ([github.com/apexlang/apex](https://github.com/apexlang/apex))
- [Candle Discord server](https://discord.gg/candle) to talk about WebAssembly, wasmRS, Apexlang, NanoBus, Rust, Go, Deno, TypeScript, and all the cool things.
