![NanoBus Logo](https://github.com/nanobus/nanobus/blob/main/docs/images/nanobus-logo.svgg)

NanoBus is a lightweight application runtime that reduces developer responsibility so that teams can **focus on core logic**.

For detailed information see the [overview](./docs/overview.md) and [architecture](./docs/architecture.md) pages.

## Install

**Linux** - Install from Terminal to `/usr/local/bin`:

```shell
wget -q https://raw.githubusercontent.com/nanobus/nanobus/main/install/install.sh -O - | /bin/bash
```

**MacOS** - Install from Terminal to `/usr/local/bin`:

```shell
curl -fsSL https://raw.githubusercontent.com/nanobus/nanobus/main/install/install.sh | /bin/bash
```

**Windows** - Install from Command Prompt:

```shell
powershell -Command "iwr -useb https://raw.githubusercontent.com/nanobus/nanobus/main/install/install.ps1 | iex"
```

> **Note**: Updates to PATH might not be visible until you restart your terminal application.

**Manually** - Select your architecture from [releases](https://github.com/nanobus/nanobus/releases).

## Create a simple application

Create a file called `bus.yaml` with the following contents:

```yaml
id: hello-world
version: 0.0.1
interfaces:
  Greeter:
    sayHello:
      steps:
        - name: Return greeting message
          # expr will evaluate a value and assign it
          # to the output of this pipeline.
          uses: expr
          with:
            # $ or pipe represent the input data
            # for this step.
            value: '"Hello, " + $.name'
```

Then run this command from your terminal:

```shell
echo '{"name": "World!"}' | nanobus invoke Greeter::sayHello
```

This should return `Hello, World!` as a JSON string. The JSON data returned by NanoBus applications can be piped to other utilities such as [jq](https://stedolan.github.io/jq/).

## Tutorials and examples

[Basic web service](https://github.com/nanobus/examples/tree/main/basic-web-service)<br>
[WebAssembly-powered web service](https://github.com/nanobus/examples/tree/main/wasm-web-service)<br>
[Dapr integration](https://github.com/nanobus/examples/tree/main/dapr)<br>
[URL Shortener](https://github.com/nanobus/examples/tree/main/urlshortener)<br>
[NanoChat](https://github.com/nanobus/examples/tree/main/nanochat)<br>

## Community support

For additional help, you can use one of these channels to ask a question:

- [Discord](https://discord.gg/candle) - Discussions with the community and the team.
- [GitHub](https://github.com/nanobus/nanobus/issues) - For bug reports and feature requests.
- [Twitter](https://twitter.com/nanobusdev) - Get the product updates easily.

## Developer setup

### Dependencies

To setup a local development environment

| Dependency | Check            | Description                                   |
|:---------- |:---------------- |:--------------------------------------------- |
| [go]       | $ go version     | Go compiler.  Ensure $HOME/go/bin is in PATH. |
| [just]     | $ just --version | Like Makefile [just] runs the needed commands |

### Install from source

```shell
git clone https://github.com/nanobus/nanobus.git
cd nanobus
just install
```

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on the code of conduct and the process for submitting pull requests.

## License

This project is licensed under the [Mozilla Public License Version 2.0](https://mozilla.org/MPL/2.0/).

[apex]: https://apexlang.io/docs/getting-started
[apexlang.io]: https://apexlang.io
[docker]: https://docs.docker.com/engine/install/
[docker-compose]: https://docs.docker.com/compose/install/
[go]: https://go.dev/doc/install
[iota]: https://github.com/nanobus/iota
[iotas]: https://github.com/nanobus/iota
[just]: https://github.com/casey/just#Installation
[nanobus]: https://github.com/nanobus/nanobus#Install
[npm]: https://docs.npmjs.com/downloading-and-installing-node-js-and-npm
[npx]: https://www.npmjs.com/package/npx#Install
[postgres]: https://www.postgresql.org/download/
[postgresql database]: https://www.postgresql.org/
[rust]: https://rustup.rs/
